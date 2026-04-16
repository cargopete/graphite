# File Handlers (IPFS)

File data sources allow you to fetch and index content from IPFS. A typical use case is an NFT contract that stores metadata CIDs on-chain — you index the `URI` event to get the CID, then the file handler parses the JSON metadata.

## Setup

Declare a file template in `graphite.toml`:

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

## Creating a File Data Source

In your event handler, trigger the file fetch by calling `data_source::create_file`:

```rust
#[handler]
pub fn handle_uri(event: &ERC721URIEvent, ctx: &graphite::EventContext) {
    // Store a pending NFT record
    let token_id = hex(&event.token_id);
    NFT::new(&token_id)
        .set_token_id(event.token_id.clone())
        .set_content_uri(event.uri.clone())
        .save();

    // Trigger the IPFS fetch
    data_source::create_file("NFTMetadata", &event.uri);
}
```

## Writing the File Handler

```rust
use graphite::json;

#[handler(file)]
pub fn handle_nft_metadata(content: &[u8], ctx: &graphite::FileContext) {
    // ctx.id — data source ID
    // ctx.address — address of the contract that created this data source

    let value = match json::from_bytes(content) {
        Some(v) => v,
        None => return,
    };

    let name        = json::get_string(&value, "name").unwrap_or_default();
    let description = json::get_string(&value, "description").unwrap_or_default();
    let image       = json::get_string(&value, "image").unwrap_or_default();

    NFTMetadata::new(&ctx.id)
        .set_name(name)
        .set_description(description)
        .set_image(image)
        .save();
}
```

## JSON Parsing

```rust
use graphite::json;

let value = json::from_bytes(content)?;

// Get typed fields
let name:    Option<String> = json::get_string(&value, "name");
let count:   Option<i64>    = json::get_i64(&value, "count");
let flag:    Option<bool>   = json::get_bool(&value, "active");
let nested:  Option<&JsonValue> = json::get_object(&value, "attributes");
```

## Direct IPFS Access

You can also call `ipfs.cat` directly inside a regular event handler:

```rust
use graphite::ipfs;

#[handler]
pub fn handle_uri(event: &ERC721URIEvent, ctx: &graphite::EventContext) {
    if let Some(content) = ipfs::cat(&event.uri) {
        // parse content immediately
    }
}
```

Note: `ipfs.cat` is synchronous and blocks the handler. For large amounts of IPFS content, the file data source template approach is preferred as it allows parallel fetching.

## ENS Resolution

```rust
use graphite::ens;

let name: Option<String> = ens::name_by_address(&address_bytes);
```

See the [file-ds example](../examples/file-ds.md) for a complete working subgraph.
