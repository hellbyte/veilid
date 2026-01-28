@echo off
set RUST_LOG=#common=debug
cargo nextest run
cargo nextest run --no-default-features --features=default-async-std 

