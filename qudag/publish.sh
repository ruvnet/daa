#!/bin/bash
set -e

# QuDAG Crates.io Publishing Script
# This script publishes all QuDAG crates in the correct dependency order

echo "🚀 QuDAG Publishing Script"
echo "========================="

# Check if API key is set
if [ -z "$CRATES_API_KEY" ]; then
    if [ -f .env ]; then
        source .env
    fi
fi

if [ -z "$CRATES_API_KEY" ]; then
    echo "❌ Error: CRATES_API_KEY not set"
    echo "Please set CRATES_API_KEY environment variable or add it to .env file"
    exit 1
fi

# Function to publish a crate
publish_crate() {
    local crate_path=$1
    local crate_name=$2
    
    echo ""
    echo "📦 Publishing $crate_name..."
    echo "Path: $crate_path"
    
    cd "$crate_path"
    
    # Verify the crate builds
    echo "  ✓ Building $crate_name..."
    cargo build --no-default-features || {
        echo "  ❌ Build failed for $crate_name"
        return 1
    }
    
    # Publish to crates.io
    echo "  ✓ Publishing to crates.io..."
    cargo publish --token "$CRATES_API_KEY" --no-default-features --allow-dirty || {
        echo "  ❌ Publish failed for $crate_name"
        return 1
    }
    
    echo "  ✅ Successfully published $crate_name"
    
    # Wait between publishes to allow crates.io to update
    echo "  ⏳ Waiting 30 seconds for crates.io to update..."
    sleep 30
    
    cd - > /dev/null
}

# Function to check if crate exists on crates.io
check_crate_exists() {
    local crate_name=$1
    echo "  🔍 Checking if $crate_name exists on crates.io..."
    
    if curl -s "https://crates.io/api/v1/crates/$crate_name" | grep -q '"name"'; then
        echo "  ⚠️  Warning: $crate_name already exists on crates.io"
        echo "  Skip this crate? (y/N) "
        read -r response
        if [[ "$response" =~ ^[Yy]$ ]]; then
            return 0
        fi
    fi
    return 1
}

echo ""
echo "📋 Publishing Order:"
echo "  1. qudag-crypto (no dependencies)"
echo "  2. qudag-dag (depends on crypto)"
echo "  3. qudag-network (depends on crypto)"
echo "  4. qudag-protocol (depends on crypto, dag, network)"
echo "  5. qudag (main crate, depends on all)"
echo "  6. qudag-cli (CLI tool, depends on all)"
echo ""

echo "🔑 Using API key: ${CRATES_API_KEY:0:10}..."
echo ""

# Ask for confirmation
echo "⚠️  Warning: Publishing to crates.io is permanent!"
echo "Continue with publishing? (y/N) "
read -r response
if [[ ! "$response" =~ ^[Yy]$ ]]; then
    echo "❌ Publishing cancelled"
    exit 0
fi

# Navigate to workspace root
WORKSPACE_ROOT="$(dirname "$0")"
cd "$WORKSPACE_ROOT"

# Publish crates in dependency order
echo ""
echo "🚀 Starting publication process..."

# 1. Crypto crate
if ! check_crate_exists "qudag-crypto"; then
    publish_crate "core/crypto" "qudag-crypto" || exit 1
fi

# 2. DAG crate
if ! check_crate_exists "qudag-dag"; then
    publish_crate "core/dag" "qudag-dag" || exit 1
fi

# 3. Network crate
if ! check_crate_exists "qudag-network"; then
    publish_crate "core/network" "qudag-network" || exit 1
fi

# 4. Protocol crate
if ! check_crate_exists "qudag-protocol"; then
    publish_crate "core/protocol" "qudag-protocol" || exit 1
fi

# 5. Main QuDAG crate
if ! check_crate_exists "qudag"; then
    publish_crate "qudag" "qudag" || exit 1
fi

# 6. CLI crate
if ! check_crate_exists "qudag-cli"; then
    publish_crate "tools/cli" "qudag-cli" || exit 1
fi

echo ""
echo "✅ All crates published successfully!"
echo ""
echo "📚 Users can now install QuDAG with:"
echo "  cargo add qudag              # For library usage"
echo "  cargo install qudag-cli      # For CLI tool"
echo ""
echo "🌐 View on crates.io:"
echo "  https://crates.io/crates/qudag"
echo "  https://crates.io/crates/qudag-cli"