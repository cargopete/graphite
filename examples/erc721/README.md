# ERC721 Subgraph Example

Indexes ERC721 `Transfer` and `Approval` events, tracking the current owner and approved address for every token.

Pre-configured for CryptoPunks (wrapped) on Ethereum mainnet. Change `address` and `startBlock` in `subgraph.yaml` for any ERC721 contract.

## What it demonstrates

- **Multiple event handlers** — `handle_transfer` and `handle_approval` in one mapping
- **Load-and-update pattern** — using `store_get` to load an existing `Token` entity before mutating it, rather than blindly overwriting it
- **ERC721 mint/burn semantics** — the zero address (`0x000...000`) is the canonical from-address for mints and to-address for burns; both are handled without special-casing
- **Approval clearing on transfer** — when a token transfers, the approved address is reset to the zero address, matching the ERC721 specification

## Entities

| Entity | Description |
|--------|-------------|
| `Token` | Current state of a token: `owner` and `approved` address |
| `Transfer` | Immutable record of each transfer event |
| `Approval` | Immutable record of each approval event |

## Build

```bash
# From the repo root
cargo build -p erc721-subgraph --target wasm32-unknown-unknown --release
```

## Test

```bash
cargo test -p erc721-subgraph
```

## Deploy

```bash
graphite deploy <your-node>/myname/erc721-subgraph
```

Requires a running graph-node fork with Rust ABI support ([PR #6462](https://github.com/graphprotocol/graph-node/pull/6462)).
