#!/bin/bash

# Test the actual Rust CLI by temporarily fixing the compilation issue

echo "=== Testing Actual QuDAG Rust CLI ==="
echo

cd /workspaces/QuDAG

# First, let's check if we can build with a workaround
echo "Attempting to build CLI with workaround..."

# Temporarily comment out the problematic RPC module
if [ -f "tools/cli/src/lib.rs" ]; then
    cp tools/cli/src/lib.rs tools/cli/src/lib.rs.backup
    sed -i 's/pub mod rpc;/\/\/ pub mod rpc;/' tools/cli/src/lib.rs
fi

# Try to build
cargo build --bin qudag 2>&1 | tail -20

# Check if build succeeded
if [ -f "target/debug/qudag" ]; then
    CLI="target/debug/qudag"
    echo "Build successful! Using: $CLI"
else
    echo "Build failed. Checking for other issues..."
    # Show the actual error
    cargo check --bin qudag 2>&1
    
    # Restore backup
    if [ -f "tools/cli/src/lib.rs.backup" ]; then
        mv tools/cli/src/lib.rs.backup tools/cli/src/lib.rs
    fi
    exit 1
fi

# Restore backup
if [ -f "tools/cli/src/lib.rs.backup" ]; then
    mv tools/cli/src/lib.rs.backup tools/cli/src/lib.rs
fi

echo
echo "=== Running Error Handling Tests on Actual CLI ==="
echo

# Function to test with timeout
test_with_timeout() {
    local description="$1"
    local command="$2"
    local expected_pattern="$3"
    
    echo "Test: $description"
    echo "Command: $command"
    
    # Run with timeout to prevent hanging
    timeout 2s bash -c "$command" 2>&1 | head -20
    exit_code=${PIPESTATUS[0]}
    
    if [ $exit_code -eq 124 ]; then
        echo "Command timed out (this might be expected for 'start' command)"
    fi
    echo "---"
    echo
}

# Test various error scenarios
test_with_timeout "No arguments" "$CLI" "error:"
test_with_timeout "Invalid command" "$CLI invalid" "error:"
test_with_timeout "Help flag" "$CLI --help" "QuDAG Protocol CLI"
test_with_timeout "Version flag" "$CLI --version" "qudag"
test_with_timeout "Start with invalid port" "$CLI start --port 999999" "error:"
test_with_timeout "Start with string port" "$CLI start --port abc" "error:"
test_with_timeout "Peer add without address" "$CLI peer add" "error:"
test_with_timeout "Address fingerprint without data" "$CLI address fingerprint" "error:"

# Test proper functionality (these might actually try to start services)
echo "=== Testing Valid Commands (with immediate interrupt) ==="
test_with_timeout "Start node (will timeout)" "$CLI start" "Starting QuDAG node"
test_with_timeout "Status command" "$CLI status" "node status"
test_with_timeout "Peer list" "$CLI peer list" "peers"

# Check for panic handling
echo "=== Testing Panic Scenarios ==="
echo "Sending various signals to running instance..."
$CLI start &
PID=$!
sleep 0.5

# Send different signals
for sig in TERM INT HUP; do
    echo "Sending SIG$sig to PID $PID"
    kill -$sig $PID 2>/dev/null
    sleep 0.2
done

# Clean up
kill -9 $PID 2>/dev/null
wait $PID 2>/dev/null

echo
echo "=== Checking Error Message Quality ==="

# Test a few commands and analyze error messages
echo "1. Testing required argument error:"
$CLI peer add 2>&1 | grep -E "(error|Error|required|missing)" || echo "No clear error message"

echo
echo "2. Testing invalid value error:"
$CLI start --port abc 2>&1 | grep -E "(error|Error|invalid|parse)" || echo "No clear error message"

echo
echo "3. Testing unknown command error:"
$CLI unknown 2>&1 | grep -E "(error|Error|unknown|unrecognized)" || echo "No clear error message"

echo
echo "=== Summary of Findings ==="
echo "CLI binary location: $CLI"
echo "Please check above output for:"
echo "- Proper error messages with clear guidance"
echo "- No panics or segfaults"
echo "- Graceful handling of invalid inputs"
echo "- Helpful error messages that guide users"