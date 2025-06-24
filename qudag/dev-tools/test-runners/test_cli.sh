#!/bin/bash

# QuDAG CLI Test Script
# Tests the CLI functionality without requiring full node startup

echo "QuDAG CLI Functionality Test"
echo "============================"

# Set the CLI binary path
CLI_BIN="cargo run -p qudag-cli --bin qudag --"

# Test 1: Show help
echo -e "\n1. Testing help command:"
$CLI_BIN --help

# Test 2: Test each main command help
echo -e "\n2. Testing start command help:"
$CLI_BIN start --help

echo -e "\n3. Testing peer command help:"
$CLI_BIN peer --help

echo -e "\n4. Testing network command help:"
$CLI_BIN network --help

echo -e "\n5. Testing address command help:"
$CLI_BIN address --help

# Test 3: Test subcommands
echo -e "\n6. Testing peer subcommands:"
$CLI_BIN peer list --help
$CLI_BIN peer add --help
$CLI_BIN peer remove --help

echo -e "\n7. Testing network subcommands:"
$CLI_BIN network stats --help
$CLI_BIN network test --help

echo -e "\n8. Testing address subcommands:"
$CLI_BIN address register --help
$CLI_BIN address resolve --help
$CLI_BIN address shadow --help
$CLI_BIN address fingerprint --help

echo -e "\n9. Testing CLI with invalid commands (error handling):"
$CLI_BIN invalid-command 2>&1 || echo "✓ Properly handled invalid command"

echo -e "\n10. Testing parameter validation:"
$CLI_BIN start --port 99999 2>&1 || echo "✓ Properly handled invalid port"

echo -e "\nCLI Structure Test Complete!"