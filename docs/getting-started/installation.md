# Installation

## Prerequisites

### Rust

Install Rust via [rustup](https://rustup.rs/), then add the WASM compilation target:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add wasm32-unknown-unknown
```

### wasm-opt

`graphite build` runs `wasm-opt -Oz` to shrink the binary. It's optional but recommended — a typical handler lands around 50–80 KB after optimisation.

```bash
# macOS
brew install binaryen

# cargo (any platform)
cargo install wasm-opt
```

### graphite-cli

```bash
cargo install graphite-cli
```

Verify:

```bash
graphite --version
```

## Optional: Etherscan API Key

`graphite init --from-contract` can fetch ABIs automatically from Etherscan (and compatible explorers). Set the key in your environment:

```bash
export ETHERSCAN_API_KEY=your_key_here
```

Supported chains include Ethereum mainnet/testnets, Arbitrum, Optimism, Base, Polygon, and any chain with an Etherscan-compatible explorer.

## Optional: Local graph-node

For local development without The Graph Studio, you need a running graph-node. The quickest path is the official Docker Compose setup from the [graph-node repository](https://github.com/graphprotocol/graph-node/tree/master/docker).

The Graph Studio works without any local infrastructure — just a deploy key.
