# Commands

## graphite init

Scaffold a new subgraph project.

```bash
graphite init <name> [OPTIONS]
```

| Flag | Description |
|------|-------------|
| `--network <network>` | Network name (e.g. `mainnet`, `arbitrum-one`). Required. |
| `--from-contract <address>` | Fetch ABI from Etherscan for this address. Requires `ETHERSCAN_API_KEY`. |

**Examples:**

```bash
# Minimal scaffold
graphite init my-subgraph --network mainnet

# Fetch ABI from Etherscan
ETHERSCAN_API_KEY=yourkey graphite init my-subgraph \
  --from-contract 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48 \
  --network mainnet
```

Generates: `Cargo.toml`, `graphite.toml`, `schema.graphql`, `abis/`, `src/lib.rs`.

---

## graphite codegen

Generate Rust types from ABIs and `schema.graphql`.

```bash
graphite codegen [OPTIONS]
```

| Flag | Description |
|------|-------------|
| `-c, --config <path>` | Path to `graphite.toml`. Defaults to `./graphite.toml`. |
| `-w, --watch` | Re-run codegen on file changes (watches ABIs and schema). |

**Output:** `src/generated/` (or the `output_dir` in `graphite.toml`).

```bash
graphite codegen          # one-shot
graphite codegen --watch  # live reload
```

---

## graphite manifest

Generate `subgraph.yaml` from `graphite.toml`.

```bash
graphite manifest [OPTIONS]
```

| Flag | Description |
|------|-------------|
| `-c, --config <path>` | Path to `graphite.toml`. Defaults to `./graphite.toml`. |
| `-o, --output <path>` | Output path. Defaults to `./subgraph.yaml`. |

```bash
graphite manifest
graphite manifest -o deploy/subgraph.yaml
```

---

## graphite build

Compile the subgraph to WASM.

```bash
graphite build [OPTIONS]
```

| Flag | Description |
|------|-------------|
| `--release` | Release build (default). |
| `-c, --config <path>` | Path to `graphite.toml`. |

Runs `cargo build --target wasm32-unknown-unknown --release`, then copies the WASM to `build/` and runs `wasm-opt -Oz` if available. Output: `build/<name>.wasm`.

```bash
graphite build
```

---

## graphite test

Run the subgraph's native tests.

```bash
graphite test [OPTIONS] [ARGS...]
```

| Flag | Description |
|------|-------------|
| `--coverage` | Run with coverage (requires `cargo-llvm-cov`). |
| extra args | Passed through to `cargo test`. |

```bash
graphite test
graphite test -- transfer_creates_entity  # run a specific test
graphite test -- --nocapture              # show println! output
```

---

## graphite deploy

Deploy the subgraph to a graph-node or The Graph Studio.

```bash
graphite deploy <name> [OPTIONS]
```

| Flag | Description |
|------|-------------|
| `--node <url>` | Graph-node deploy endpoint. Required. |
| `--ipfs <url>` | IPFS endpoint for uploading WASM and schema. Required. |
| `--deploy-key <key>` | Deploy key (required for The Graph Studio). |
| `--version-label <label>` | Version label, e.g. `v1.0.0` (required for Studio). |
| `-c, --config <path>` | Path to `graphite.toml`. |

**The Graph Studio:**

```bash
graphite deploy \
  --node https://api.studio.thegraph.com/deploy/ \
  --ipfs https://api.thegraph.com/ipfs/ \
  --deploy-key YOUR_DEPLOY_KEY \
  --version-label v1.0.0 \
  your-subgraph-slug
```

**Local graph-node:**

```bash
graphite deploy \
  --node http://localhost:8020 \
  --ipfs http://localhost:5001 \
  myname/my-subgraph
```

The CLI:
1. Builds the WASM if not already built.
2. Uploads the WASM, schema, and ABIs to IPFS.
3. Rewrites `subgraph.yaml` with IPFS content hashes.
4. Calls the graph-node `subgraph_deploy` JSON-RPC endpoint.
5. Prints the playground and query URLs on success.
