#!/usr/bin/env bash
# Build a subgraph WASM and post-process with wasm-opt to strip bulk-memory opcodes.
# Graph-node's Wasmer runtime does not support the 0xFC bulk-memory extension (opcode 252).
# Rust 1.87+ emits bulk-memory by default and -C target-feature=-bulk-memory is ignored,
# so wasm-opt --llvm-memory-copy-fill-lowering is required as a post-processing step.
#
# Usage: ./build-wasm.sh <crate-name>
# Example: ./build-wasm.sh milestone

set -euo pipefail

CRATE="${1:-milestone}"
TARGET="wasm32-unknown-unknown"
RELEASE_DIR="target/${TARGET}/release"
INPUT="${RELEASE_DIR}/${CRATE}.wasm"
OUTPUT="${RELEASE_DIR}/${CRATE}.wasm"

echo "Building ${CRATE}..."
cargo build -p "${CRATE}" --target "${TARGET}" --release

if ! command -v wasm-opt &>/dev/null; then
    echo "WARNING: wasm-opt not found. Install binaryen to strip bulk-memory opcodes."
    echo "The WASM may fail to load in graph-node with 'Unknown opcode 252'."
    exit 0
fi

echo "Post-processing with wasm-opt (lowering bulk-memory to MVP)..."
TMPFILE=$(mktemp /tmp/wasm-opt-XXXXXX.wasm)
wasm-opt "${INPUT}" \
    --enable-bulk-memory \
    --llvm-memory-copy-fill-lowering \
    -Oz \
    -o "${TMPFILE}"
mv "${TMPFILE}" "${OUTPUT}"

echo "Done: ${OUTPUT}"
echo "Size: $(wc -c < "${OUTPUT}") bytes"

# Sanity check
if xxd "${OUTPUT}" | grep -q " fc "; then
    echo "WARNING: 0xFC opcodes still present - graph-node may reject this WASM."
else
    echo "OK: No bulk-memory opcodes (0xFC) detected. Ready for graph-node."
fi
