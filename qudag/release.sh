#!/bin/bash
set -e

# QuDAG Release Script - Update version and republish
# This script handles version bumping and republishing to crates.io

echo "🚀 QuDAG Release Script"
echo "======================"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

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

# Get current version
CURRENT_VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
echo -e "${BLUE}Current version: $CURRENT_VERSION${NC}"

# Suggest new version
IFS='.' read -ra VERSION_PARTS <<< "$CURRENT_VERSION"
MAJOR=${VERSION_PARTS[0]}
MINOR=${VERSION_PARTS[1]}
PATCH=${VERSION_PARTS[2]}

SUGGESTED_PATCH="$MAJOR.$MINOR.$((PATCH + 1))"
SUGGESTED_MINOR="$MAJOR.$((MINOR + 1)).0"

echo ""
echo -e "${YELLOW}Version options:${NC}"
echo "1. Patch release: $SUGGESTED_PATCH (bug fixes, documentation)"
echo "2. Minor release: $SUGGESTED_MINOR (new features, backward compatible)"
echo "3. Custom version"

read -p "Choose version type (1/2/3): " version_choice

case $version_choice in
    1)
        NEW_VERSION=$SUGGESTED_PATCH
        ;;
    2)
        NEW_VERSION=$SUGGESTED_MINOR
        ;;
    3)
        read -p "Enter custom version (x.y.z): " NEW_VERSION
        ;;
    *)
        echo "❌ Invalid choice"
        exit 1
        ;;
esac

echo ""
echo -e "${GREEN}New version: $NEW_VERSION${NC}"

# Confirm version update
read -p "Update version to $NEW_VERSION? (y/N): " confirm_version
if [[ ! "$confirm_version" =~ ^[Yy]$ ]]; then
    echo "❌ Version update cancelled"
    exit 0
fi

echo ""
echo -e "${BLUE}📝 Updating version in workspace...${NC}"

# Update workspace version
sed -i.bak "s/version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" Cargo.toml
echo "✅ Updated Cargo.toml"

# Update version in main README if it exists
if grep -q "qudag = \"$CURRENT_VERSION\"" README.md; then
    sed -i.bak "s/qudag = \"$CURRENT_VERSION\"/qudag = \"$NEW_VERSION\"/" README.md
    echo "✅ Updated README.md"
fi

# Update individual crate READMEs
for readme in core/*/README.md tools/*/README.md qudag/README.md; do
    if [ -f "$readme" ] && grep -q "\"$CURRENT_VERSION\"" "$readme"; then
        sed -i.bak "s/\"$CURRENT_VERSION\"/\"$NEW_VERSION\"/" "$readme"
        echo "✅ Updated $readme"
    fi
done

echo ""
echo -e "${BLUE}🔧 Building and testing...${NC}"

# Build and test
if cargo check --workspace --no-default-features; then
    echo -e "${GREEN}✅ Build successful${NC}"
else
    echo -e "${RED}❌ Build failed${NC}"
    exit 1
fi

if cargo test --workspace --no-default-features; then
    echo -e "${GREEN}✅ Tests passed${NC}"
else
    echo -e "${RED}❌ Tests failed${NC}"
    read -p "Continue anyway? (y/N): " continue_anyway
    if [[ ! "$continue_anyway" =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

echo ""
echo -e "${YELLOW}📦 Ready to publish version $NEW_VERSION${NC}"
echo ""
echo "This will publish:"
echo "  1. qudag-crypto v$NEW_VERSION"
echo "  2. qudag-dag v$NEW_VERSION"
echo "  3. qudag-network v$NEW_VERSION"
echo "  4. qudag-protocol v$NEW_VERSION"
echo "  5. qudag v$NEW_VERSION"
echo "  6. qudag-cli v$NEW_VERSION"
echo ""

read -p "Proceed with publishing? (y/N): " confirm_publish
if [[ ! "$confirm_publish" =~ ^[Yy]$ ]]; then
    echo "❌ Publishing cancelled"
    exit 0
fi

echo ""
echo -e "${GREEN}🚀 Publishing to crates.io...${NC}"

# Function to publish a crate
publish_crate() {
    local crate_path=$1
    local crate_name=$2
    
    echo ""
    echo -e "${BLUE}📦 Publishing $crate_name v$NEW_VERSION...${NC}"
    
    cd "$crate_path"
    
    # Publish to crates.io
    if cargo publish --token "$CRATES_API_KEY" --no-default-features --allow-dirty; then
        echo -e "${GREEN}✅ Successfully published $crate_name v$NEW_VERSION${NC}"
        echo "  🌐 https://crates.io/crates/$crate_name"
    else
        echo -e "${RED}❌ Publish failed for $crate_name${NC}"
        cd - > /dev/null
        return 1
    fi
    
    # Wait between publishes
    if [ "$crate_name" != "qudag-cli" ]; then
        echo "  ⏳ Waiting 45 seconds for crates.io to update..."
        sleep 45
    fi
    
    cd - > /dev/null
}

# Publish in dependency order
publish_crate "core/crypto" "qudag-crypto" || exit 1
publish_crate "core/dag" "qudag-dag" || exit 1
publish_crate "core/network" "qudag-network" || exit 1
publish_crate "core/protocol" "qudag-protocol" || exit 1
publish_crate "qudag" "qudag" || exit 1
publish_crate "tools/cli" "qudag-cli" || exit 1

echo ""
echo -e "${GREEN}🎉 Release $NEW_VERSION published successfully!${NC}"
echo ""
echo -e "${BLUE}📚 Updated packages:${NC}"
echo "  • qudag v$NEW_VERSION - https://crates.io/crates/qudag"
echo "  • qudag-cli v$NEW_VERSION - https://crates.io/crates/qudag-cli"
echo "  • qudag-crypto v$NEW_VERSION - https://crates.io/crates/qudag-crypto"
echo "  • qudag-dag v$NEW_VERSION - https://crates.io/crates/qudag-dag"
echo "  • qudag-network v$NEW_VERSION - https://crates.io/crates/qudag-network"
echo "  • qudag-protocol v$NEW_VERSION - https://crates.io/crates/qudag-protocol"
echo ""
echo -e "${BLUE}🔄 Users can update with:${NC}"
echo "  cargo install qudag-cli --force  # Update CLI"
echo "  cargo update                     # Update library dependencies"
echo ""

# Create git tag
echo -e "${BLUE}🏷️  Creating git tag...${NC}"
if git tag "v$NEW_VERSION"; then
    echo -e "${GREEN}✅ Created tag v$NEW_VERSION${NC}"
    echo "Push tag with: git push origin v$NEW_VERSION"
else
    echo -e "${YELLOW}⚠️  Tag creation failed (tag may already exist)${NC}"
fi

echo ""
echo -e "${GREEN}✨ Release v$NEW_VERSION complete!${NC}"