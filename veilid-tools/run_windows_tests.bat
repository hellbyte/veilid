@echo off
set RUST_LOG=#common=debug
cargo nextest run test --features=tracing
cargo nextest run test --no-default-features --features=rt-async-std,tracing
cargo nextest run test
cargo nextest run test --no-default-features --features=rt-async-std
cargo test --doc
