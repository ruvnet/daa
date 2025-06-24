#!/bin/bash

# QuDAG Compatibility Testing Script
# Tests across different Rust versions, feature combinations, and target platforms

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test results tracking
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
    ((PASSED_TESTS++))
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
    ((FAILED_TESTS++))
}

run_test() {
    local test_name="$1"
    local test_command="$2"
    
    log_info "Running test: $test_name"
    ((TOTAL_TESTS++))
    
    if eval "$test_command" >/dev/null 2>&1; then
        log_success "$test_name"
        return 0
    else
        log_error "$test_name"
        return 1
    fi
}

# Test configuration
SUPPORTED_RUST_VERSIONS=(
    "1.70.0"    # MSRV candidate
    "1.75.0"    # Stable from 6 months ago
    "1.80.0"    # Recent stable
    "stable"    # Current stable
    "beta"      # Beta channel
    "nightly"   # Nightly (for experimental features)
)

TARGET_PLATFORMS=(
    "x86_64-unknown-linux-gnu"
    "x86_64-unknown-linux-musl"
    "x86_64-pc-windows-msvc"
    "x86_64-apple-darwin"
    "aarch64-unknown-linux-gnu"
    "wasm32-unknown-unknown"
)

# Feature flag combinations to test
FEATURE_COMBINATIONS=(
    ""                          # Default features
    "--no-default-features"     # Minimal features
    "--all-features"            # All features
)

# Workspace members for testing
WORKSPACE_MEMBERS=(
    "core/crypto"
    "core/dag"
    "core/network"
    "core/protocol"
    "tools/cli"
    "tools/simulator"
    "benchmarks"
)

print_header() {
    echo "=============================================="
    echo "QuDAG Protocol Compatibility Testing"
    echo "=============================================="
    echo "Current Rust version: $(rustc --version)"
    echo "Date: $(date)"
    echo "=============================================="
}

# Test current Rust version with different feature combinations
test_feature_combinations() {
    log_info "Testing feature combinations with current Rust version"
    
    for features in "${FEATURE_COMBINATIONS[@]}"; do
        for member in "${WORKSPACE_MEMBERS[@]}"; do
            if [ -d "$member" ]; then
                run_test "Features '$features' for $member" \
                    "cargo test -p $(basename $member) $features --quiet"
            fi
        done
    done
}

# Test build with different Rust versions (requires rustup)
test_rust_versions() {
    log_info "Testing compatibility with different Rust versions"
    
    if ! command -v rustup &> /dev/null; then
        log_warning "rustup not found, skipping Rust version tests"
        return
    fi
    
    # Save current toolchain
    local current_toolchain=$(rustup show active-toolchain | cut -d' ' -f1)
    
    for version in "${SUPPORTED_RUST_VERSIONS[@]}"; do
        log_info "Testing with Rust $version"
        
        # Install toolchain if needed
        if ! rustup toolchain list | grep -q "$version"; then
            log_info "Installing Rust $version"
            rustup toolchain install "$version" || {
                log_warning "Failed to install Rust $version, skipping"
                continue
            }
        fi
        
        # Switch to test version
        rustup default "$version" || {
            log_warning "Failed to switch to Rust $version, skipping"
            continue
        }
        
        # Test core functionality
        run_test "Build with Rust $version" "cargo build --workspace --quiet"
        run_test "Test with Rust $version" "cargo test --workspace --quiet"
        
        # Test crypto module specifically (most critical)
        run_test "Crypto module with Rust $version" "cargo test -p qudag-crypto --quiet"
    done
    
    # Restore original toolchain
    rustup default "$current_toolchain"
}

# Test cross-compilation for different targets
test_cross_compilation() {
    log_info "Testing cross-compilation for different target platforms"
    
    if ! command -v rustup &> /dev/null; then
        log_warning "rustup not found, skipping cross-compilation tests"
        return
    fi
    
    for target in "${TARGET_PLATFORMS[@]}"; do
        log_info "Testing cross-compilation for $target"
        
        # Add target if not already installed
        rustup target add "$target" 2>/dev/null || true
        
        # Test compilation for each workspace member
        for member in "${WORKSPACE_MEMBERS[@]}"; do
            if [ -d "$member" ]; then
                # Skip certain combinations that don't make sense
                if [[ "$target" == "wasm32-unknown-unknown" && "$member" == "tools/cli" ]]; then
                    continue
                fi
                
                run_test "Cross-compile $member for $target" \
                    "cargo check -p $(basename $member) --target $target --quiet"
            fi
        done
    done
}

# Test no_std compatibility where applicable
test_no_std_compatibility() {
    log_info "Testing no_std compatibility"
    
    # Core crypto should work in no_std environments
    if [ -d "core/crypto" ]; then
        # Create a temporary no_std test
        local temp_dir=$(mktemp -d)
        cat > "$temp_dir/Cargo.toml" << 'EOF'
[package]
name = "no_std_test"
version = "0.1.0"
edition = "2021"

[dependencies]
qudag-crypto = { path = "../core/crypto", default-features = false }

[lib]
name = "no_std_test"
path = "lib.rs"
EOF
        
        cat > "$temp_dir/lib.rs" << 'EOF'
#![no_std]

use qudag_crypto::HashFunction;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_std_hash() {
        let data = b"test data";
        let hash = HashFunction::blake3(data);
        assert!(!hash.is_empty());
    }
}
EOF
        
        cd "$temp_dir"
        run_test "no_std compatibility for crypto" "cargo test --quiet"
        cd - > /dev/null
        rm -rf "$temp_dir"
    fi
}

# Test with different dependency versions
test_dependency_compatibility() {
    log_info "Testing with minimal dependency versions"
    
    # Test with minimal versions to ensure compatibility
    run_test "Minimal dependency versions" "cargo +nightly build -Z minimal-versions --quiet" || {
        log_warning "Minimal versions test failed (requires nightly Rust)"
    }
}

# Test compilation in release mode
test_release_builds() {
    log_info "Testing release builds"
    
    for member in "${WORKSPACE_MEMBERS[@]}"; do
        if [ -d "$member" ]; then
            run_test "Release build for $member" \
                "cargo build -p $(basename $member) --release --quiet"
        fi
    done
}

# Test documentation generation
test_documentation() {
    log_info "Testing documentation generation"
    
    run_test "Documentation generation" "cargo doc --workspace --no-deps --quiet"
    run_test "Documentation with all features" "cargo doc --workspace --all-features --no-deps --quiet"
}

# Test benchmarks compilation
test_benchmarks() {
    log_info "Testing benchmark compilation"
    
    for member in "${WORKSPACE_MEMBERS[@]}"; do
        if [ -d "$member" ] && ls "$member"/benches/*.rs >/dev/null 2>&1; then
            run_test "Benchmark compilation for $member" \
                "cargo bench -p $(basename $member) --no-run --quiet"
        fi
    done
}

# Test with different optimization levels
test_optimization_levels() {
    log_info "Testing different optimization levels"
    
    local opt_levels=("0" "1" "2" "3" "s" "z")
    
    for level in "${opt_levels[@]}"; do
        RUSTFLAGS="-C opt-level=$level" run_test "Optimization level $level" \
            "cargo build -p qudag-crypto --quiet"
    done
}

# Test security-related compiler flags
test_security_flags() {
    log_info "Testing with security-hardened compiler flags"
    
    local security_flags=(
        "-C overflow-checks=on"
        "-C debug-assertions=on" 
        "-C panic=abort"
        "-F warnings"
    )
    
    for flag in "${security_flags[@]}"; do
        RUSTFLAGS="$flag" run_test "Security flag: $flag" \
            "cargo build -p qudag-crypto --quiet"
    done
}

# Main execution
main() {
    print_header
    
    # Ensure we're in the project root
    if [ ! -f "Cargo.toml" ]; then
        log_error "Must be run from project root directory"
        exit 1
    fi
    
    # Run all test suites
    test_feature_combinations
    test_rust_versions
    test_cross_compilation
    test_no_std_compatibility
    test_dependency_compatibility
    test_release_builds
    test_documentation
    test_benchmarks
    test_optimization_levels
    test_security_flags
    
    # Print summary
    echo "=============================================="
    echo "COMPATIBILITY TEST SUMMARY"
    echo "=============================================="
    echo "Total tests: $TOTAL_TESTS"
    echo -e "Passed: ${GREEN}$PASSED_TESTS${NC}"
    echo -e "Failed: ${RED}$FAILED_TESTS${NC}"
    
    if [ $FAILED_TESTS -eq 0 ]; then
        echo -e "${GREEN}All compatibility tests passed!${NC}"
        exit 0
    else
        echo -e "${RED}Some compatibility tests failed.${NC}"
        exit 1
    fi
}

# Run with specific test if provided
if [ $# -gt 0 ]; then
    case "$1" in
        "features") test_feature_combinations ;;
        "versions") test_rust_versions ;;
        "cross") test_cross_compilation ;;
        "nostd") test_no_std_compatibility ;;
        "deps") test_dependency_compatibility ;;
        "release") test_release_builds ;;
        "docs") test_documentation ;;
        "bench") test_benchmarks ;;
        "opt") test_optimization_levels ;;
        "security") test_security_flags ;;
        *) echo "Unknown test: $1"; exit 1 ;;
    esac
else
    main
fi