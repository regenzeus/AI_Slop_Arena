#!/usr/bin/env sh
# Lints, tests and regenerates the traceability table.
# Run from the repository root: sh tools/check.sh
set -eu
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
sh tools/trace.sh
