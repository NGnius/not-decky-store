#!/bin/sh
set -e

echo "Container's IP address: `awk 'END{print $1}' /etc/hosts`"

cd /not-decky-store

echo "--- Rust version info ---"
rustup --version
rustc --version
cargo --version

echo "--- Building plugin backend ---"
cargo build --release
cp target/release/not-decky-store ./not-decky-store-docker

echo " --- Cleaning up ---"
# remove estranged target folder
cargo clean
