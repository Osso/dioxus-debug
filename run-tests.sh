#!/usr/bin/env bash
set -euo pipefail

cargo fmt --check
cargo clippy --features server -- -D warnings
cargo clippy --features client,script -- -D warnings
cargo check --example cli --features client,script
echo "All checks passed"
