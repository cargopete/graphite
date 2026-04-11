//! `graphite manifest` — generates subgraph.yaml from graphite.toml.
//!
//! Reads the ABI files to discover event signatures and constructs the
//! graph-node manifest automatically. Block handlers and call handlers
//! are emitted when declared in graphite.toml.

use anyhow::{Context, Result};
use std::fmt::Write as FmtWrite;
use std::path::PathBuf;

// Re-use the same config types from main.rs via pub(crate).
use crate::{ContractConfig, GraphiteConfig};

pub fn generate(config_path: &PathBuf, output_path: &PathBuf) -> Result<()> {
    let config_str = std::fs::read_to_string(config_path)
        .with_context(|| format!("Failed to read {}", config_path.display()))?;
    let config: GraphiteConfig = toml::from_str(&config_str)
        .with_context(|| format!("Failed to parse {}", config_path.display()))?;

    let network = config.network.as_deref().unwrap_or("mainnet");

    // Discover crate name from Cargo.toml (for the WASM file path).
    let crate_name = read_crate_name().unwrap_or_else(|_| "subgraph".to_string());
    let wasm_path = format!(
        "./target/wasm32-unknown-unknown/release/{}.wasm",
        crate_name.replace('-', "_")
    );

    // Collect entity names from schema.graphql (for the `entities:` list).
    let entity_names = config
        .schema
        .as_ref()
        .and_then(|p| collect_entity_names(p).ok())
        .unwrap_or_default();

    let mut out = String::new();
    writeln!(out, "specVersion: 0.0.5").unwrap();
    writeln!(out, "schema:").unwrap();
    writeln!(out, "  file: ./schema.graphql").unwrap();

    // ── dataSources ──────────────────────────────────────────────────────────
    if !config.contracts.is_empty() {
        writeln!(out, "dataSources:").unwrap();
        for c in &config.contracts {
            let events = load_events(&c.abi)?;
            write_datasource(&mut out, c, network, &entity_names, &events, &wasm_path, false)?;
        }
    }

    // ── templates ────────────────────────────────────────────────────────────
    if !config.templates.is_empty() {
        writeln!(out, "templates:").unwrap();
        for t in &config.templates {
            let events = load_events(&t.abi)?;
            write_datasource(&mut out, t, network, &entity_names, &events, &wasm_path, true)?;
        }
    }

    std::fs::write(output_path, &out)
        .with_context(|| format!("Failed to write {}", output_path.display()))?;

    println!("  Written: {}", output_path.display());
    println!();
    println!("Review and adjust addresses / startBlock as needed, then run:");
    println!("  graphite build && graphite deploy <name>");
    Ok(())
}

fn write_datasource(
    out: &mut String,
    c: &ContractConfig,
    network: &str,
    entity_names: &[String],
    events: &[EventSig],
    wasm_path: &str,
    is_template: bool,
) -> Result<()> {
    writeln!(out, "  - kind: ethereum/contract").unwrap();
    writeln!(out, "    name: {}", c.name).unwrap();
    writeln!(out, "    network: {}", network).unwrap();
    writeln!(out, "    source:").unwrap();

    if is_template {
        // Templates have no address / startBlock — those come at runtime.
        writeln!(out, "      abi: {}", c.name).unwrap();
    } else {
        let address = c.address.as_deref().unwrap_or("0x0000000000000000000000000000000000000000");
        let start_block = c.start_block.unwrap_or(0);
        writeln!(out, "      address: \"{}\"", address).unwrap();
        writeln!(out, "      abi: {}", c.name).unwrap();
        writeln!(out, "      startBlock: {}", start_block).unwrap();
    }

    writeln!(out, "    mapping:").unwrap();
    writeln!(out, "      kind: ethereum/events").unwrap();
    writeln!(out, "      apiVersion: 0.0.7").unwrap();
    writeln!(out, "      language: wasm/assemblyscript").unwrap();

    // Entities list
    writeln!(out, "      entities:").unwrap();
    if entity_names.is_empty() {
        writeln!(out, "        - Entity  # replace with your entity names").unwrap();
    } else {
        for e in entity_names {
            writeln!(out, "        - {}", e).unwrap();
        }
    }

    // ABIs
    writeln!(out, "      abis:").unwrap();
    writeln!(out, "        - name: {}", c.name).unwrap();
    writeln!(
        out,
        "          file: ./{}",
        c.abi.display()
    )
    .unwrap();

    // Event handlers
    if !events.is_empty() {
        writeln!(out, "      eventHandlers:").unwrap();
        for ev in events {
            writeln!(out, "        - event: {}", ev.signature).unwrap();
            writeln!(out, "          handler: handle{}", ev.name).unwrap();
        }
    }

    // Call handlers
    if !c.call_handlers.is_empty() {
        writeln!(out, "      callHandlers:").unwrap();
        for ch in &c.call_handlers {
            writeln!(out, "        - function: {}", ch.function).unwrap();
            writeln!(out, "          handler: {}", ch.handler).unwrap();
        }
    }

    // Block handlers
    if !c.block_handlers.is_empty() {
        writeln!(out, "      blockHandlers:").unwrap();
        for bh in &c.block_handlers {
            writeln!(out, "        - handler: {}", bh.handler).unwrap();
            if let Some(filter) = &bh.filter {
                writeln!(out, "          filter:").unwrap();
                writeln!(out, "            kind: {}", filter.kind).unwrap();
                if let Some(every) = filter.every {
                    writeln!(out, "            every: {}", every).unwrap();
                }
            }
        }
    }

    if c.receipt {
        writeln!(out, "      receipt: true").unwrap();
    }
    writeln!(out, "      file: {}", wasm_path).unwrap();
    Ok(())
}

struct EventSig {
    name: String,
    signature: String,
}

/// Load events from an ABI JSON file and return their graph-node signatures.
fn load_events(abi_path: &PathBuf) -> Result<Vec<EventSig>> {
    let abi_str = std::fs::read_to_string(abi_path)
        .with_context(|| format!("Failed to read ABI: {}", abi_path.display()))?;
    let abi: serde_json::Value = serde_json::from_str(&abi_str)
        .with_context(|| format!("Failed to parse ABI: {}", abi_path.display()))?;

    let items = match abi.as_array() {
        Some(a) => a,
        None => return Ok(vec![]),
    };

    let mut events = Vec::new();
    for item in items {
        let ty = item.get("type").and_then(|v| v.as_str()).unwrap_or("");
        if ty != "event" {
            continue;
        }
        let name = item.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
        if name.is_empty() {
            continue;
        }

        // Build the signature in graph-node format:
        // EventName(indexed type,type,...) — indexed params get the prefix.
        let params = item.get("inputs").and_then(|v| v.as_array()).map(|arr| {
            arr.iter()
                .map(|p| {
                    let indexed = p.get("indexed").and_then(|v| v.as_bool()).unwrap_or(false);
                    let ty = p.get("type").and_then(|v| v.as_str()).unwrap_or("bytes");
                    if indexed {
                        format!("indexed {}", ty)
                    } else {
                        ty.to_string()
                    }
                })
                .collect::<Vec<_>>()
                .join(",")
        }).unwrap_or_default();

        let signature = format!("{}({})", name, params);
        events.push(EventSig { name, signature });
    }

    Ok(events)
}

/// Read the `name` field from ./Cargo.toml.
fn read_crate_name() -> Result<String> {
    let cargo_toml = std::fs::read_to_string("Cargo.toml").context("Failed to read Cargo.toml")?;
    cargo_toml
        .lines()
        .find(|l| l.trim_start().starts_with("name"))
        .and_then(|l| l.split('=').nth(1))
        .map(|s| s.trim().trim_matches('"').to_string())
        .context("Could not find crate name in Cargo.toml")
}

/// Parse entity names from schema.graphql (@entity types, excluding _Schema_ and aggregations).
fn collect_entity_names(schema_path: &PathBuf) -> Result<Vec<String>> {
    let src = std::fs::read_to_string(schema_path)
        .with_context(|| format!("Failed to read {}", schema_path.display()))?;

    let doc = graphql_parser::schema::parse_schema::<String>(&src)
        .map_err(|e| anyhow::anyhow!("schema parse error: {}", e))?;

    let mut names = Vec::new();
    for def in &doc.definitions {
        use graphql_parser::schema::Definition;
        if let Definition::TypeDefinition(graphql_parser::schema::TypeDefinition::Object(obj)) = def {
            // Skip internal / non-entity types
            if obj.name.starts_with("__")
                || obj.name == "_Schema_"
                || obj.name == "Query"
                || obj.name == "Mutation"
                || obj.name == "Subscription"
            {
                continue;
            }
            if !obj.directives.iter().any(|d| d.name == "entity") {
                continue;
            }
            if obj.directives.iter().any(|d| d.name == "aggregation") {
                continue;
            }
            names.push(obj.name.clone());
        }
    }
    Ok(names)
}
