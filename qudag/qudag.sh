#!/bin/bash

# QuDAG Project Initialization Script
set -euo pipefail

echo "Initializing QuDAG development environment..."

# Check if Rust is installed
if ! command -v rustc &> /dev/null; then
    echo "Installing Rust toolchain..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi

# Update Rust toolchain
rustup update
rustup component add clippy rustfmt

# Create project directory structure
create_directory_structure() {
    local dirs=(
        "core/crypto"
        "core/dag"
        "core/network"
        "core/protocol"
        "tools/cli"
        "tools/simulator"
        "benchmarks"
        "tests/integration"
    )

    for dir in "${dirs[@]}"; do
        mkdir -p "$dir"
        # Create lib.rs for each core module
        if [[ "$dir" == core/* ]]; then
            module_name=$(basename "$dir")
            cat > "$dir/lib.rs" << EOF
//! QuDAG $module_name module
#![deny(unsafe_code)]
#![warn(missing_docs)]

/// Module version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
EOF
        fi
    done
}

# Initialize Cargo workspace
init_cargo_workspace() {
    cat > Cargo.toml << EOF
[workspace]
members = [
    "core/crypto",
    "core/dag",
    "core/network",
    "core/protocol",
    "tools/cli",
    "tools/simulator",
]
resolver = "2"

[workspace.dependencies]
thiserror = "1.0"
tracing = "0.1"
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
proptest = "1.0"

[workspace.package]
version = "0.1.0"
edition = "2021"
authors = ["rUv <https://github.com/ruvnet>"]
repository = "https://github.com/ruvnet/QuDAG"
license = "MIT OR Apache-2.0"
EOF

    # Create individual Cargo.toml files for each module
    for module in core/{crypto,dag,network,protocol} tools/{cli,simulator}; do
        module_name=$(basename "$module")
        cat > "$module/Cargo.toml" << EOF
[package]
name = "qudag-$module_name"
version.workspace = true
edition.workspace = true
authors.workspace = true
repository.workspace = true
license.workspace = true

[dependencies]
thiserror.workspace = true
tracing.workspace = true
serde.workspace = true

[dev-dependencies]
proptest.workspace = true
EOF
    done
}

# Set up testing infrastructure
setup_testing() {
    mkdir -p tests/integration
    cat > tests/integration/main.rs << EOF
//! QuDAG integration tests
#![deny(unsafe_code)]

#[cfg(test)]
mod tests {
    #[test]
    fn basic_integration() {
        assert!(true, "Basic integration test placeholder");
    }
}
EOF
}

echo "Creating directory structure..."
create_directory_structure

echo "Initializing Cargo workspace..."
init_cargo_workspace

echo "Setting up testing infrastructure..."
setup_testing

# Initialize git hooks for code quality
mkdir -p .git/hooks
cat > .git/hooks/pre-commit << EOF
#!/bin/bash
set -e

echo "Running pre-commit checks..."

# Format code
cargo fmt -- --check

# Run clippy
cargo clippy -- -D warnings

# Run tests
cargo test --all-features --workspace
EOF
chmod +x .git/hooks/pre-commit

echo "Running initial build..."
cargo build

echo "Running tests..."
cargo test --all-features --workspace

echo "
QuDAG development environment setup complete!

Next steps:
1. Review the project structure in the 'core' and 'tools' directories
2. Check test infrastructure in 'tests/integration'
3. Start development using TDD methodology
4. Run 'cargo test' to verify everything works

For more information, see the documentation in docs/
"