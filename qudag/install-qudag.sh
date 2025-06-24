#!/bin/bash

# QuDAG CLI Installation Script
set -e

echo "🚀 Building QuDAG CLI..."
cargo build --bin qudag --release

echo "📦 Installing QuDAG CLI..."
if [ -d "$HOME/.local/bin" ] && [[ ":$PATH:" == *":$HOME/.local/bin:"* ]]; then
    cp target/release/qudag "$HOME/.local/bin/qudag"
    echo "✅ Installed to $HOME/.local/bin/qudag"
elif [ -w "/usr/local/bin" ]; then
    cp target/release/qudag /usr/local/bin/qudag
    echo "✅ Installed to /usr/local/bin/qudag"
else
    sudo cp target/release/qudag /usr/local/bin/qudag
    echo "✅ Installed to /usr/local/bin/qudag (with sudo)"
fi

echo "🔧 Testing installation..."
if command -v qudag &> /dev/null; then
    echo "✅ QuDAG CLI installed successfully!"
    echo ""
    echo "Try these commands:"
    echo "  qudag --help"
    echo "  qudag vault --help"
    echo "  qudag vault generate --length 16"
    echo "  qudag vault config show"
else
    echo "❌ Installation failed - qudag command not found in PATH"
    exit 1
fi