//! Graphite CLI — build tooling for Rust subgraphs.
//!
//! Commands:
//! - `graphite init` — scaffold a new subgraph project
//! - `graphite codegen` — generate Rust types from ABI + schema
//! - `graphite build` — compile to WASM
//! - `graphite test` — run tests (delegates to cargo test)
//! - `graphite deploy` — deploy to graph-node

mod codegen;

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

        Commands::Deploy { node, ipfs, name } => {
            println!("Deploying subgraph: {}", name);
            cmd_deploy(node.as_deref(), ipfs.as_deref(), &name)
        }
    }
}

fn cmd_init(_name: &str, _from_contract: Option<&str>, _network: &str) -> Result<()> {
    // TODO: Scaffold project structure
    // - Create Cargo.toml with graphite dependency
    // - Create subgraph.yaml manifest
    // - Create schema.graphql placeholder
    // - Create src/lib.rs with example handler
    // - If from_contract, fetch ABI and generate initial mappings
    println!("  TODO: Project scaffolding not yet implemented");
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
    /// Contract definitions
    #[serde(default)]
    contracts: Vec<ContractConfig>,
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
    std::fs::create_dir_all(&config.output_dir)
        .with_context(|| format!("Failed to create output dir: {}", config.output_dir.display()))?;

    // Generate mod.rs for the generated module
    let mut mod_contents = String::from("//! Generated code — do not edit.\n\n");

    // Generate schema entities if specified
    if let Some(ref schema_path) = config.schema {
        println!("  Generating entities from schema...");

        let code = codegen::generate_schema_entities(schema_path)
            .with_context(|| format!("Failed to generate entities from {}", schema_path.display()))?;

        let output_path = config.output_dir.join("schema.rs");
        std::fs::write(&output_path, &code)
            .with_context(|| format!("Failed to write {}", output_path.display()))?;

        mod_contents.push_str("mod schema;\npub use schema::*;\n\n");
        println!("    → {}", output_path.display());
    }

    // Generate contract bindings
    for contract in &config.contracts {
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
    // TODO: Build WASM
    // - Run cargo build --target wasm32-unknown-unknown
    // - Run wasm-opt for size optimization
    // - Validate exported functions match manifest
    // - Copy to build/ directory
    let mode = if release { "--release" } else { "" };
    println!(
        "  TODO: Would run: cargo build --target wasm32-unknown-unknown {}",
        mode
    );
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

fn cmd_deploy(_node: Option<&str>, _ipfs: Option<&str>, _name: &str) -> Result<()> {
    // TODO: Deploy subgraph
    // - Build if not already built
    // - Upload WASM and schema to IPFS
    // - Register with graph-node
    println!("  TODO: Deployment not yet implemented");
    Ok(())
}
