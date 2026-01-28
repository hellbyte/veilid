#!/bin/bash
TOOLCHAIN=${1:-nightly}
TARGET=$DEFAULT_CARGO_TARGET

if [ -z "$TARGET" ]; then
    TARGET=$(rustup default | cut -d- -f2- | cut -d' ' -f1)
fi

cargo test --doc --target $TARGET
cargo install cargo-docs-rs
cargo +$TOOLCHAIN docs-rs -p veilid-core --target $TARGET
cargo +$TOOLCHAIN docs-rs -p veilid-tools --target $TARGET
cargo +$TOOLCHAIN docs-rs -p veilid-remote-api --target $TARGET
