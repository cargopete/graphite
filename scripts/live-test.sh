#!/usr/bin/env bash
set -euo pipefail

# Live integration test script for Graphite
#
# Prerequisites:
#   - Docker running with graphite-postgres and graphite-ipfs containers
#   - graph-node built at /Users/pepe/Projects/graph-node/target/release/graph-node
#   - WASM built at target/wasm32-unknown-unknown/release/erc20_subgraph.wasm

GRAPH_NODE_DIR="/Users/pepe/Projects/graph-node"
GRAPHITE_DIR="/Users/pepe/Projects/graphite"
EXAMPLE_DIR="${GRAPHITE_DIR}/examples/erc20"
WASM_FILE="${GRAPHITE_DIR}/target/wasm32-unknown-unknown/release/erc20_subgraph.wasm"

IPFS_API="http://localhost:5001"
GRAPH_NODE_ADMIN="http://localhost:8020"
GRAPH_NODE_QUERY="http://localhost:8000"
ETHEREUM_RPC="https://ethereum-rpc.publicnode.com"

SUBGRAPH_NAME="graphite/erc20-test"

echo "=== Graphite Live Integration Test ==="
echo ""

# Step 1: Verify prerequisites
echo "[1/6] Checking prerequisites..."

if ! docker ps | grep -q graphite-postgres; then
    echo "ERROR: graphite-postgres container not running"
    exit 1
fi

if ! docker ps | grep -q graphite-ipfs; then
    echo "ERROR: graphite-ipfs container not running"
    exit 1
fi

if [ ! -f "${GRAPH_NODE_DIR}/target/release/graph-node" ]; then
    echo "ERROR: graph-node binary not found. Build with: cd ${GRAPH_NODE_DIR} && cargo build --release"
    exit 1
fi

if [ ! -f "${WASM_FILE}" ]; then
    echo "ERROR: WASM not found. Build with: cargo build --target wasm32-unknown-unknown --release -p erc20-subgraph"
    exit 1
fi

echo "  All prerequisites met."

# Step 2: Upload files to IPFS
echo ""
echo "[2/6] Uploading files to IPFS..."

upload_to_ipfs() {
    local file="$1"
    local hash
    hash=$(curl -s -F "file=@${file}" "${IPFS_API}/api/v0/add" | python3 -c "import sys,json; print(json.load(sys.stdin)['Hash'])")
    echo "$hash"
}

WASM_HASH=$(upload_to_ipfs "${WASM_FILE}")
echo "  WASM:    ${WASM_HASH}"

SCHEMA_HASH=$(upload_to_ipfs "${EXAMPLE_DIR}/schema-live.graphql")
echo "  Schema:  ${SCHEMA_HASH}"

ABI_HASH=$(upload_to_ipfs "${EXAMPLE_DIR}/abis/ERC20.json")
echo "  ABI:     ${ABI_HASH}"

# Step 3: Create the subgraph manifest with IPFS references
echo ""
echo "[3/6] Creating subgraph manifest..."

# Build the manifest with IPFS paths
MANIFEST=$(cat <<YAML
specVersion: 0.0.5
schema:
  file:
    /: /ipfs/${SCHEMA_HASH}
dataSources:
  - kind: ethereum
    name: ERC20
    network: mainnet
    source:
      address: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"
      abi: ERC20
      startBlock: 24756400
    mapping:
      kind: wasm/rust
      apiVersion: 0.0.7
      language: rust
      entities:
        - Transfer
      abis:
        - name: ERC20
          file:
            /: /ipfs/${ABI_HASH}
      eventHandlers:
        - event: Transfer(indexed address,indexed address,uint256)
          handler: handle_transfer
      file:
        /: /ipfs/${WASM_HASH}
YAML
)

# Upload manifest to IPFS
MANIFEST_FILE=$(mktemp)
echo "${MANIFEST}" > "${MANIFEST_FILE}"
MANIFEST_HASH=$(upload_to_ipfs "${MANIFEST_FILE}")
rm "${MANIFEST_FILE}"
echo "  Manifest: ${MANIFEST_HASH}"

# Step 4: Start graph-node (if not running)
echo ""
echo "[4/6] Starting graph-node..."

if pgrep -f "graph-node" > /dev/null 2>&1; then
    echo "  graph-node already running, killing it..."
    pkill -f "graph-node" || true
    sleep 2
fi

GRAPH_NODE_BIN="${GRAPH_NODE_DIR}/target/release/graph-node"

# Start graph-node in background
RUST_LOG=info "${GRAPH_NODE_BIN}" \
    --postgres-url "postgresql://graph-node:let-me-in@localhost:5432/graph-node" \
    --ethereum-rpc "mainnet:${ETHEREUM_RPC}" \
    --ipfs "localhost:5001" \
    > /tmp/graph-node.log 2>&1 &

GRAPH_NODE_PID=$!
echo "  graph-node started (PID: ${GRAPH_NODE_PID})"
echo "  Logs: /tmp/graph-node.log"

# Wait for graph-node to be ready
echo "  Waiting for graph-node to start..."
for i in $(seq 1 30); do
    if curl -s "${GRAPH_NODE_ADMIN}" > /dev/null 2>&1; then
        echo "  graph-node ready!"
        break
    fi
    if [ $i -eq 30 ]; then
        echo "  ERROR: graph-node failed to start. Check /tmp/graph-node.log"
        tail -20 /tmp/graph-node.log
        exit 1
    fi
    sleep 1
done

# Step 5: Deploy the subgraph
echo ""
echo "[5/6] Deploying subgraph..."

# Create subgraph name
curl -s -X POST "${GRAPH_NODE_ADMIN}" \
    -H "Content-Type: application/json" \
    -d "{\"jsonrpc\":\"2.0\",\"method\":\"subgraph_create\",\"params\":{\"name\":\"${SUBGRAPH_NAME}\"},\"id\":1}" \
    | python3 -c "import sys,json; r=json.load(sys.stdin); print('  Created:', r.get('result', r.get('error', {}).get('message', 'unknown')))"

# Deploy version
curl -s -X POST "${GRAPH_NODE_ADMIN}" \
    -H "Content-Type: application/json" \
    -d "{\"jsonrpc\":\"2.0\",\"method\":\"subgraph_deploy\",\"params\":{\"name\":\"${SUBGRAPH_NAME}\",\"ipfs_hash\":\"${MANIFEST_HASH}\"},\"id\":2}" \
    | python3 -c "import sys,json; r=json.load(sys.stdin); print('  Deployed:', r.get('result', r.get('error', {}).get('message', 'unknown')))"

# Step 6: Monitor indexing
echo ""
echo "[6/6] Monitoring indexing progress..."
echo ""
echo "  Query endpoint: ${GRAPH_NODE_QUERY}/subgraphs/name/${SUBGRAPH_NAME}"
echo "  graph-node PID: ${GRAPH_NODE_PID}"
echo "  graph-node logs: tail -f /tmp/graph-node.log"
echo ""

# Poll for indexed entities
for i in $(seq 1 60); do
    sleep 5

    RESULT=$(curl -s "${GRAPH_NODE_QUERY}/subgraphs/name/${SUBGRAPH_NAME}" \
        -X POST \
        -H "Content-Type: application/json" \
        -d '{"query":"{ transfers(first: 3, orderBy: blockNumber, orderDirection: desc) { id from to value blockNumber } }"}' \
        2>/dev/null)

    if echo "${RESULT}" | python3 -c "import sys,json; d=json.load(sys.stdin); ts=d.get('data',{}).get('transfers',[]); print(f'  Block sync... Found {len(ts)} transfers'); [print(f'    [{t[\"blockNumber\"]}] {t[\"from\"][:10]}... → {t[\"to\"][:10]}... value={t[\"value\"]}') for t in ts[:3]]" 2>/dev/null; then
        TRANSFER_COUNT=$(echo "${RESULT}" | python3 -c "import sys,json; print(len(json.load(sys.stdin).get('data',{}).get('transfers',[])))" 2>/dev/null)
        if [ "${TRANSFER_COUNT}" -gt 0 ] 2>/dev/null; then
            echo ""
            echo "=== SUCCESS! Graphite subgraph is indexing Transfer events! ==="
            echo ""
            echo "Query your subgraph:"
            echo "  curl '${GRAPH_NODE_QUERY}/subgraphs/name/${SUBGRAPH_NAME}' \\"
            echo "    -X POST -H 'Content-Type: application/json' \\"
            echo "    -d '{\"query\":\"{ transfers(first: 5) { id from to value blockNumber } }\"}'"
            echo ""
            echo "To stop: kill ${GRAPH_NODE_PID} && docker stop graphite-postgres graphite-ipfs"
            exit 0
        fi
    else
        # Check if there's an error
        echo "${RESULT}" | head -c 200
    fi
done

echo ""
echo "Timed out waiting for indexed entities. Check logs:"
echo "  tail -f /tmp/graph-node.log"
echo "  PID: ${GRAPH_NODE_PID}"
