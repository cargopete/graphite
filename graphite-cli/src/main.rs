//! Graphite CLI — build tooling for Rust subgraphs.
//!
//! Commands:
//! - `graphite init` — scaffold a new subgraph project
//! - `graphite codegen` — generate Rust types from ABI + schema
//! - `graphite build` — compile to WASM
//! - `graphite test` — run tests (delegates to cargo test)
//! - `graphite deploy` — deploy to graph-node

use anyhow::Result;
use clap::{Parser, Subcommand};

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
    Codegen,

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

        Commands::Codegen => {
            println!("Generating types from ABI and schema...");
            cmd_codegen()
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

fn cmd_codegen() -> Result<()> {
    // TODO: Generate Rust types
    // - Parse subgraph.yaml for ABI paths and data sources
    // - Parse schema.graphql for entity definitions
    // - Generate generated/schema.rs with Entity derive structs
    // - Generate generated/<contract>.rs with event structs
    println!("  TODO: Code generation not yet implemented");
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
