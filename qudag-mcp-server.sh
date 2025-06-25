#!/bin/bash
# QuDAG MCP Server Wrapper Script

echo "🚀 QuDAG MCP Server Wrapper Starting..."

# Check if cargo is available
if ! command -v cargo &> /dev/null; then
    echo "❌ Error: Cargo not found. Please install Rust."
    exit 1
fi

# Navigate to QuDAG MCP directory
cd /workspaces/daa/qudag/qudag-mcp

# Try to run the example directly
echo "📦 Running QuDAG MCP Server..."
cargo run --example basic_server -- --no-interactive 2>&1

# If that fails, provide helpful error message
if [ $? -ne 0 ]; then
    echo "❌ Failed to start QuDAG MCP Server"
    echo "💡 Try running: cd /workspaces/daa/qudag/qudag-mcp && cargo build"
    exit 1
fi