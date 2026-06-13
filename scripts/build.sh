#!/usr/bin/env bash
set -euo pipefail

echo "==> Building Release Binary..."
cargo build --release

echo "==> Copying Binary to dist/binaries..."
mkdir -p dist/binaries
cp target/release/disco dist/binaries/

echo "==> Build complete!"
