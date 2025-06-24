#!/bin/bash

# Source cargo environment
source "$HOME/.cargo/env"

# Navigate to crypto directory
cd /workspaces/QuDAG/core/crypto

# Run tests with detailed output
RUST_BACKTRACE=1 cargo test --all-features -- --nocapture