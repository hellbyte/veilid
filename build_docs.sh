#!/bin/bash

cargo install cargo-docs-rs
cargo +nightly docs-rs -p veilid-core
cargo +nightly docs-rs -p veilid-tools
cargo +nightly docs-rs -p veilid-remote-api