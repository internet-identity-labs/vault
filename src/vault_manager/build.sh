#!/usr/bin/env bash
set -euo pipefail

# Checking for dependencies
# XXX: we currently cannot check for the exact version of ic-cdk-optimizer
# because of https://github.com/dfinity/cdk-rs/issues/181
# Once the issue is fixed, we can ensure that the correct version is installed
if ! command -v ic-wasm; then
  echo could not find ic-wasm
  cargo install ic-wasm
fi

REPO_DIR="$(dirname "$0")"
TARGET="wasm32-unknown-unknown"

cargo_build_args=(
  --manifest-path "$REPO_DIR/Cargo.toml"
  --target "$TARGET"
  --release
  -j1
)

echo Running cargo build "${cargo_build_args[@]}"

cargo build "${cargo_build_args[@]}"

CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$REPO_DIR/../../target/}"

#ic-cdk-optimizer \
#  "$CARGO_TARGET_DIR/$TARGET/release/vault_manager.wasm" \
#  -o "$REPO_DIR/../../vault_manager.wasm"


echo "$CARGO_TARGET_DIR/$TARGET/release/vault_manager.wasm"
echo "$REPO_DIR/../../vault_manager.wasm"
ic-wasm "$CARGO_TARGET_DIR/$TARGET/release/vault_manager.wasm" -o "$REPO_DIR/../../vault_manager.wasm" shrink