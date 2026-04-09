//! Deploy subgraph to graph-node via IPFS + JSON-RPC.
//!
//! Flow:
//! 1. Parse subgraph.yaml to find all local file references
//! 2. Upload each file (WASM, schema, ABIs) to IPFS
//! 3. Rewrite manifest with IPFS hashes
//! 4. Upload the manifest itself to IPFS
//! 5. JSON-RPC: subgraph_create + subgraph_deploy

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

const DEFAULT_IPFS: &str = "http://localhost:5001";
const DEFAULT_NODE: &str = "http://localhost:8020";

pub fn deploy(
    node: Option<&str>,
    ipfs: Option<&str>,
    name: &str,
    deploy_key: Option<&str>,
    version_label: Option<&str>,
) -> Result<()> {
    let ipfs_url = ipfs.unwrap_or(DEFAULT_IPFS);
    let node_url = node.unwrap_or(DEFAULT_NODE);

    // Find subgraph.yaml
    let manifest_path = find_manifest()?;
    println!("  Manifest: {}", manifest_path.display());

    // Parse it
    let manifest_str = std::fs::read_to_string(&manifest_path)
        .with_context(|| format!("Failed to read {}", manifest_path.display()))?;
    let manifest: serde_yaml::Value =
        serde_yaml::from_str(&manifest_str).context("Failed to parse subgraph.yaml")?;

    // Collect all file paths from the manifest
    let manifest_dir = manifest_path.parent().unwrap_or_else(|| Path::new("."));
    let files = collect_file_refs(&manifest)?;
    println!("  Found {} file references", files.len());

    // Upload each file to IPFS and build a path->hash map
    let mut path_to_hash: Vec<(String, String)> = Vec::new();
    for file_ref in &files {
        let resolved = manifest_dir.join(file_ref);
        let resolved = resolved.canonicalize().with_context(|| {
            format!(
                "File not found: {} (resolved from {})",
                resolved.display(),
                file_ref
            )
        })?;

        let hash = upload_to_ipfs(ipfs_url, &resolved, deploy_key)
            .with_context(|| format!("Failed to upload {} to IPFS", resolved.display()))?;
        println!("  Uploaded {} -> {}", file_ref, hash);
        path_to_hash.push((file_ref.clone(), hash));
    }

    // Rewrite manifest: replace local paths with IPFS references
    let ipfs_manifest = rewrite_manifest(&manifest_str, &path_to_hash);

    // Upload manifest to IPFS
    let manifest_hash =
        upload_bytes_to_ipfs(ipfs_url, ipfs_manifest.as_bytes(), "subgraph.yaml", deploy_key)
            .context("Failed to upload manifest to IPFS")?;
    println!("  Manifest uploaded: {}", manifest_hash);

    // Create subgraph name (idempotent — ignores "already exists" errors)
    // Studio doesn't use subgraph_create; skip if we have a deploy key.
    if deploy_key.is_none() {
        println!("  Creating subgraph name: {}", name);
        match jsonrpc_call(node_url, "subgraph_create", &serde_json::json!({"name": name}), deploy_key) {
            Ok(result) => println!("  Created: {}", result),
            Err(e) => {
                let msg = e.to_string();
                if msg.contains("already") {
                    println!("  (already exists, continuing)");
                } else {
                    return Err(e).context("subgraph_create failed");
                }
            }
        }
    }

    // Deploy
    println!("  Deploying...");
    let mut deploy_params = serde_json::json!({"name": name, "ipfs_hash": manifest_hash});
    if let Some(label) = version_label {
        deploy_params["version_label"] = serde_json::Value::String(label.to_string());
    }
    let result = jsonrpc_call(node_url, "subgraph_deploy", &deploy_params, deploy_key)
        .context("subgraph_deploy failed")?;
    println!("  Deployed: {}", result);

    println!();
    println!("Subgraph deployed successfully!");
    println!("  Name: {}", name);
    println!("  IPFS hash: {}", manifest_hash);
    let query_base = node_url.replace(":8020", ":8000");
    println!("  Query URL: {}/subgraphs/name/{}", query_base, name);

    Ok(())
}

fn find_manifest() -> Result<PathBuf> {
    for name in ["subgraph.yaml", "subgraph.yml"] {
        let p = PathBuf::from(name);
        if p.exists() {
            return Ok(p);
        }
    }
    anyhow::bail!("subgraph.yaml not found in current directory")
}

/// Walk the YAML tree and collect all `file:` values that are plain strings (local paths).
fn collect_file_refs(value: &serde_yaml::Value) -> Result<Vec<String>> {
    let mut refs = Vec::new();
    collect_file_refs_inner(value, false, &mut refs);
    Ok(refs)
}

fn collect_file_refs_inner(value: &serde_yaml::Value, is_file_key: bool, refs: &mut Vec<String>) {
    match value {
        serde_yaml::Value::String(s) if is_file_key => {
            // Only collect local paths, not already-IPFS references
            if !s.starts_with("/ipfs/") && !s.starts_with("Qm") {
                refs.push(s.clone());
            }
        }
        serde_yaml::Value::Mapping(map) => {
            for (k, v) in map {
                let key_is_file = matches!(k, serde_yaml::Value::String(s) if s == "file");
                collect_file_refs_inner(v, key_is_file, refs);
            }
        }
        serde_yaml::Value::Sequence(seq) => {
            for item in seq {
                collect_file_refs_inner(item, false, refs);
            }
        }
        _ => {}
    }
}

/// Rewrite the manifest text, replacing local file paths with IPFS link objects.
///
/// Graph-node expects IPFS file references in the format:
///   file:
///     /: /ipfs/QmHash
///
/// The indentation of `/: /ipfs/...` is 2 more than the `file:` line.
fn rewrite_manifest(manifest: &str, replacements: &[(String, String)]) -> String {
    let mut lines: Vec<String> = manifest.lines().map(String::from).collect();

    for (path, hash) in replacements {
        for i in 0..lines.len() {
            let line = &lines[i];
            // Match lines like `      file: ./path` or `  file: "./path"`
            let trimmed = line.trim_start();
            if !trimmed.starts_with("file:") {
                continue;
            }
            let after_file = trimmed.strip_prefix("file:").unwrap().trim();
            let unquoted = after_file.trim_matches('"');
            if unquoted != path {
                continue;
            }
            // Found the line — compute indentation
            let indent = line.len() - trimmed.len();
            let child_indent = " ".repeat(indent + 2);
            lines[i] = format!("{}file:", " ".repeat(indent));
            lines.insert(i + 1, format!("{}/: /ipfs/{}", child_indent, hash));
            break;
        }
    }

    lines.join("\n")
}

/// Upload a file to IPFS, returns the hash.
fn upload_to_ipfs(ipfs_url: &str, path: &Path, auth: Option<&str>) -> Result<String> {
    let data = std::fs::read(path).with_context(|| format!("Failed to read {}", path.display()))?;
    let filename = path
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "file".to_string());
    upload_bytes_to_ipfs(ipfs_url, &data, &filename, auth)
}

/// Upload raw bytes to IPFS, returns the hash.
fn upload_bytes_to_ipfs(
    ipfs_url: &str,
    data: &[u8],
    filename: &str,
    auth: Option<&str>,
) -> Result<String> {
    let url = format!("{}/api/v0/add", ipfs_url);

    let boundary = "----graphite-boundary-a1b2c3";
    let mut body = Vec::new();

    body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
    body.extend_from_slice(
        format!(
            "Content-Disposition: form-data; name=\"file\"; filename=\"{}\"\r\n",
            filename
        )
        .as_bytes(),
    );
    body.extend_from_slice(b"Content-Type: application/octet-stream\r\n\r\n");
    body.extend_from_slice(data);
    body.extend_from_slice(format!("\r\n--{}--\r\n", boundary).as_bytes());

    let mut req = ureq::post(&url).header(
        "Content-Type",
        &format!("multipart/form-data; boundary={}", boundary),
    );
    if let Some(key) = auth {
        req = req.header("Authorization", &format!("Bearer {}", key));
    }
    let mut response = req
        .send(&body)
        .with_context(|| format!("IPFS upload failed for {}", filename))?;

    let json: serde_json::Value = response
        .body_mut()
        .read_json()
        .context("Failed to parse IPFS response")?;

    json["Hash"]
        .as_str()
        .map(String::from)
        .context("IPFS response missing 'Hash' field")
}

/// Make a JSON-RPC call to graph-node.
fn jsonrpc_call(
    node_url: &str,
    method: &str,
    params: &serde_json::Value,
    auth: Option<&str>,
) -> Result<serde_json::Value> {
    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": 1
    });

    let mut req = ureq::post(node_url);
    if let Some(key) = auth {
        req = req.header("Authorization", &format!("Bearer {}", key));
    }
    let mut resp = req
        .send_json(&body)
        .with_context(|| format!("JSON-RPC call to {} failed", method))?;

    let response: serde_json::Value = resp
        .body_mut()
        .read_json()
        .context("Failed to parse JSON-RPC response")?;

    if let Some(error) = response.get("error") {
        let msg = error["message"].as_str().unwrap_or("unknown error");
        anyhow::bail!("{}: {}", method, msg);
    }

    Ok(response["result"].clone())
}
