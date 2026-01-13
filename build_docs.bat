@echo off
setlocal

set TOOLCHAIN=%1
if "%TOOLCHAIN%"=="" (
    set TOOLCHAIN=nightly
)

cargo install cargo-docs-rs
cargo +%TOOLCHAIN% docs-rs -p veilid-core
cargo +%TOOLCHAIN% docs-rs -p veilid-tools
cargo +%TOOLCHAIN% docs-rs -p veilid-remote-api

endlocal
