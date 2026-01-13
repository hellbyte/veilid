from typing import Annotated
from pathlib import Path
import os

import dagger
from dagger import Ignore, dag, function, object_type
import platform

VEILID_REPO = "registry.gitlab.com/veilid/veilid"
ZIG_VERSION = "0.13.0"
CMAKE_VERSION_MINOR = "4.0"
CMAKE_VERSION_PATCH = "4.0.2"
WASM_BINDGEN_CLI_VERSION = "0.2.106"
RUST_VERSION = "1.86.0"
RUST_UNIT_TESTS_NIGHTLY_VERSION="nightly-2026-01-01"
RUST_PACKAGE_TESTS_NIGHTLY_VERSION="nightly"
RETRY_COUNT = "12"

DIRECTORY_IGNORE_PATTERNS = [
    "__pycache__/",
    "__pypackages__/",
    "._*",
    ".accio/",
    ".apdisk",
    ".AppleDB",
    ".AppleDesktop",
    ".AppleDouble",
    ".atom/",
    ".build/",
    ".buildlog/",
    ".cache",
    ".com.apple.timemachine.donotpresent",
    ".config/",
    ".coverage",
    ".coverage.*",
    ".cxx",
    ".dart_tool/",
    ".dart/",
    ".dagger/"
    ".dmypy.json",
    ".DocumentRevisions-V100",
    ".DS_Store",
    ".eggs/",
    ".env",
    ".externalNativeBuild",
    ".flutter",
    ".flutter-plugins",
    ".flutter-plugins-dependencies",
    ".flutter-plugins-dependencies/",
    ".flutter-plugins/",
    ".fseventsd",
    ".generated/"
    ".git/",
    ".gradle",
    ".history",
    ".hypothesis/",
    ".idea",
    ".idea/",
    ".installed.cfg",
    ".ipynb_checkpoints",
    ".LSOverride",
    ".mypy_cache/",
    ".nox/",
    ".packages",
    ".pdm-build/",
    ".pdm-python",
    ".pdm.toml",
    ".pub-cache/",
    ".pub/",
    ".pybuilder/",
    ".pyre/",
    ".pytest_cache/",
    ".Python",
    ".python-version",
    ".pytype/",
    ".ropeproject",
    ".ruff_cache/",
    ".sconsign.dblite",
    ".scrapy",
    ".Spotlight-V100",
    ".spyderproject",
    ".spyproject",
    ".svn/",
    ".tags*",
    ".TemporaryItems",
    ".tmp-earthly-out/",
    ".tox/",
    ".Trashes",
    ".vagrant/",
    ".venv",
    ".VolumeIcon.icns",
    ".vscode/",
    ".vscode/*",
    ".webassets-cache",
    "[Dd]esktop.ini",
    "*.[Cc]ache",
    "*.cab",
    "*.class",
    "*.cover",
    "*.dSYM",
    "*.dSYM.zip",
    "*.egg",
    "*.egg-info/",
    "*.hmap",
    "*.iml",
    "*.ipa",
    "*.ipr",
    "*.iws",
    "*.lnk",
    "*.local",
    "*.log",
    "*.manifest",
    "*.mo",
    "*.mode1v3",
    "*.mode2v3",
    "*.moved-aside",
    "*.msi",
    "*.msix",
    "*.msm",
    "*.msp",
    "*.njsproj",
    "*.ntvs*",
    "*.pbxuser",
    "*.perspectivev3",
    "*.pot",
    "*.py,cover",
    "*.py[cod]",
    "*.pyc",
    "*.sage.py",
    "*.sln.docstates",
    "*.so",
    "*.spec",
    "*.stackdump",
    "*.suo",
    "*.sw?",
    "*.swp",
    "*.user",
    "*.userosscache",
    "*.xccheckout",
    "*.xcscmblueprint",
    "***/*.rs.bk",
    "**/__pycache__/",
    "**/.generated/",
    "**/.symlinks/",
    "**/.vagrant/",
    "**/*.jks",
    "**/*.keystore",
    "**/*.rs.bk",
    "**/*sync/",
    "**/DerivedData/",
    "**/dgph",
    "**/doc/api/",
    "**/Flutter/ephemeral/",
    "**/ios/Flutter/.last_build_id",
    "**/Pods/",
    "**/xcuserdata/",
    ".venv/",
    "*$py.class",
    "*sync/",
    "__pycache__",
    "android/app/debug",
    "android/app/profile",
    "android/app/release",
    "captures/",
    "dagger.gen.go",
    "~*",
    "$RECYCLE.BIN/",
    "app.*.map.json",
    "app.*.symbols",
    "bin/",
    "build/",
    "Carthage/Build/",
    "celerybeat-schedule",
    "celerybeat.pid",
    "cover/",
    "coverage.xml",
    "coverage/target",
    "cython_debug/",
    "db.sqlite3",
    "db.sqlite3-journal",
    "Dependencies/",
    "DerivedData/",
    "develop-eggs/",
    "dist-ssr",
    "dist/",
    "dmypy.json",
    "docs/_build/",
    "downloads/",
    "eggs/",
    "ehthumbs_vista.db",
    "ehthumbs.db",
    "env.bak/",
    "env/",
    "ENV/",
    "fastlane/Preview.html",
    "fastlane/report.xml",
    "fastlane/screenshots/**/*.png",
    "fastlane/test_output",
    "flamegraph.svg",
    "flutter/",
    "Flutter/",
    "GeneratedPluginRegistrant.h",
    "GeneratedPluginRegistrant.java",
    "GeneratedPluginRegistrant.m",
    "gradle-wrapper.jar",
    "htmlcov/",
    "Icon",
    "Icon?",
    "instance/",
    "iOSInjectionProject/",
    "ipython_config.py",
    "key.properties",
    "lerna-debug.log*",
    "lib/",
    "lib64/",
    "local_settings.py",
    "local.properties",
    "logs",
    "logs/",
    "MANIFEST",
    "Network",
    "Trash",
    "Folder",
    "node_modules",
    "nosetests.xml",
    "npm-debug.log*",
    "packages/",
    "parts/",
    "perf.data",
    "perf.data.old",
    "pip-delete-this-directory.txt",
    "pip-log.txt",
    "pkg/",
    "playground.xcworkspace",
    "pnpm-debug.log*",
    "profile",
    "profile_default/",
    "Runner/GeneratedPluginRegistrant.*",
    "sdist/",
    "ServiceDefinitions.json",
    "share/python-wheels/",
    "target/",
    "Temporary",
    "Items",
    "Thumbs.db",
    "Thumbs.db:encryptable",
    "timeline.xctimeline",
    "tmp/*",
    "var/",
    "venv.bak/",
    "venv/",
    "wasm-pack.log",
    "wheels/",
    "x64/",
    "x86/",
    "xcuserdata",
    "xcuserdata/",
    "yarn-debug.log*",
    "yarn-error.log*",
]

# Validate host architecture
arch = platform.machine()
if arch != "x86_64" and arch != "aarch64":
    raise ValueError(f"Unsupported host platform: {arch}")

@object_type
class Veilid:

    def _base_container(self) -> dagger.Container:
        """Creates the base container with all environment variables and initial setup"""

        return (
            dag.container()
            .from_("ubuntu:18.04")
            .with_env_variable("RUSTUP_HOME", "/usr/local/rustup")
            .with_env_variable("RUSTUP_DIST_SERVER", "https://static.rust-lang.org")
            .with_env_variable("CARGO_HOME", "/usr/local/cargo")
            .with_env_variable("PATH", "$PATH:/usr/local/cargo/bin:/usr/local/zig", expand=True)
            .with_env_variable("LD_LIBRARY_PATH", "/usr/local/lib")
            .with_env_variable("RUST_BACKTRACE", "1")
            .with_workdir("/veilid")
        )

    @function
    def deps_base(self) -> dagger.Container:
        """Install build prerequisites & setup required directories"""
        container = self._base_container()

        # Configure apt
        apt_config = f"""Acquire::Retries "{RETRY_COUNT}";
        Acquire::https::Timeout "240";
        Acquire::http::Timeout "240";
        APT::Get::Assume-Yes "true";
        APT::Install-Recommends "false";
        APT::Install-Suggests "false";
        Debug::Acquire::https "true";"""

        container = container.with_new_file("/etc/apt/apt.conf.d/99custom", apt_config)

        # Update package lists
        container = container.with_exec(["apt-get", "-y", "update"])

        # Install base packages including Windows cross-compilation tools
        base_packages = [
            "apt-get", "install", "-y",
            "ca-certificates", "iproute2", "curl", "build-essential",
            "libssl-dev", "openssl", "file", "git", "pkg-config",
            "libdbus-1-dev", "libdbus-glib-1-dev", "libgirepository1.0-dev",
            "libcairo2-dev", "checkinstall", "unzip", "zip", "libncursesw5-dev",
            "libncurses5-dev", "gcc-mingw-w64-x86-64", "mingw-w64"
        ]
        container = container.with_exec(base_packages)

        # Install cross-compilation toolchains and development libraries based on architecture
        arch = platform.machine()
        if arch == "x86_64":
            container = container.with_exec([
                "apt-get", "install", "-y",
                "gcc-aarch64-linux-gnu", "libc6-dev-arm64-cross"
            ])
        elif arch == "aarch64":
            container = container.with_exec([
                "apt-get", "install", "-y",
                "gcc-x86-64-linux-gnu", "libc6-dev-amd64-cross", "gcc-multilib-x86-64-linux-gnu"
            ])
        else:
            container = container.with_exec([
                "apt-get", "install", "-y",
                "gcc-aarch64-linux-gnu", "gcc-x86-64-linux-gnu",
                "libc6-dev-arm64-cross", "libc6-dev-amd64-cross", "gcc-multilib-x86-64-linux-gnu"
            ])

        # Install CMake
        arch = platform.machine()
        cmake_url = f"https://cmake.org/files/v{CMAKE_VERSION_MINOR}/cmake-{CMAKE_VERSION_PATCH}-linux-{arch}.sh"
        container = (
            container
            .with_exec(["curl", "--retry", RETRY_COUNT, "--retry-connrefused", "-O", cmake_url])
            .with_exec(["mkdir", "/opt/cmake"])
            .with_exec(["sh", f"cmake-{CMAKE_VERSION_PATCH}-linux-{arch}.sh", "--skip-license", "--prefix=/opt/cmake"])
            .with_exec(["ln", "-s", "/opt/cmake/bin/cmake", "/usr/local/bin/cmake"])
        )

        return container

    @function
    def deps_rust(self) -> dagger.Container:
        """Install Rust toolchain, targets, and cargo tools"""
        container = self.deps_base()

        # Install rustup and Rust toolchain
        container = container.with_exec([
            "sh", "-c",
            f"curl --retry {RETRY_COUNT} --retry-connrefused --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- --default-toolchain={RUST_VERSION} -y --no-modify-path"
        ])

        # Set permissions and verify installation
        container = container.with_exec([
            "chmod", "-R", "a+w", "/usr/local/rustup", "/usr/local/cargo"
        ]).with_exec([
            "rustup", "--version"
        ]).with_exec([
            "cargo", "--version"
        ]).with_exec([
            "rustc", "--version"
        ])

        # Install Rust targets with retry logic
        rust_targets = [
            "x86_64-unknown-linux-gnu",
            "aarch64-unknown-linux-gnu",
            "x86_64-pc-windows-gnu",
            # "aarch64-pc-windows-gnullvm",
            "aarch64-apple-darwin",
            "aarch64-linux-android",
            "armv7-linux-androideabi",
            "i686-linux-android",
            "x86_64-linux-android",
            "wasm32-unknown-unknown"
        ]

        # Add targets (simplified without complex retry logic for now)
        for target in rust_targets:
            container = container.with_exec(["rustup", "target", "add", target])

        # Add a single default-target nightly toolchain for some tests
        container = container.with_exec(["rustup", "toolchain", "install", f"nightly-{RUST_NIGHTLY_VERSION}"])

        # Install cargo tools
        container = container.with_exec([
            "cargo", "install", "wasm-pack", "wasm-opt", "--locked"
        ]).with_exec([
            "cargo", "install", "-f", "wasm-bindgen-cli", "--locked", "--version", WASM_BINDGEN_CLI_VERSION
        ]).with_exec([
            "cargo", "install", "cargo-chef", "--locked"
        ])

        # Install Zig for cross-compilation
        arch = platform.machine()
        zig_url = f"https://ziglang.org/download/{ZIG_VERSION}/zig-linux-{arch}-{ZIG_VERSION}.tar.xz"
        container = (
            container
            .with_exec(["curl", "--retry", RETRY_COUNT, "--retry-connrefused", "-O", zig_url])
            .with_exec(["tar", "-C", "/usr/local", "-xJf", f"zig-linux-{arch}-{ZIG_VERSION}.tar.xz"])
            .with_exec(["mv", f"/usr/local/zig-linux-{arch}-{ZIG_VERSION}", "/usr/local/zig"])
            .with_exec(["cargo", "install", "cargo-zigbuild"])
        )

        return container

    @function
    def deps_linux(self) -> dagger.Container:
        """Linux build dependencies (base + rust without Android tools)"""
        # In Dagger, we can simply call deps_rust which already includes everything needed
        # The Earthfile version copies artifacts, but in Dagger the container already has everything
        return self.deps_rust()

    @function
    def deps_cache(self, source: Annotated[dagger.Directory, Ignore(DIRECTORY_IGNORE_PATTERNS)]) -> dagger.Container:
        """Pre-compile Rust dependencies for faster builds using cargo-chef"""
        container = self.deps_linux()

        # Create project directory structure
        container = container.with_exec([
            "mkdir", "-p",
            "veilid-cli", "veilid-core", "veilid-core/examples/basic",
            "veilid-core/examples/private_route", "veilid-server", "veilid-tools",
            "veilid-wasm", "veilid-flutter", "veilid-flutter/rust", "veilid-remote-api"
        ])

        # Copy only Cargo.toml files and build configuration for dependency resolution
        container = (
            container
            .with_file(".cargo/config.toml", source.file(".cargo/config.toml"))
            .with_file("Cargo.lock", source.file("Cargo.lock"))
            .with_file("Cargo.toml", source.file("Cargo.toml"))
            .with_file("veilid-cli/Cargo.toml", source.file("veilid-cli/Cargo.toml"))
            .with_file("veilid-core/Cargo.toml", source.file("veilid-core/Cargo.toml"))
            .with_file("veilid-core/examples/basic/Cargo.toml", source.file("veilid-core/examples/basic/Cargo.toml"))
            .with_file("veilid-core/examples/private_route/Cargo.toml", source.file("veilid-core/examples/private_route/Cargo.toml"))
            .with_file("veilid-server/Cargo.toml", source.file("veilid-server/Cargo.toml"))
            .with_file("veilid-tools/Cargo.toml", source.file("veilid-tools/Cargo.toml"))
            .with_file("veilid-remote-api/Cargo.toml", source.file("veilid-remote-api/Cargo.toml"))
            .with_file("veilid-flutter/rust/Cargo.toml", source.file("veilid-flutter/rust/Cargo.toml"))
            .with_file("veilid-wasm/Cargo.toml", source.file("veilid-wasm/Cargo.toml"))
            .with_file("veilid-wasm/wasm_remap_paths.sh", source.file("veilid-wasm/wasm_remap_paths.sh"))
            .with_file("veilid-wasm/wasm_build_dart.sh", source.file("veilid-wasm/wasm_build_dart.sh"))
        )

        # Prepare cargo chef recipe based on dependencies
        container = container.with_exec([
            "cargo", "chef", "prepare", "--recipe-path", "recipe.json"
        ])

        # Cook dependencies for test profile using architecture-specific target
        arch = platform.machine()
        if arch == "x86_64":
            default_target = "x86_64-unknown-linux-gnu"
        elif arch == "aarch64":
            default_target = "aarch64-unknown-linux-gnu"
        else:
            default_target = "x86_64-unknown-linux-gnu"  # fallback

        container = container.with_exec([
            "cargo", "chef", "cook", "--profile=test", "--tests",
            "--target", default_target, "--recipe-path", "recipe.json",
            "-p", "veilid-server", "-p", "veilid-cli", "-p", "veilid-tools",
            "-p", "veilid-core", "-p", "veilid-remote-api"
        ])

        # Cook dependencies for release x86_64
        # Careful not to include veilid-flutter or veilid-wasm here as they add the 'json-camel-case' feature
        # which will screw up the build of veilid-server because it will automatically add that feature inappropriately
        container = container.with_exec([
            "cargo", "chef", "cook", "--zigbuild", "--release",
            "--target", "x86_64-unknown-linux-gnu", "--recipe-path", "recipe.json",
            "-p", "veilid-server", "-p", "veilid-cli", "-p", "veilid-tools",
            "-p", "veilid-core", "-p", "veilid-remote-api"
        ])

        # Cook dependencies for release aarch64
        # Careful not to include veilid-flutter or veilid-wasm here as they add the 'json-camel-case' feature
        # which will screw up the build of veilid-server because it will automatically add that feature inappropriately
        container = container.with_exec([
            "cargo", "chef", "cook", "--zigbuild", "--release",
            "--target", "aarch64-unknown-linux-gnu", "--recipe-path", "recipe.json",
            "-p", "veilid-server", "-p", "veilid-cli", "-p", "veilid-tools",
            "-p", "veilid-core", "-p", "veilid-remote-api"
        ])

        # Cook WASM dependencies with path remapping
        container = container.with_exec([
            "sh", "-c",
            "./veilid-wasm/wasm_remap_paths.sh cargo chef cook --zigbuild --release --target wasm32-unknown-unknown --recipe-path recipe.json -p veilid-wasm"
        ])

        return container

    @function
    def code_linux(self, source: Annotated[dagger.Directory, Ignore(DIRECTORY_IGNORE_PATTERNS)], base: str = "local", ci_registry_image: str = VEILID_REPO) -> dagger.Container:
        """Import the whole veilid code repository with build dependencies"""
        # Choose base container based on BASE parameter
        if base == "local":
            # Use local deps_cache (equivalent to +build-linux-cache)
            container = self.deps_cache(source)
        elif base == "uncached":
            # Use just deps-linux without cache
            container = self.deps_linux()
        else:
            # Use remote cache image from registry (equivalent to container mode)
            container = dag.container().from_(f"{ci_registry_image}/build-cache:latest")

        # Copy the full source code
        container = (
            container
            .with_directory("/veilid", source)
            .with_workdir("/veilid")
        )

        # Check to make sure Cargo.lock is up to date
        container = container.with_exec(["cargo", "update", "-w", "--locked"])

        # Restore original Cargo.lock (copy it again to ensure it's preserved)
        container = container.with_file("Cargo.lock", source.file("Cargo.lock"))

        return container

    @function
    def clippy(self, source: Annotated[dagger.Directory, Ignore(DIRECTORY_IGNORE_PATTERNS)], base: str = "local", ci_registry_image: str = VEILID_REPO) -> dagger.Container:
        """Run clippy linting for multiple targets"""
        container = self.code_linux(source, base, ci_registry_image)

        # Run clippy for different targets
        container = (
            container
            .with_exec(["cargo-zigbuild", "clippy", "--locked", "--target", "x86_64-unknown-linux-gnu"])
            .with_exec(["cargo-zigbuild", "clippy", "--locked", "--target", "x86_64-pc-windows-gnu"])
            .with_exec(["cargo-zigbuild", "clippy", "--locked", "--target", "aarch64-apple-darwin"])
            .with_exec(["cargo", "clippy", "--locked", "--manifest-path=veilid-wasm/Cargo.toml", "--target", "wasm32-unknown-unknown"])
        )

        return container

    @function
    def build_windows_amd64(self, source: Annotated[dagger.Directory, Ignore(DIRECTORY_IGNORE_PATTERNS)], base: str = "local", ci_registry_image: str = VEILID_REPO) -> dagger.Directory:
        """Build Windows AMD64 binaries and return the target directory"""
        container = self.code_linux(source, base, ci_registry_image)

        # Build release binaries for x86_64-pc-windows-gnu
        # Careful not to include veilid-flutter or veilid-wasm here as they add the 'json-camel-case' feature
        # which will screw up the build of veilid-server because it will automatically add that feature inappropriately
        container = container.with_exec([
            "cargo", "zigbuild", "--locked", "--target", "x86_64-pc-windows-gnu", "--release",
            "-p", "veilid-server", "-p", "veilid-cli", "-p", "veilid-tools",
            "-p", "veilid-core", "-p", "veilid-remote-api"
        ])

        # Return the built artifacts directory
        return container.directory("./target/x86_64-pc-windows-gnu")


    @function
    def build_linux_amd64(self, source: Annotated[dagger.Directory, Ignore(DIRECTORY_IGNORE_PATTERNS)], base: str = "local", ci_registry_image: str = VEILID_REPO) -> dagger.Directory:
        """Build Linux AMD64 binaries and return the target directory"""
        container = self.code_linux(source, base, ci_registry_image)

        # Build release binaries for x86_64-unknown-linux-gnu
        # Careful not to include veilid-flutter or veilid-wasm here as they add the 'json-camel-case' feature
        # which will screw up the build of veilid-server because it will automatically add that feature inappropriately
        container = container.with_exec([
            "cargo", "zigbuild", "--locked", "--target", "x86_64-unknown-linux-gnu", "--release",
            "-p", "veilid-server", "-p", "veilid-cli", "-p", "veilid-tools",
            "-p", "veilid-core", "-p", "veilid-remote-api"
        ])

        # Return the built artifacts directory
        return container.directory("./target/x86_64-unknown-linux-gnu")

    @function
    def build_linux_arm64(self, source: Annotated[dagger.Directory, Ignore(DIRECTORY_IGNORE_PATTERNS)], base: str = "local", ci_registry_image: str = VEILID_REPO) -> dagger.Directory:
        """Build Linux ARM64 binaries and return the target directory"""
        container = self.code_linux(source, base, ci_registry_image)

        # Build release binaries for aarch64-unknown-linux-gnu
        # Careful not to include veilid-flutter or veilid-wasm here as they add the 'json-camel-case' feature
        # which will screw up the build of veilid-server because it will automatically add that feature inappropriately
        container = container.with_exec([
            "cargo", "zigbuild", "--locked", "--target", "aarch64-unknown-linux-gnu", "--release",
            "-p", "veilid-server", "-p", "veilid-cli", "-p", "veilid-tools",
            "-p", "veilid-core", "-p", "veilid-remote-api"
        ])

        # Return the built artifacts directory
        return container.directory("./target/aarch64-unknown-linux-gnu")

    # No support yet. One could do this with a host-mount of the Apple Developer SDKs and running on a MacOS machine.
    # @function
    # def build_macos_arm64(self, source: Annotated[dagger.Directory, Ignore(DIRECTORY_IGNORE_PATTERNS)], base: str = "local", ci_registry_image: str = VEILID_REPO) -> dagger.Directory:
    #     """Build MacOS ARM64 binaries and return the target directory"""
    #     container = self.code_linux(source, base, ci_registry_image)

    #     # Build release binaries for aarch64-apple-darwin
    #     # Careful not to include veilid-flutter or veilid-wasm here as they add the 'json-camel-case' feature
    #     # which will screw up the build of veilid-server because it will automatically add that feature inappropriately
    #     container = container.with_exec([
    #         "cargo", "zigbuild", "--locked", "--target", "aarch64-apple-darwin", "--release",
    #         "-p", "veilid-server", "-p", "veilid-cli", "-p", "veilid-tools",
    #         "-p", "veilid-core", "-p", "veilid-remote-api"
    #     ]).terminal()

    #     # Return the built artifacts directory
    #     return container.directory("./target/aarch64-apple-darwin")


    @function
    async def test_native(self, source: Annotated[dagger.Directory, Ignore(DIRECTORY_IGNORE_PATTERNS)], base: str = "local", ci_registry_image: str = VEILID_REPO) -> str:
        """Run native unit tests"""
        container = self.code_linux(source, base, ci_registry_image)

        # Determine default cargo target based on architecture
        arch = platform.machine()
        if arch == "x86_64":
            default_target = "x86_64-unknown-linux-gnu"
        elif arch == "aarch64":
            default_target = "aarch64-unknown-linux-gnu"
        else:
            default_target = "x86_64-unknown-linux-gnu"  # fallback

        # Run native unit tests and capture output
        result = await container.with_exec([
            "cargo", "test", "--locked", "--tests", "--target", default_target,
            "-p", "veilid-server", "-p", "veilid-cli", "-p", "veilid-tools",
            "-p", "veilid-core", "-p", "veilid-remote-api"
        ]).stdout()

        return f"Native tests completed:\n{result}"

    @function
    async def test_docs(self, source: Annotated[dagger.Directory, Ignore(DIRECTORY_IGNORE_PATTERNS)], rust_nightly_version: str , base: str = "local", ci_registry_image: str = VEILID_REPO) -> str:
        """Build and test documentation"""
        container = self.code_linux(source, base, ci_registry_image)

        # Run documentation build and capture output
        result = await container.with_exec(["./build_docs.sh", rust_nightly_version]).stdout()

        return f"Documentation build completed:\n{result}"

    @function
    async def test_wasm(self, source: Annotated[dagger.Directory, Ignore(DIRECTORY_IGNORE_PATTERNS)], base: str = "local", ci_registry_image: str = VEILID_REPO) -> str:
        """Build WASM module (actual tests require network access)"""
        container = self.code_linux(source, base, ci_registry_image)

        # Build WASM release and capture output (tests require network access, so just build for now)
        result = await container.with_exec(["veilid-wasm/wasm_build_dart.sh", "release"]).stdout()

        return f"WASM build completed:\n{result}"

    @function
    async def test_all(self, source: Annotated[dagger.Directory, Ignore(DIRECTORY_IGNORE_PATTERNS)], base: str = "local", ci_registry_image: str = VEILID_REPO) -> str:
        """Run all tests: clippy, native tests, docs, and WASM build"""
        results = []

        # Run clippy (already includes multiple targets)
        clippy_container = self.clippy(source, base, ci_registry_image)
        clippy_result = await clippy_container.stdout()
        results.append(f"Clippy completed:\n{clippy_result}")

        # Run native tests
        native_result = await self.test_native(source, base, ci_registry_image)
        results.append(native_result)

        # Build docs
        docs_result = await self.test_docs(source, RUST_UNIT_TESTS_NIGHTLY_VERSION, base, ci_registry_image)
        results.append(docs_result)

        # Build WASM
        wasm_result = await self.test_wasm(source, base, ci_registry_image)
        results.append(wasm_result)

        return "\n\n=== TEST SUMMARY ===\n" + "\n\n".join(results)

    @function
    async def release_test_all(self, source: Annotated[dagger.Directory, Ignore(DIRECTORY_IGNORE_PATTERNS)], is_nightly: bool = False, base: str = "local", ci_registry_image: str = VEILID_REPO) -> str:
        """Tests to run before packaging releases"""
        results = []

        # Test DOCS.RS build with most recent nightly
        docs_result = await self.test_docs(source, RUST_PACKAGE_TESTS_NIGHTLY_VERSION, base, ci_registry_image)
        results.append(docs_result)

        return "\n\n=== TEST SUMMARY ===\n" + "\n\n".join(results)

    @function
    def package_deb(self, source: Annotated[dagger.Directory, Ignore(DIRECTORY_IGNORE_PATTERNS)], target_arch: str, is_nightly: bool = False, base: str = "local", ci_registry_image: str = VEILID_REPO) -> dagger.Directory:
        """Package DEB files for specified architecture"""
        # Get built binaries
        if target_arch == "amd64":
            build_dir = self.build_linux_amd64(source)
            rust_target = "x86_64-unknown-linux-gnu"
        elif target_arch == "arm64":
            build_dir = self.build_linux_arm64(source)
            rust_target = "aarch64-unknown-linux-gnu"
        else:
            raise ValueError(f"Unsupported architecture: {target_arch}")

        # Start with code-linux for the packaging scripts
        container = self.code_linux(source, base, ci_registry_image)

        # Copy build artifacts into container
        container = (
            container
            .with_directory("/veilid/package", source.directory("package"))
            .with_file(f"/veilid/target/{rust_target}/release/veilid-server", build_dir.file("release/veilid-server"))
            .with_file(f"/veilid/target/{rust_target}/release/veilid-cli", build_dir.file("release/veilid-cli"))
        )

        # Set nightly flag
        nightly_flag = "true" if is_nightly else "false"

        # Create DEB packages
        container = (
            container
            .with_exec([
                "/veilid/package/debian/earthly_make_veilid_server_deb.sh",
                target_arch, rust_target, nightly_flag
            ])
            .with_exec([
                "/veilid/package/debian/earthly_make_veilid_cli_deb.sh",
                target_arch, rust_target, nightly_flag
            ])
        )

        # Return the package output directory
        return container.directory("/dpkg/out")

    @function
    def package_rpm(self, source: Annotated[dagger.Directory, Ignore(DIRECTORY_IGNORE_PATTERNS)], target_arch: str, is_nightly: bool = False, base: str = "local", ci_registry_image: str = VEILID_REPO) -> dagger.Directory:
        """Package RPM files for specified architecture"""
        # Get built binaries
        if target_arch == "x86_64":
            build_dir = self.build_linux_amd64(source,base,ci_registry_image)
            rust_target = "x86_64-unknown-linux-gnu"
            platform = "linux/amd64"
        elif target_arch == "aarch64":
            build_dir = self.build_linux_arm64(source,base,ci_registry_image)
            rust_target = "aarch64-unknown-linux-gnu"
            platform = "linux/arm64"
        else:
            raise ValueError(f"Unsupported architecture: {target_arch}")

        # Use Rocky Linux for RPM packaging
        container = (
            dag.container(platform=dagger.Platform(platform))
            .from_("rockylinux:9")
            .with_exec(["yum", "install", "-y", "createrepo", "rpm-build", "rpm-sign", "yum-utils", "rpmdevtools"])
            .with_exec(["rpmdev-setuptree"])
        )

        # Set up directory structure
        container = (
            container
            .with_exec(["mkdir", "-p", "/veilid/target"])
            .with_exec(["mkdir", "-p", "/veilid/veilid-cli", "/veilid/veilid-server"])
            .with_exec(["mkdir", "-p", "/rpm-work-dir/veilid-server"])
        )

        # Copy necessary files
        container = (
            container
            .with_file("/veilid/veilid-cli/Cargo.toml", source.file("veilid-cli/Cargo.toml"))
            .with_file("/veilid/veilid-server/Cargo.toml", source.file("veilid-server/Cargo.toml"))
            .with_directory("/veilid/package", source.directory("package"))
            .with_file(f"/veilid/target/{rust_target}/release/veilid-server", build_dir.file("release/veilid-server"))
            .with_file(f"/veilid/target/{rust_target}/release/veilid-cli", build_dir.file("release/veilid-cli"))
        )

        # Set nightly flag
        nightly_flag = "true" if is_nightly else "false"

        # Create RPM packages
        container = (
            container
            .with_exec([
                "veilid/package/rpm/veilid-server/earthly_make_veilid_server_rpm.sh",
                target_arch, rust_target, nightly_flag
            ])
            .with_exec([
                "veilid/package/rpm/veilid-cli/earthly_make_veilid_cli_rpm.sh",
                target_arch, rust_target, nightly_flag
            ])
        )

        # Return the RPM output directory
        return container.directory(f"/root/rpmbuild/RPMS/{target_arch}")

    @function
    async def package_zip(self, source: Annotated[dagger.Directory, Ignore(DIRECTORY_IGNORE_PATTERNS)], target_arch: str, is_nightly: bool = False, base: str = "local", ci_registry_image: str = VEILID_REPO) -> dagger.Directory:
        """Package ZIP files for specified architecture"""
        # Get built binaries
        if target_arch == "amd64":
            build_dir = self.build_windows_amd64(source, base, ci_registry_image)
            rust_target = "x86_64-pc-windows-gnu"
        # elif target_arch == "arm64":
        #     build_dir = self.build_windows_arm64(source, base, ci_registry_image)
        #     rust_target = "aarch64-pc-windows-gnullvm"
        else:
            raise ValueError(f"Unsupported architecture: {target_arch}")

        # Start with code-linux for the packaging scripts
        container = self.code_linux(source, base, ci_registry_image)

        # Copy build artifacts into container
        container = container.with_directory(f"/veilid/target/{rust_target}", build_dir)

        # Copy package directory
        container = container.with_directory("/veilid/package", source.directory("package"))

        # Create ZIP packages
        container = (
            container
            .with_workdir(f"/veilid/target/{rust_target}/release")
            .with_exec(["mkdir","-p","/out"])
            .with_exec(["/veilid/package/cargo_version.sh", "/veilid/veilid-server/Cargo.toml"],redirect_stdout="/tmp/version")
            .with_exec(["sh","-c",r"date '+%Y%m%d'"],redirect_stdout="/tmp/datestamp")
        )
        version = (await container.file("/tmp/version").contents()).strip()
        datestamp = (await container.file("/tmp/datestamp").contents()).strip()

        if is_nightly:
            container = container.with_exec([
                "zip",
                f"/out/veilid-{datestamp}_{target_arch}.zip",
                "veilid-server.exe",
                "veilid-cli.exe"
            ])
        else:
            container = container.with_exec([
                "zip",
                f"/out/veilid-{version}_{target_arch}.zip",
                "veilid-server.exe",
                "veilid-cli.exe"
            ])

        # Return the package output directory
        return container.directory("/out")

    @function
    def package_linux_amd64(self, source: Annotated[dagger.Directory, Ignore(DIRECTORY_IGNORE_PATTERNS)], is_nightly: bool = False, base: str = "local", ci_registry_image: str = VEILID_REPO) -> dagger.Directory:
        """Package both DEB and RPM for Linux AMD64"""
        # Create a container to collect all packages
        container = dag.container().from_("alpine:latest").with_exec(["mkdir", "-p", "/packages"])

        # Get DEB packages
        deb_dir = self.package_deb(source, "amd64", is_nightly,base,ci_registry_image)
        container = container.with_directory("/packages/deb", deb_dir)

        # Get RPM packages
        rpm_dir = self.package_rpm(source, "x86_64", is_nightly,base,ci_registry_image)
        container = container.with_directory("/packages/rpm", rpm_dir)

        return container.directory("/packages")

    @function
    def package_linux_arm64(self, source: Annotated[dagger.Directory, Ignore(DIRECTORY_IGNORE_PATTERNS)], is_nightly: bool = False, base: str = "local", ci_registry_image: str = VEILID_REPO) -> dagger.Directory:
        """Package both DEB and RPM for Linux ARM64"""
        # Create a container to collect all packages
        container = dag.container().from_("alpine:latest").with_exec(["mkdir", "-p", "/packages"])

        # Get DEB packages
        deb_dir = self.package_deb(source, "arm64", is_nightly, base, ci_registry_image)
        container = container.with_directory("/packages/deb", deb_dir)

        # Get RPM packages
        rpm_dir = self.package_rpm(source, "aarch64", is_nightly, base, ci_registry_image)
        container = container.with_directory("/packages/rpm", rpm_dir)

        return container.directory("/packages")


    @function
    async def package_linux(self, source: Annotated[dagger.Directory, Ignore(DIRECTORY_IGNORE_PATTERNS)], is_nightly: bool = False, base: str = "local", ci_registry_image: str = VEILID_REPO) -> dagger.Directory:
        """Package for all Linux architectures (AMD64 and ARM64)"""

        # Create a container to collect all packages
        container = dag.container().from_("alpine:latest").with_exec(["mkdir", "-p", "/packages"])

        # Get AMD64 packages
        amd64_dir = self.package_linux_amd64(source, is_nightly, base, ci_registry_image)
        container = container.with_directory("/packages/amd64", amd64_dir)

        # Get ARM64 packages
        arm64_dir = self.package_linux_arm64(source, is_nightly, base, ci_registry_image)
        container = container.with_directory("/packages/arm64", arm64_dir)

        return container.directory("/packages")

    @function
    async def package_windows_amd64(self, source: Annotated[dagger.Directory, Ignore(DIRECTORY_IGNORE_PATTERNS)], is_nightly: bool = False, base: str = "local", ci_registry_image: str = VEILID_REPO) -> dagger.Directory:
        """Package ZIP for Windows AMD64"""
        # Create a container to collect all packages
        container = dag.container().from_("alpine:latest").with_exec(["mkdir", "-p", "/packages"])

        # Get Zip packages
        zip_dir = await self.package_zip(source, "amd64", is_nightly, base, ci_registry_image)
        container = container.with_directory("/packages/zip", zip_dir)

        return container.directory("/packages")

    @function
    async def package_windows(self, source: Annotated[dagger.Directory, Ignore(DIRECTORY_IGNORE_PATTERNS)], is_nightly: bool = False, base: str = "local", ci_registry_image: str = VEILID_REPO) -> dagger.Directory:
        """Package for all Windows architectures (AMD64)"""
        # Create a container to collect all packages
        container = dag.container().from_("alpine:latest").with_exec(["mkdir", "-p", "/packages"])

        # Get AMD64 packages
        amd64_dir = await self.package_windows_amd64(source, is_nightly, base, ci_registry_image)
        container = container.with_directory("/packages/amd64", amd64_dir)

        return container.directory("/packages")