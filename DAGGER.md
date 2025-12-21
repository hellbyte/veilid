# Veilid Dagger Migration

This document details the migration from Earthly to Dagger for the Veilid project's build system.

## Quick Start

Recommended build configuration is:

* AMD64 or ARM64 machine
* Linux or MacOS
* At least 8GB of RAM
* At least 100GB of free disk space
* At least 4 CPU cores

First, ensure a container runtime is installed. Tested runtimes include:

* Docker
    - If you are using Docker, things should work out of the box with the above allocations on the host.
* Podman
    - If you are using Podman, you will need to configure it. See [instructions at the end of this document](#using-podman).

After your container runtime is avalable, [install dagger](https://docs.dagger.io/install/),

### Common Commands

All command execute from the root of the `veilid` repository:

```bash
# Install dependencies and run tests
dagger call test-all --source .

# Build for specific architectures
dagger call build-linux-amd-64 --source .
dagger call build-linux-arm-64 --source .

# Package for distribution
dagger call package-linux --source . export --path ./target/packages

# Run individual operations
dagger call clippy --source .
dagger call test-native --source .
```

## Function Reference

### Dependencies and Base Images

| Dagger Function | Earthly Target      | Description                                                                |
| --------------- | ------------------- | -------------------------------------------------------------------------- |
| `deps_base()`   | `deps-base`         | Install build prerequisites (Ubuntu 18.04, CMake, cross-compilation tools) |
| `deps_rust()`   | `deps-rust`         | Install Rust toolchain, targets, cargo tools, and Zig                      |
| `deps_linux()`  | `deps-linux`        | Linux build dependencies (equivalent to deps-rust in Dagger)               |
| `deps_cache()`  | `build-linux-cache` | Pre-compile Rust dependencies using cargo-chef for faster builds           |

### Source and Code Preparation

| Dagger Function | Earthly Target | Description                                                              |
| --------------- | -------------- | ------------------------------------------------------------------------ |
| `code_linux()`  | `code-linux`   | Import source code with build dependencies, supports multiple base modes |

**Base modes:**
- `local` (default): Uses local deps_cache 
- `uncached`: Uses deps_linux without cache
- `container`: Uses remote registry cache image

### Linting and Code Quality

| Dagger Function | Earthly Target | Description                                             |
| --------------- | -------------- | ------------------------------------------------------- |
| `clippy()`      | `clippy`       | Run clippy linting for Linux, Windows, and WASM targets |

**Note:** macOS target is commented out due to cross-compilation complexity.

### Building

| Dagger Function         | Earthly Target        | Description                               |
| ----------------------- | --------------------- | ----------------------------------------- |
| `build_linux_amd64()`   | `build-linux-amd64`   | Build release binaries for x86_64 Linux   |
| `build_linux_arm64()`   | `build-linux-arm64`   | Build release binaries for aarch64 Linux  |
| `build_windows_amd64()` | `build-windows-amd64` | Build release binaries for x86_64 Windows |

**Returns:** `dagger.Directory` containing built artifacts (not local files like Earthly's `SAVE ARTIFACT`)

### Testing

| Dagger Function | Earthly Target(s)         | Description                                     |
| --------------- | ------------------------- | ----------------------------------------------- |
| `test_native()` | `unit-tests-native-linux` | Run native unit tests for current architecture  |
| `test_docs()`   | `unit-tests-docs-linux`   | Build and test documentation                    |
| `test_wasm()`   | `unit-tests-wasm-linux`   | Build WASM module (network tests disabled)      |
| `test_all()`    | `unit-tests-linux`        | Run all test suites: clippy, native, docs, WASM |

**Note:** Individual clippy test targets (`unit-tests-clippy-*`) are consolidated into the main `clippy()` function.

### Packaging

| Dagger Function           | Earthly Target          | Description                                          |
| ------------------------- | ----------------------- | ---------------------------------------------------- |
| `package_deb()`           | `package-linux-*-deb`   | Create DEB packages for specified Linux architecture |
| `package_rpm()`           | `package-linux-*-rpm`   | Create RPM packages for specified Linux architecture |
| `package_linux_amd64()`   | `package-linux-amd64`   | Create both DEB and RPM packages for x86_64 Linux    |
| `package_linux_arm64()`   | `package-linux-arm64`   | Create both DEB and RPM packages for aarch64 Linux   |
| `package_linux()`         | `package-linux`         | Create packages for all Linux architectures          |
| `package_windows_amd64()` | `package-windows-amd64` | Create ZIP packages for Windows x86_64               |
| `package_windows()`       | `package-windows`       | Create packages for all Windows architectures        |

## Key Differences: Earthly vs Dagger

### Architecture & Patterns

| Aspect                | Earthly                      | Dagger                                         |
| --------------------- | ---------------------------- | ---------------------------------------------- |
| **File artifacts**    | `SAVE ARTIFACT ... AS LOCAL` | Return `dagger.Directory`, use `export --path` |
| **Parallelization**   | `WAIT` + `BUILD` blocks      | Automatic via Dagger's execution engine        |
| **Caching**           | Manual registry push/pull    | Built-in content-addressed caching             |
| **Cross-compilation** | Multiple toolchain packages  | Simplified with Zig (zigbuild)                 |
| **Container reuse**   | `FROM +target` references    | Function composition and reuse                 |

### Specific Changes

#### 1. **Artifact Handling**
```bash
# Earthly
SAVE ARTIFACT ./target/x86_64-unknown-linux-gnu AS LOCAL ./target/artifacts/x86_64-unknown-linux-gnu

# Dagger  
dagger call build-linux-amd-64 --source . export --path ./target/artifacts/x86_64-unknown-linux-gnu
```

#### 2. **Function Consolidation**
- **Earthly:** 7 separate `unit-tests-*` targets
- **Dagger:** 4 focused test functions (`test_native`, `test_docs`, `test_wasm`, `test_all`)

#### 3. **Cross-compilation Simplification**
- **Earthly:** Complex MinGW, cross-gcc, libc-dev packages
- **Dagger:** Primarily uses `cargo zigbuild` for cross-compilation

#### 4. **Output Visibility**
- **Earthly:** Build output visible by default
- **Dagger:** Test functions return `str` with captured output for visibility

## Functions Not Migrated

### Skipped Functions
| Earthly Target                  | Reason                              |
| ------------------------------- | ----------------------------------- |
| `deps-android`                  | Skipped for initial migration       |
| `code-android`                  | Skipped for initial migration       |
| `build-android`                 | Skipped for initial migration       |
| `build-macos-arm64`             | Commented out in original Earthfile |
| `unit-tests-clippy-macos-linux` | needs macOS cross-compilation       |

### Functions That Didn't Need Migration
- **Individual clippy targets:** Consolidated into single `clippy()` function
- **Wait/Build orchestration:** Handled automatically by Dagger's execution model

## Cross-Compilation Status

| Target                      | Status     | Notes                                      |
| --------------------------- | ---------- | ------------------------------------------ |
| `x86_64-unknown-linux-gnu`  | ✅ Working  | Uses cross-compilation toolchain           |
| `x86_64-pc-windows-gnu`     | ✅ Working  | Uses MinGW-w64                             |
| `aarch64-unknown-linux-gnu` | ✅ Working  | Uses cross-compilation toolchain           |
| `aarch64-apple-darwin`      | ❌ Disabled | Requires osxcross or similar complex setup |
| `wasm32-unknown-unknown`    | ✅ Working  | Native WASM support                        |

## Usage Examples

### Development Workflow
```bash
# Quick development check
dagger call clippy --source .

# Run all tests
dagger call test-all --source .

# Build for production
dagger call build-linux-amd-64 --source . export --path ./build/amd64
```

### CI/CD Pipeline
```bash
# Full pipeline: test, build, and package
dagger call test-all --source .
dagger call package-linux --source . export --path ./dist

# Nightly builds
dagger call package-linux --source . --is-nightly true export --path ./dist/nightly
```

### Cache Management
```bash
# Use uncached mode for clean builds
dagger call test-all --source . --base uncached

# Use container mode for CI (with remote cache)
dagger call test-all --source . --base container --ci-registry-image registry.example.com/veilid/veilid
```

### Using Podman
Podman requires configuring the podman machine used to build Veilid.

```bash
podman machine init -m 4096 --now podman-machine-default
podman machine ssh podman-machine-default sudo modprobe iptable_nat
podman machine ssh podman-machine-default sudo setenforce Permissive
```

A script in `.dagger/podman_setup.sh` exists to make this process easier as it may need to happen
whenever you start the podman machine. Build failures can result if these configurations are not met.

### Development on the Dagger build itself

```bash
# Install Dagger SDK
cd .dagger
dagger develop --sdk=python

# Create virtual environment
uv venv

# Install dependencies in virtual environment
uv sync
```

Now, point your development environment to the `.dagger/.venv/bin/python` interpreter

## Performance Benefits

1. **Better Caching:** Dagger's content-addressed caching is more efficient than Earthly's layer caching
2. **Parallel Execution:** Automatic parallelization without manual `WAIT`/`BUILD` blocks
3. **Simplified Cross-compilation:** Zig handles most cross-compilation complexity
4. **Artifact Management:** No need for manual registry management or local artifact copying

## Migration Notes

- All functions support the same `base` parameter for cache control
- Package functions now organize output in structured directories
- Test functions provide detailed output for better CI integration
- Cross-compilation is more reliable with Zig toolchain
- Memory checks removed (Dagger handles resource management)