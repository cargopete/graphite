//! Graphite CLI — build tooling for Rust subgraphs.
//!
//! Commands:
//! - `graphite init` — scaffold a new subgraph project
//! - `graphite codegen` — generate Rust types from ABI + schema
//! - `graphite build` — compile to WASM
//! - `graphite test` — run tests (delegates to cargo test)
//! - `graphite deploy` — deploy to graph-node

mod codegen;
mod deploy;

use anyhow::{Context, Result};
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
    },

    /// Compile the subgraph to WASM
    Build {
        /// Build in release mode
        #[arg(long, default_value = "true")]
        release: bool,
    },

    /// Run tests
    Test {
        /// Additional arguments to pass to cargo test
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

        Commands::Codegen { config } => {
            let config_path = config.unwrap_or_else(|| PathBuf::from("graphite.toml"));
            println!("Generating types from ABI and schema...");
            cmd_codegen(&config_path)
        }

        Commands::Build { release } => {
            println!(
                "Building subgraph ({} mode)...",
                if release { "release" } else { "debug" }
            );
            cmd_build(release)
        }

        Commands::Test { args } => {
            println!("Running tests...");
            cmd_test(&args)
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

fn cmd_init(name: &str, _from_contract: Option<&str>, network: &str) -> Result<()> {
    let project_dir = PathBuf::from(name);

    // Check if directory already exists
    if project_dir.exists() {
        anyhow::bail!("Directory '{}' already exists", name);
    }

    // Create directory structure
    std::fs::create_dir_all(project_dir.join("src"))?;
    std::fs::create_dir_all(project_dir.join("abis"))?;

    // Create Cargo.toml
    let cargo_toml = format!(
        r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib"]

[dependencies]
graphite = {{ git = "https://github.com/cargopete/graphite.git" }}

[profile.release]
opt-level = "z"
lto = true
"#,
        name = name
    );
    std::fs::write(project_dir.join("Cargo.toml"), cargo_toml)?;
    println!("  Created Cargo.toml");

    // Create graphite.toml
    let graphite_toml = format!(
        r#"# Graphite subgraph configuration

output_dir = "src/generated"
schema = "schema.graphql"

# Add your contract ABIs here:
# [[contracts]]
# name = "MyContract"
# abi = "abis/MyContract.json"

# Dynamic data source templates (factory pattern):
# [[templates]]
# name = "Pair"
# abi = "abis/Pair.json"
"#
    );
    std::fs::write(project_dir.join("graphite.toml"), graphite_toml)?;
    println!("  Created graphite.toml");

    // Create subgraph.yaml (The Graph manifest)
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
      address: "0x0000000000000000000000000000000000000000"
      abi: {name}
      startBlock: 0
    mapping:
      kind: ethereum/events
      apiVersion: 0.0.6
      language: wasm/assemblyscript
      entities:
        - ExampleEntity
      abis:
        - name: {name}
          file: ./abis/{name}.json
      eventHandlers:
        - event: Transfer(indexed address,indexed address,uint256)
          handler: handleTransfer
      file: ./target/wasm32-unknown-unknown/release/{name_snake}.wasm
"#,
        name = name,
        network = network,
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

    // Create src/lib.rs
    let lib_rs = r#"//! Subgraph handlers

mod generated;

use graphite::prelude::*;
use generated::*;

// Example handler - replace with your own logic
#[handler]
pub fn handle_transfer(host: &mut impl HostFunctions, event: &ERC20TransferEvent) {
    // Load or create entity
    let id = event.id();
    let mut entity = ExampleEntity::load(host, &id)
        .unwrap_or_else(|| ExampleEntity::new(&id));

    // Update fields
    entity.sender = Bytes::from_slice(event.from.as_slice());
    entity.value = event.value.clone();

    // Save to store
    entity.save(host);
}
"#;
    std::fs::write(project_dir.join("src/lib.rs"), lib_rs)?;
    println!("  Created src/lib.rs");

    // Create placeholder ABI
    let placeholder_abi = r#"[
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
]"#;
    std::fs::write(
        project_dir.join(format!("abis/{}.json", name)),
        placeholder_abi,
    )?;
    println!("  Created abis/{}.json (placeholder)", name);

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
    println!("  # Update graphite.toml with your contract");
    println!("  # Edit schema.graphql with your entities");
    println!("  graphite codegen");
    println!("  cargo build --release --target wasm32-unknown-unknown");

    Ok(())
}

/// Configuration for codegen, read from graphite.toml
#[derive(Debug, serde::Deserialize)]
struct GraphiteConfig {
    /// Output directory for generated code
    #[serde(default = "default_output_dir")]
    output_dir: PathBuf,
    /// Path to GraphQL schema file
    schema: Option<PathBuf>,
    /// Data source contract definitions
    #[serde(default)]
    contracts: Vec<ContractConfig>,
    /// Dynamic data source template definitions — same ABI bindings as contracts,
    /// but listed under `templates:` in the subgraph manifest.
    #[serde(default)]
    templates: Vec<ContractConfig>,
}

fn default_output_dir() -> PathBuf {
    PathBuf::from("src/generated")
}

#[derive(Debug, serde::Deserialize)]
struct ContractConfig {
    /// Contract name (used for struct prefixes)
    name: String,
    /// Path to the ABI JSON file
    abi: PathBuf,
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

        // Write the generated file
        let filename = format!("{}.rs", contract.name.to_lowercase());
        let output_path = config.output_dir.join(&filename);
        std::fs::write(&output_path, &code)
            .with_context(|| format!("Failed to write {}", output_path.display()))?;

        // Add to mod.rs
        mod_contents.push_str(&format!(
            "mod {};\npub use {}::*;\n\n",
            contract.name.to_lowercase(),
            contract.name.to_lowercase()
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
    let target_dir = PathBuf::from("target").join(TARGET).join(profile);

    // Get crate name from Cargo.toml
    let cargo_toml = std::fs::read_to_string("Cargo.toml").context("Failed to read Cargo.toml")?;
    let crate_name = cargo_toml
        .lines()
        .find(|l| l.starts_with("name"))
        .and_then(|l| l.split('=').nth(1))
        .map(|s| s.trim().trim_matches('"').replace('-', "_"))
        .context("Could not find crate name in Cargo.toml")?;

    let wasm_file = target_dir.join(format!("{}.wasm", crate_name));

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
    // Delegate to cargo test
    let status = std::process::Command::new("cargo")
        .arg("test")
        .args(args)
        .status()?;

    if !status.success() {
        anyhow::bail!("Tests failed");
    }
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
