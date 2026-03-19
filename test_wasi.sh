#!/usr/bin/env bash
set -euo pipefail

echo "=== Native build ==="
cargo check -p soroban-rs

echo ""
echo "=== Native tests ==="
cargo test -p soroban-rs

echo ""
echo "=== wasip2 build with tls-rustcrypto ==="
RUSTFLAGS="--cfg tokio_unstable" cargo +nightly build --target wasm32-wasip2 -p soroban-rs

echo ""
echo "All checks passed."
