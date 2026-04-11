//! Graphite CLI — build tooling for Rust subgraphs.
//!
//! Commands:
//! - `graphite init`     — scaffold a new subgraph project
//! - `graphite codegen`  — generate Rust types from ABI + schema
//! - `graphite manifest` — generate subgraph.yaml from graphite.toml
//! - `graphite build`    — compile to WASM
//! - `graphite test`     — run tests (delegates to cargo test)
//! - `graphite deploy`   — deploy to graph-node

mod codegen;
mod deploy;
mod manifest;

use anyhow::{Context, Result};
use heck::ToUpperCamelCase;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "graphite")]
#[command(about = "Build tooling for Rust subgraphs on The Graph")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new subgraph project
    Init {
        /// Project name
        #[arg(default_value = "my-subgraph")]
        name: String,

        /// Initialize from a contract address
        #[arg(long)]
        from_contract: Option<String>,

        /// Network (mainnet, goerli, etc.)
        #[arg(long, default_value = "mainnet")]
        network: String,
    },

    /// Generate Rust types from ABI and GraphQL schema
    Codegen {
        /// Path to graphite.toml config (default: ./graphite.toml)
        #[arg(long, short)]
        config: Option<PathBuf>,

        /// Watch ABI and schema files for changes and re-run automatically
        #[arg(long, short)]
        watch: bool,
    },

    /// Generate subgraph.yaml from graphite.toml
    Manifest {
        /// Path to graphite.toml config (default: ./graphite.toml)
        #[arg(long, short)]
        config: Option<PathBuf>,

        /// Output path (default: ./subgraph.yaml)
        #[arg(long, short)]
        output: Option<PathBuf>,
    },

    /// Compile the subgraph to WASM
    Build {
        /// Build in release mode
        #[arg(long, default_value = "true")]
        release: bool,
    },

    /// Run tests
    Test {
        /// Generate HTML coverage report via cargo-llvm-cov (must be installed)
        #[arg(long)]
        coverage: bool,

        /// Additional arguments to pass to cargo test / cargo llvm-cov
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },

    /// Deploy the subgraph
    Deploy {
        /// Graph node URL
        #[arg(long)]
        node: Option<String>,

        /// IPFS URL
        #[arg(long)]
        ipfs: Option<String>,

        /// Deploy key for authentication (Graph Studio)
        #[arg(long)]
        deploy_key: Option<String>,

        /// Version label (Graph Studio)
        #[arg(long)]
        version_label: Option<String>,

        /// Subgraph name (e.g., "username/subgraph-name")
        name: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init {
            name,
            from_contract,
            network,
        } => {
            println!("Initializing new subgraph: {}", name);
            if let Some(ref contract) = from_contract {
                println!("  From contract: {} on {}", contract, network);
            }
            cmd_init(&name, from_contract.as_deref(), &network)
        }

        Commands::Codegen { config, watch } => {
            let config_path = config.unwrap_or_else(|| PathBuf::from("graphite.toml"));
            if watch {
                println!("Watching for changes (Ctrl+C to stop)...");
                cmd_codegen_watch(&config_path)
            } else {
                println!("Generating types from ABI and schema...");
                cmd_codegen(&config_path)
            }
        }

        Commands::Manifest { config, output } => {
            let config_path = config.unwrap_or_else(|| PathBuf::from("graphite.toml"));
            let output_path = output.unwrap_or_else(|| PathBuf::from("subgraph.yaml"));
            println!("Generating subgraph.yaml...");
            cmd_manifest(&config_path, &output_path)
        }

        Commands::Build { release } => {
            println!(
                "Building subgraph ({} mode)...",
                if release { "release" } else { "debug" }
            );
            cmd_build(release)
        }

        Commands::Test { coverage, args } => {
            if coverage {
                println!("Running tests with coverage...");
                cmd_test_coverage(&args)
            } else {
                println!("Running tests...");
                cmd_test(&args)
            }
        }

        Commands::Deploy {
            node,
            ipfs,
            deploy_key,
            version_label,
            name,
        } => {
            println!("Deploying subgraph: {}", name);
            cmd_deploy(
                node.as_deref(),
                ipfs.as_deref(),
                &name,
                deploy_key.as_deref(),
                version_label.as_deref(),
            )
        }
    }
}

/// Fetch a verified contract ABI from Etherscan V2 (or compatible explorer).
///
/// Uses the `ETHERSCAN_API_KEY` env var if set. Falls back to no key (works
/// for many contracts but is rate-limited). Supports all major EVM networks
/// via Etherscan's unified V2 API (chainId parameter).
fn fetch_etherscan_abi(address: &str, network: &str) -> Result<String> {
    let api_key = std::env::var("ETHERSCAN_API_KEY").unwrap_or_default();

    // Etherscan V2 unified API — single endpoint, chain selected by chainid param.
    let chain_id: u64 = match network {
        "mainnet" | "ethereum" => 1,
        "goerli" => 5,
        "sepolia" => 11155111,
        "holesky" => 17000,
        "polygon" | "matic" => 137,
        "polygon-mumbai" | "mumbai" => 80001,
        "arbitrum-one" | "arbitrum" => 42161,
        "arbitrum-nova" => 42170,
        "arbitrum-sepolia" => 421614,
        "optimism" => 10,
        "optimism-sepolia" => 11155420,
        "base" => 8453,
        "base-sepolia" => 84532,
        "bsc" | "binance" => 56,
        "bsc-testnet" => 97,
        "avalanche" => 43114,
        "avalanche-fuji" | "fuji" => 43113,
        "gnosis" | "xdai" => 100,
        "linea" => 59144,
        "scroll" => 534352,
        "zksync" | "zksync-era" => 324,
        "blast" => 81457,
        "mantle" => 5000,
        "celo" => 42220,
        "fantom" => 250,
        _ => 1, // default to mainnet
    };

    let url = format!(
        "https://api.etherscan.io/v2/api?chainid={}&module=contract&action=getabi&address={}&apikey={}",
        chain_id, address, api_key
    );

    let body: serde_json::Value = ureq::get(&url)
        .call()
        .context("HTTP request to Etherscan failed")?
        .body_mut()
        .read_json()
        .context("Failed to parse Etherscan response as JSON")?;

    let status = body.get("status").and_then(|v| v.as_str()).unwrap_or("");
    if status != "1" {
        let message = body
            .get("result")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown error");
        anyhow::bail!("Etherscan returned an error: {}", message);
    }

    body.get("result")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .context("Missing 'result' field in Etherscan response")
}

fn cmd_init(name: &str, from_contract: Option<&str>, network: &str) -> Result<()> {
    let project_dir = PathBuf::from(name);

    // Check if directory already exists
    if project_dir.exists() {
        anyhow::bail!("Directory '{}' already exists", name);
    }

    // Create directory structure
    std::fs::create_dir_all(project_dir.join("src"))?;
    std::fs::create_dir_all(project_dir.join("abis"))?;

    // Optionally fetch ABI from Etherscan
    let (abi_json, contract_address) = if let Some(address) = from_contract {
        println!("  Fetching ABI from Etherscan for {}...", address);
        match fetch_etherscan_abi(address, network) {
            Ok(abi) => {
                println!("  ABI fetched successfully.");
                (abi, Some(address.to_string()))
            }
            Err(e) => {
                eprintln!("  Warning: could not fetch ABI: {}", e);
                eprintln!("  Falling back to placeholder ABI.");
                (placeholder_abi(), None)
            }
        }
    } else {
        (placeholder_abi(), None)
    };

    // Create Cargo.toml
    let cargo_toml = format!(
        r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
graphite = {{ package = "graphite-sdk", version = "1", default-features = false }}
graphite-macros = {{ version = "1" }}
graph-as-runtime = {{ version = "1" }}

[dev-dependencies]
graphite = {{ package = "graphite-sdk", version = "1", features = ["std"] }}

[profile.release]
opt-level = "z"
lto = true
panic = "abort"
"#,
        name = name
    );
    std::fs::write(project_dir.join("Cargo.toml"), cargo_toml)?;
    println!("  Created Cargo.toml");

    // Create graphite.toml
    let contracts_section = if let Some(ref addr) = contract_address {
        format!(
            r#"
[[contracts]]
name = "{name}"
abi = "abis/{name}.json"
address = "{addr}"
start_block = 0
network = "{network}"
"#,
            name = name,
            addr = addr,
            network = network,
        )
    } else {
        format!(
            r#"
[[contracts]]
name = "{name}"
abi = "abis/{name}.json"
address = "0x0000000000000000000000000000000000000000"  # replace with your contract address
start_block = 0  # replace with deployment block

# Dynamic data source templates (factory pattern):
# [[templates]]
# name = "Pair"
# abi = "abis/Pair.json"
"#,
            name = name,
        )
    };
    let graphite_toml = format!(
        "# Graphite subgraph configuration\n\noutput_dir = \"src/generated\"\nschema = \"schema.graphql\"\nnetwork = \"{network}\"\n{contracts_section}",
        network = network,
        contracts_section = contracts_section,
    );
    std::fs::write(project_dir.join("graphite.toml"), graphite_toml)?;
    println!("  Created graphite.toml");

    // Create subgraph.yaml (The Graph manifest)
    let scaffold_address = contract_address
        .as_deref()
        .unwrap_or("0x0000000000000000000000000000000000000000");
    let subgraph_yaml = format!(
        r#"specVersion: 0.0.5
schema:
  file: ./schema.graphql
# Grafting — uncomment to resume indexing from a previously deployed subgraph.
# This avoids re-indexing from block 0 when you redeploy with schema changes.
#graft:
#  base: Qm...              # subgraph deployment ID (IPFS hash) to graft from
#  block: 12345678          # block number to start from (must exist in base)
dataSources:
  - kind: ethereum
    name: {name}
    network: {network}
    source:
      address: "{scaffold_address}"
      abi: {name}
      startBlock: 0
    mapping:
      kind: ethereum/events
      apiVersion: 0.0.7
      language: wasm/assemblyscript
      entities:
        - ExampleEntity
      abis:
        - name: {name}
          file: ./abis/{name}.json
      eventHandlers:
        - event: Transfer(indexed address,indexed address,uint256)
          handler: handle_transfer
      file: ./build/{name_snake}.wasm
"#,
        name = name,
        network = network,
        scaffold_address = scaffold_address,
        name_snake = name.replace('-', "_")
    );
    std::fs::write(project_dir.join("subgraph.yaml"), subgraph_yaml)?;
    println!("  Created subgraph.yaml");

    // Create schema.graphql
    let schema = r#"# Example entity - replace with your own schema

type ExampleEntity @entity {
  id: ID!
  count: BigInt!
  sender: Bytes!
  value: BigInt!
}
"#;
    std::fs::write(project_dir.join("schema.graphql"), schema)?;
    println!("  Created schema.graphql");

    // Create src/lib.rs — event type name is {ContractIdent}TransferEvent
    // Use the same naming logic as codegen/abi.rs: only camel-case if hyphens/underscores present.
    let contract_ident = if name.contains('-') || name.contains('_') {
        name.to_upper_camel_case()
    } else {
        name.to_owned()
    };
    let transfer_event = format!("{}TransferEvent", contract_ident);
    let lib_rs = format!(
        r#"//! Subgraph handlers — generated by `graphite init`.
//!
//! Run `graphite codegen` first to populate `src/generated/` from your ABI
//! and schema, then adapt the handler below to your contract's events.

#![cfg_attr(target_arch = "wasm32", no_std)]
extern crate alloc;

use alloc::format;
use graphite_macros::handler;

mod generated;
use generated::{{ExampleEntity, {transfer_event}}};

/// Handle a Transfer event.
///
/// Builds a unique entity ID from the transaction hash and log index, then
/// creates (or overwrites) an ExampleEntity in the store.
#[handler]
pub fn handle_transfer(event: &{transfer_event}, _ctx: &graphite::EventContext) {{
    let id = format!(
        "{{}}-{{}}",
        hex_bytes(&event.tx_hash),
        hex_bytes(&event.log_index),
    );

    ExampleEntity::new(&id)
        .set_sender(event.from.to_vec())
        .set_value(event.value.clone())
        .save();
}}

/// Format a byte slice as a lowercase hex string (no 0x prefix).
fn hex_bytes(b: &[u8]) -> alloc::string::String {{
    b.iter().map(|x| format!("{{:02x}}", x)).collect()
}}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {{
    use super::*;
    use graph_as_runtime::ethereum::{{EthereumValue, EventParam, FromRawEvent, RawEthereumEvent}};
    use graphite::mock;

    fn mock_transfer_event() -> RawEthereumEvent {{
        RawEthereumEvent {{
            tx_hash: [0xab; 32],
            log_index: alloc::vec![0],
            block_number: alloc::vec![1, 0, 0, 0],
            block_timestamp: alloc::vec![100, 0, 0, 0],
            params: alloc::vec![
                EventParam {{ name: "from".into(), value: EthereumValue::Address([0xaa; 20]) }},
                EventParam {{ name: "to".into(),   value: EthereumValue::Address([0xbb; 20]) }},
                EventParam {{ name: "value".into(), value: EthereumValue::Uint(alloc::vec![42]) }},
            ],
            ..Default::default()
        }}
    }}

    #[test]
    fn transfer_creates_entity() {{
        mock::reset();

        handle_transfer_impl(
            &{transfer_event}::from_raw_event(&mock_transfer_event()).unwrap(),
            &graphite::EventContext::default(),
        );

        let tx_hex = "ab".repeat(32);
        let id = format!("{{}}-00", tx_hex);
        assert!(mock::has_entity("ExampleEntity", &id));
        mock::assert_entity("ExampleEntity", &id)
            .field_bytes("sender", &[0xaa; 20])
            .field_exists("value");
    }}
}}
"#,
        transfer_event = transfer_event,
    );
    std::fs::write(project_dir.join("src/lib.rs"), lib_rs)?;
    println!("  Created src/lib.rs");

    // Write ABI (fetched or placeholder)
    std::fs::write(
        project_dir.join(format!("abis/{}.json", name)),
        &abi_json,
    )?;
    if contract_address.is_some() {
        println!("  Created abis/{}.json (from Etherscan)", name);
    } else {
        println!("  Created abis/{}.json (placeholder)", name);
    }

    // Create .gitignore
    let gitignore = r#"/target/
Cargo.lock
src/generated/
"#;
    std::fs::write(project_dir.join(".gitignore"), gitignore)?;
    println!("  Created .gitignore");

    println!();
    println!("Project '{}' created successfully!", name);
    println!();
    println!("Next steps:");
    println!("  cd {}", name);
    println!("  # Add your contract ABI to abis/");
    println!("  # Update graphite.toml with your contract address");
    println!("  # Edit schema.graphql with your entities");
    println!("  graphite codegen    # generate Rust types from ABI + schema");
    println!("  graphite manifest   # generate subgraph.yaml");
    println!("  graphite build      # compile to WASM");
    println!("  graphite test       # run native tests (no Docker needed)");

    Ok(())
}

/// Configuration for codegen, read from graphite.toml
#[derive(Debug, serde::Deserialize)]
pub(crate) struct GraphiteConfig {
    /// Output directory for generated code
    #[serde(default = "default_output_dir")]
    pub(crate) output_dir: PathBuf,
    /// Path to GraphQL schema file
    pub(crate) schema: Option<PathBuf>,
    /// Target network (mainnet, arbitrum-one, etc.) — used by `graphite manifest`
    pub(crate) network: Option<String>,
    /// Data source contract definitions
    #[serde(default)]
    pub(crate) contracts: Vec<ContractConfig>,
    /// Dynamic data source template definitions — same ABI bindings as contracts,
    /// but listed under `templates:` in the subgraph manifest.
    #[serde(default)]
    pub(crate) templates: Vec<ContractConfig>,
}

fn default_output_dir() -> PathBuf {
    PathBuf::from("src/generated")
}

#[derive(Debug, serde::Deserialize)]
pub(crate) struct ContractConfig {
    /// Contract name (used for struct prefixes)
    pub(crate) name: String,
    /// Path to the ABI JSON file
    pub(crate) abi: PathBuf,
    /// Deployed contract address (used by `graphite manifest`)
    pub(crate) address: Option<String>,
    /// Block number to start indexing from (used by `graphite manifest`)
    pub(crate) start_block: Option<u64>,
    /// Block handlers (optional filter support)
    #[serde(default)]
    pub(crate) block_handlers: Vec<BlockHandlerConfig>,
    /// Call handlers — explicit function signature + handler name pairs
    #[serde(default)]
    pub(crate) call_handlers: Vec<CallHandlerConfig>,
    /// If true, emit `receipt: true` in the mapping section.
    /// Enables `ctx.receipt` in event handlers (requires graph-node ≥ 0.26).
    #[serde(default)]
    pub(crate) receipt: bool,
}

/// A block handler entry in graphite.toml.
///
/// ```toml
/// [[contracts.block_handlers]]
/// handler = "handleBlock"
///
/// [[contracts.block_handlers]]
/// handler = "handleBlockPolled"
/// filter = { kind = "polling", every = 10 }
/// ```
#[derive(Debug, serde::Deserialize)]
pub(crate) struct BlockHandlerConfig {
    pub(crate) handler: String,
    pub(crate) filter: Option<BlockHandlerFilter>,
}

/// Optional filter on a block handler.
#[derive(Debug, serde::Deserialize)]
pub(crate) struct BlockHandlerFilter {
    pub(crate) kind: String,
    /// Used when `kind = "polling"` — emit block every N blocks.
    pub(crate) every: Option<u64>,
}

/// A call handler entry in graphite.toml.
///
/// ```toml
/// [[contracts.call_handlers]]
/// function = "transfer(address,uint256)"
/// handler = "handleTransfer"
/// ```
#[derive(Debug, serde::Deserialize)]
pub(crate) struct CallHandlerConfig {
    pub(crate) function: String,
    pub(crate) handler: String,
}

fn cmd_codegen(config_path: &PathBuf) -> Result<()> {
    // Read config
    let config_str = std::fs::read_to_string(config_path)
        .with_context(|| format!("Failed to read config: {}", config_path.display()))?;
    let config: GraphiteConfig = toml::from_str(&config_str)
        .with_context(|| format!("Failed to parse config: {}", config_path.display()))?;

    // Create output directory
    std::fs::create_dir_all(&config.output_dir).with_context(|| {
        format!(
            "Failed to create output dir: {}",
            config.output_dir.display()
        )
    })?;

    // Generate mod.rs for the generated module
    let mut mod_contents = String::from("//! Generated code — do not edit.\n\n");

    // Generate schema entities if specified
    if let Some(ref schema_path) = config.schema {
        println!("  Generating entities from schema...");

        let code = codegen::generate_schema_entities(schema_path).with_context(|| {
            format!("Failed to generate entities from {}", schema_path.display())
        })?;

        let output_path = config.output_dir.join("schema.rs");
        std::fs::write(&output_path, &code)
            .with_context(|| format!("Failed to write {}", output_path.display()))?;

        mod_contents.push_str("mod schema;\npub use schema::*;\n\n");
        println!("    → {}", output_path.display());
    }

    // Generate contract bindings (dataSources + templates share the same codegen)
    let all_contracts = config.contracts.iter().chain(config.templates.iter());
    for contract in all_contracts {
        println!("  Generating bindings for {}...", contract.name);

        let code = codegen::generate_abi_bindings(&contract.abi, &contract.name)
            .with_context(|| format!("Failed to generate bindings for {}", contract.name))?;

        // Write the generated file — use snake_case for valid Rust module names
        let mod_name = contract.name.to_lowercase().replace('-', "_");
        let filename = format!("{}.rs", mod_name);
        let output_path = config.output_dir.join(&filename);
        std::fs::write(&output_path, &code)
            .with_context(|| format!("Failed to write {}", output_path.display()))?;

        // Add to mod.rs
        mod_contents.push_str(&format!(
            "mod {};\npub use {}::*;\n\n",
            mod_name, mod_name
        ));

        println!("    → {}", output_path.display());
    }

    // Write mod.rs
    let mod_path = config.output_dir.join("mod.rs");
    std::fs::write(&mod_path, &mod_contents)
        .with_context(|| format!("Failed to write {}", mod_path.display()))?;
    println!("    → {}", mod_path.display());

    println!("Done! Add `mod generated;` to your lib.rs to use the generated code.");
    Ok(())
}

fn cmd_codegen_watch(config_path: &PathBuf) -> Result<()> {
    use notify::{Event, RecursiveMode, Watcher, recommended_watcher};
    use std::sync::mpsc;
    use std::time::{Duration, Instant};

    // Run once immediately.
    if let Err(e) = cmd_codegen(config_path) {
        eprintln!("codegen error: {:#}", e);
    }

    // Collect paths to watch from the config.
    let config_str = std::fs::read_to_string(config_path)
        .with_context(|| format!("Failed to read {}", config_path.display()))?;
    let config: GraphiteConfig = toml::from_str(&config_str)?;

    let (tx, rx) = mpsc::channel::<notify::Result<Event>>();
    let mut watcher = recommended_watcher(tx)?;

    // Watch config file itself.
    watcher.watch(config_path, RecursiveMode::NonRecursive)?;
    // Watch schema.
    if let Some(ref schema) = config.schema {
        if schema.exists() {
            watcher.watch(schema, RecursiveMode::NonRecursive)?;
        }
    }
    // Watch each ABI file.
    for c in config.contracts.iter().chain(config.templates.iter()) {
        if c.abi.exists() {
            watcher.watch(&c.abi, RecursiveMode::NonRecursive)?;
        }
    }

    println!("Watching... (Ctrl+C to stop)");

    let debounce = Duration::from_millis(300);
    let mut last_run = Instant::now() - debounce;

    loop {
        match rx.recv() {
            Ok(Ok(_event)) => {
                // Debounce: ignore events that arrive within 300 ms of the last run.
                if last_run.elapsed() < debounce {
                    continue;
                }
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default();
                println!("\n[{}.{:03}] Change detected — regenerating...", now.as_secs(), now.subsec_millis());
                if let Err(e) = cmd_codegen(config_path) {
                    eprintln!("codegen error: {:#}", e);
                }
                last_run = Instant::now();
            }
            Ok(Err(e)) => eprintln!("watch error: {}", e),
            Err(_) => break,
        }
    }
    Ok(())
}

fn cmd_manifest(config_path: &PathBuf, output_path: &PathBuf) -> Result<()> {
    manifest::generate(config_path, output_path)
}

fn placeholder_abi() -> String {
    r#"[
  {
    "anonymous": false,
    "inputs": [
      { "indexed": true, "name": "from", "type": "address" },
      { "indexed": true, "name": "to", "type": "address" },
      { "indexed": false, "name": "value", "type": "uint256" }
    ],
    "name": "Transfer",
    "type": "event"
  }
]"#
    .to_string()
}

fn cmd_build(release: bool) -> Result<()> {
    const TARGET: &str = "wasm32-unknown-unknown";

    // Build with cargo
    let mut cmd = std::process::Command::new("cargo");
    cmd.arg("build").arg("--target").arg(TARGET);

    if release {
        cmd.arg("--release");
    }

    println!(
        "  Running: cargo build --target {} {}",
        TARGET,
        if release { "--release" } else { "" }
    );

    let status = cmd.status().context("Failed to run cargo build")?;
    if !status.success() {
        anyhow::bail!("cargo build failed");
    }

    // Find the built wasm file
    let profile = if release { "release" } else { "debug" };

    // Get crate name from Cargo.toml
    let cargo_toml = std::fs::read_to_string("Cargo.toml").context("Failed to read Cargo.toml")?;
    let crate_name = cargo_toml
        .lines()
        .find(|l| l.starts_with("name"))
        .and_then(|l| l.split('=').nth(1))
        .map(|s| s.trim().trim_matches('"').replace('-', "_"))
        .context("Could not find crate name in Cargo.toml")?;

    // In a workspace, cargo places output in the workspace root's target/.
    // Walk up from cwd until we find a target/<profile> directory that contains
    // the wasm file, or fall back to local target/.
    let wasm_name = format!("{}.wasm", crate_name);
    let wasm_file = {
        let rel = PathBuf::from("target").join(TARGET).join(profile).join(&wasm_name);
        if rel.exists() {
            rel
        } else {
            // Try workspace root (up to 4 levels)
            let mut found = None;
            let mut dir = std::env::current_dir()?;
            for _ in 0..4 {
                if !dir.pop() { break; }
                let candidate = dir.join("target").join(TARGET).join(profile).join(&wasm_name);
                if candidate.exists() {
                    found = Some(candidate);
                    break;
                }
            }
            found.unwrap_or(rel)
        }
    };

    if !wasm_file.exists() {
        anyhow::bail!("WASM file not found: {}", wasm_file.display());
    }

    let size = std::fs::metadata(&wasm_file)?.len();
    println!(
        "  Built: {} ({:.1} KB)",
        wasm_file.display(),
        size as f64 / 1024.0
    );

    // Create build directory and copy
    let build_dir = PathBuf::from("build");
    std::fs::create_dir_all(&build_dir)?;

    let output_file = build_dir.join(format!("{}.wasm", crate_name));
    std::fs::copy(&wasm_file, &output_file)?;
    println!("  Copied to: {}", output_file.display());

    // Optimize with wasm-opt.
    // --enable-bulk-memory --llvm-memory-copy-fill-lowering is required:
    // Rust 1.87+ emits memory.copy/memory.fill bulk-memory opcodes (0xFC prefix)
    // that older graph-node versions reject. This flag lowers them to loops.
    let wasm_opt_result = std::process::Command::new("wasm-opt")
        .arg("--enable-bulk-memory")
        .arg("--llvm-memory-copy-fill-lowering")
        .arg("-Oz")
        .arg(&output_file)
        .arg("-o")
        .arg(&output_file)
        .status();

    match wasm_opt_result {
        Ok(s) if s.success() => {
            let optimized_size = std::fs::metadata(&output_file)?.len();
            println!(
                "  Optimized with wasm-opt: {:.1} KB → {:.1} KB",
                size as f64 / 1024.0,
                optimized_size as f64 / 1024.0
            );
        }
        Ok(_) => {
            anyhow::bail!("wasm-opt failed — check the output above");
        }
        Err(_) => {
            println!(
                "  WARNING: wasm-opt not found. Install binaryen for graph-node compatibility."
            );
            println!("  Without it, graph-node may reject the WASM (bulk-memory opcodes).");
        }
    }

    println!();
    println!("Build complete! WASM file: {}", output_file.display());

    Ok(())
}

fn cmd_test(args: &[String]) -> Result<()> {
    let extra = if args.is_empty() { String::new() } else { format!(" {}", args.join(" ")) };
    println!("  Running: cargo test{}", extra);
    let status = std::process::Command::new("cargo")
        .arg("test")
        .args(args)
        .status()?;

    if !status.success() {
        anyhow::bail!("Tests failed");
    }
    Ok(())
}

fn cmd_test_coverage(args: &[String]) -> Result<()> {
    println!("  Running: cargo llvm-cov --html {}", args.join(" "));
    println!("  (requires cargo-llvm-cov: cargo install cargo-llvm-cov)");

    let status = std::process::Command::new("cargo")
        .arg("llvm-cov")
        .arg("--html")
        .args(args)
        .status()
        .context("Failed to run cargo llvm-cov — is cargo-llvm-cov installed?")?;

    if !status.success() {
        anyhow::bail!("cargo llvm-cov failed");
    }

    println!();
    println!("Coverage report: target/llvm-cov/html/index.html");
    Ok(())
}

fn cmd_deploy(
    node: Option<&str>,
    ipfs: Option<&str>,
    name: &str,
    deploy_key: Option<&str>,
    version_label: Option<&str>,
) -> Result<()> {
    deploy::deploy(node, ipfs, name, deploy_key, version_label)
}
