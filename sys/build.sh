#!/bin/sh
set -e

cd "$(dirname "$0")"

# Build the sys crate into a local target/ directory so the C compiler can find
# the static library without chasing Cargo's hashed artifact names.
CARGO_TARGET_DIR=target cargo build

# Link the C example against the freshly built static library.
gcc example.c target/debug/deps/libratatui_relu_sys*.a -o example

./example
