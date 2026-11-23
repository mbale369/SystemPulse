#!/bin/sh
# Vérification locale uniquement : aucune installation, publication ou action matérielle.
set -eu
cd "$(dirname "$0")/.."
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo check --workspace --locked

# Application desktop : compilation seule (workspace Cargo séparé).
if [ -f apps/desktop/src-tauri/Cargo.toml ]; then
    (cd apps/desktop/src-tauri && cargo fmt -- --check && cargo clippy --all-targets --locked -- -D warnings && cargo check --locked)
fi
