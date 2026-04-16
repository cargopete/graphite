# File Data Source

**Source:** [`examples/file-ds/`](https://github.com/your-org/graphite/tree/main/examples/file-ds)

Demonstrates IPFS file data sources. An ERC721 contract emits a `URI` event containing an IPFS CID. The file handler fetches and parses the JSON metadata.

## What It Does

1. The ERC721 `URI` event contains a CID (e.g. `ipfs://QmXxx.../metadata.json`).
2. The event handler creates a stub `NFT` entity and triggers a file data source.
3. graph-node fetches the content from IPFS and calls the file handler with the raw bytes.
4. The file handler parses the JSON and populates `NFTMetadata`.

## Schema

```graphql
type NFT @entity {
  id: ID!
  tokenId: BigInt!
  contentURI: String!
  metadata: NFTMetadata
}

type NFTMetadata @entity {
  id: ID!
  name: String
  description: String
  image: String
}
```

## Event Handler

```rust
#[handler]
pub fn handle_uri(event: &ERC721URIEvent, _ctx: &graphite::EventContext) {
    let token_id = hex(&event.id);

    NFT::new(&token_id)
        .set_token_id(event.id.clone())
        .set_content_uri(event.value.clone())
        .save();

    // Trigger IPFS fetch — graph-node will call handle_nft_metadata when ready
    data_source::create_file("NFTMetadata", &event.value);
}
```

## File Handler

```rust
use graphite::json;

#[handler(file)]
pub fn handle_nft_metadata(content: &[u8], ctx: &graphite::FileContext) {
    let value = match json::from_bytes(content) {
        Some(v) => v,
        None => return,
    };

    NFTMetadata::new(&ctx.id)
        .set_name(json::get_string(&value, "name").unwrap_or_default())
        .set_description(json::get_string(&value, "description").unwrap_or_default())
        .set_image(json::get_string(&value, "image").unwrap_or_default())
        .save();
}
```

## graphite.toml

```toml
[[contracts]]
name        = "ERC721"
abi         = "abis/ERC721.json"
address     = "0x..."
start_block = 1000000

[[contracts.event_handlers]]
event   = "URI(string,uint256)"
handler = "handle_uri"

[[templates]]
name    = "NFTMetadata"
kind    = "file/ipfs"
handler = "handle_nft_metadata"
```

## Key Points

- File handlers receive raw bytes (`&[u8]`). Use `graphite::json::from_bytes` to parse JSON.
- `ctx.id` in the file handler contains a unique identifier for the data source instance — use it as the entity ID to link back to the originating entity.
- File handlers cannot be tested with `cargo test` in the same way as event handlers — the IPFS fetch is a graph-node operation. Test the JSON parsing logic separately if needed.
