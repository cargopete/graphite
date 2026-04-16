# Examples

All examples are in the `examples/` directory of the repository and compile to WASM. Each has a full test suite runnable with `cargo test`.

| Example | What it demonstrates |
|---------|---------------------|
| [ERC20](erc20.md) | Basic event handler, single entity type. **Live on The Graph Studio (Arbitrum One).** |
| [ERC721](erc721.md) | Multiple event handlers, multiple entity types. **Live on The Graph Studio (Arbitrum One).** |
| [ERC1155](erc1155.md) | Three handlers: `TransferSingle`, `TransferBatch`, `URI`. Batch processing. |
| [Uniswap V2](uniswap-v2.md) | Factory + template pattern. Dynamic data sources. Counter updates. |
| [File Data Source](file-ds.md) | IPFS file handlers. JSON parsing. |
