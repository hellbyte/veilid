#!/bin/bash

TOOLCHAIN=${1:-nightly}

cargo install cargo-docs-rs
cargo +$TOOLCHAIN docs-rs -p veilid-core
cargo +$TOOLCHAIN docs-rs -p veilid-tools
cargo +$TOOLCHAIN docs-rs -p veilid-remote-api