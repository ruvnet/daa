#!/bin/bash

# QuDAG NPM Package Publishing Script
# This script helps publish the QuDAG NPM package

set -e

echo "🌐 QuDAG NPM Package Publishing Script"
echo "======================================"
echo ""

# Check if we're in the right directory
if [ ! -f "package.json" ]; then
    echo "❌ Error: package.json not found. Please run this script from the qudag-npm directory."
    exit 1
fi

# Step 1: Install dependencies
echo "📦 Installing dependencies..."
npm install

# Step 2: Build TypeScript files
echo "🔨 Building TypeScript files..."
npm run build

# Step 3: Run tests
echo "🧪 Running tests..."
npm test || {
    echo "⚠️  Warning: Tests failed, but continuing..."
}

# Step 4: Check if logged in to npm
echo "🔐 Checking npm login status..."
npm whoami 2>/dev/null || {
    echo "❌ You are not logged in to npm."
    echo "Please run: npm login"
    exit 1
}

# Step 5: Dry run to check what will be published
echo "📋 Files that will be published:"
npm pack --dry-run

# Step 6: Confirm publication
echo ""
read -p "📤 Do you want to publish this package to npm? (y/N) " -n 1 -r
echo ""

if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "🚀 Publishing to npm..."
    npm publish --access public
    
    echo ""
    echo "✅ Package published successfully!"
    echo ""
    echo "🎉 Users can now install with:"
    echo "    npm install -g qudag"
    echo "    npm install qudag"
    echo ""
    echo "🚀 Or use directly with npx:"
    echo "    npx qudag@latest --help"
else
    echo "❌ Publication cancelled."
fi