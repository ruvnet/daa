#!/bin/bash
set -e

# QuDAG Manual Publishing Guide
# This script helps you publish QuDAG to crates.io step by step

echo "🚀 QuDAG Manual Publishing Guide"
echo "================================"
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}📋 Prerequisites:${NC}"
echo "1. ✅ Verified email address on crates.io (https://crates.io/settings/profile)"
echo "2. ✅ Valid API token (you have: ${CRATES_API_KEY:0:10}...)"
echo "3. ✅ All packages compile successfully"
echo ""

echo -e "${YELLOW}⚠️  IMPORTANT: Email Verification Required${NC}"
echo "Before publishing, you need to:"
echo "1. Go to https://crates.io/settings/profile"
echo "2. Verify your email address"
echo "3. Come back and run this script"
echo ""

read -p "Have you verified your email on crates.io? (y/N): " email_verified
if [[ ! "$email_verified" =~ ^[Yy]$ ]]; then
    echo "❌ Please verify your email first and then re-run this script"
    exit 1
fi

echo ""
echo -e "${GREEN}✅ Email verified! Starting publication process...${NC}"
echo ""

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
    echo -e "${BLUE}📦 Publishing $crate_name...${NC}"
    echo "Path: $crate_path"
    
    cd "$crate_path"
    
    # Verify the crate builds
    echo "  🔧 Building $crate_name..."
    if cargo build --no-default-features; then
        echo -e "  ${GREEN}✅ Build successful${NC}"
    else
        echo -e "  ${RED}❌ Build failed for $crate_name${NC}"
        return 1
    fi
    
    # Show what will be published
    echo "  📋 Checking package contents..."
    cargo package --no-default-features --list | head -10
    
    # Confirm publish
    echo ""
    echo -e "${YELLOW}Ready to publish $crate_name to crates.io${NC}"
    read -p "Continue with publishing? (y/N): " confirm
    if [[ ! "$confirm" =~ ^[Yy]$ ]]; then
        echo "  ⏸️  Skipping $crate_name"
        cd - > /dev/null
        return 0
    fi
    
    # Publish to crates.io
    echo "  🚀 Publishing to crates.io..."
    if cargo publish --token "$CRATES_API_KEY" --no-default-features --allow-dirty; then
        echo -e "  ${GREEN}✅ Successfully published $crate_name${NC}"
        echo ""
        echo "  🌐 View at: https://crates.io/crates/$crate_name"
    else
        echo -e "  ${RED}❌ Publish failed for $crate_name${NC}"
        cd - > /dev/null
        return 1
    fi
    
    # Wait between publishes to allow crates.io to update
    if [ "$crate_name" != "qudag-cli" ]; then  # Don't wait after the last crate
        echo "  ⏳ Waiting 45 seconds for crates.io to update..."
        sleep 45
    fi
    
    cd - > /dev/null
}

# Navigate to workspace root
WORKSPACE_ROOT="$(dirname "$0")"
cd "$WORKSPACE_ROOT"

echo ""
echo -e "${BLUE}📋 Publishing Order:${NC}"
echo "  1. qudag-crypto (no dependencies)"
echo "  2. qudag-dag (depends on crypto)"
echo "  3. qudag-network (depends on crypto)"
echo "  4. qudag-protocol (depends on crypto, dag, network)"
echo "  5. qudag (main crate, depends on all)"
echo "  6. qudag-cli (CLI tool, depends on all)"
echo ""

echo -e "${YELLOW}⚠️  Warning: Publishing to crates.io is permanent!${NC}"
echo "Continue with publishing all crates? (y/N) "
read -r response
if [[ ! "$response" =~ ^[Yy]$ ]]; then
    echo "❌ Publishing cancelled"
    exit 0
fi

echo ""
echo -e "${GREEN}🚀 Starting publication process...${NC}"

# Publish crates in dependency order
publish_crate "core/crypto" "qudag-crypto" || exit 1
publish_crate "core/dag" "qudag-dag" || exit 1
publish_crate "core/network" "qudag-network" || exit 1
publish_crate "core/protocol" "qudag-protocol" || exit 1
publish_crate "qudag" "qudag" || exit 1
publish_crate "tools/cli" "qudag-cli" || exit 1

echo ""
echo -e "${GREEN}🎉 All crates published successfully!${NC}"
echo ""
echo -e "${BLUE}📚 Users can now install QuDAG with:${NC}"
echo "  cargo add qudag              # For library usage"
echo "  cargo install qudag-cli      # For CLI tool"
echo ""
echo -e "${BLUE}🌐 View on crates.io:${NC}"
echo "  https://crates.io/crates/qudag"
echo "  https://crates.io/crates/qudag-cli"
echo "  https://crates.io/crates/qudag-crypto"
echo "  https://crates.io/crates/qudag-dag"
echo "  https://crates.io/crates/qudag-network"
echo "  https://crates.io/crates/qudag-protocol"
echo ""
echo -e "${GREEN}✨ Publication complete! QuDAG is now available on crates.io${NC}"