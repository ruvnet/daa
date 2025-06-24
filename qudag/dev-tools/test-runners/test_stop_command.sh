#!/bin/bash

# Test script for QuDAG CLI stop command functionality
# This script tests various scenarios for the stop command

echo "=============================================="
echo "QuDAG CLI Stop Command Test Suite"
echo "=============================================="

# Function to check if a process is running on a port
check_port() {
    local port=$1
    lsof -i :$port > /dev/null 2>&1
    return $?
}

# Function to wait for process to stop
wait_for_stop() {
    local port=$1
    local max_attempts=20
    local attempts=0
    
    while [ $attempts -lt $max_attempts ]; do
        if ! check_port $port; then
            return 0
        fi
        sleep 0.5
        attempts=$((attempts + 1))
    done
    return 1
}

# Function to cleanup any existing processes
cleanup() {
    echo "Cleaning up any existing processes..."
    pkill -f "qudag-cli.*start" 2>/dev/null || true
    sleep 2
}

# Test 1: Test stopping non-existent node
echo ""
echo "Test 1: Testing stop command on non-existent node"
echo "================================================="
cleanup
cargo run -p qudag-cli -- stop
echo "Expected: Should report no node running"
echo ""

# Test 2: Start a node and test normal stop
echo "Test 2: Start node and test normal stop command"
echo "==============================================="
echo "Starting node on port 8001..."
cargo run -p qudag-cli -- start --port 8001 &
NODE_PID=$!
sleep 3

# Check if node started
if check_port 8001; then
    echo "✓ Node started successfully on port 8001"
    
    # Test stop command
    echo "Sending stop command..."
    cargo run -p qudag-cli -- stop
    
    # Wait for node to stop
    if wait_for_stop 8001; then
        echo "✓ Node stopped successfully"
    else
        echo "✗ Node failed to stop within timeout"
        kill $NODE_PID 2>/dev/null || true
    fi
else
    echo "✗ Failed to start node on port 8001"
    kill $NODE_PID 2>/dev/null || true
fi
echo ""

# Test 3: Test multiple stop commands
echo "Test 3: Test multiple stop commands"
echo "===================================="
echo "Starting node on port 8002..."
cargo run -p qudag-cli -- start --port 8002 &
NODE_PID=$!
sleep 3

if check_port 8002; then
    echo "✓ Node started successfully on port 8002"
    
    echo "Sending first stop command..."
    cargo run -p qudag-cli -- stop &
    STOP_PID1=$!
    
    sleep 1
    
    echo "Sending second stop command..."
    cargo run -p qudag-cli -- stop &
    STOP_PID2=$!
    
    # Wait for both stop commands to complete
    wait $STOP_PID1
    wait $STOP_PID2
    
    if wait_for_stop 8002; then
        echo "✓ Node stopped successfully with multiple stop commands"
    else
        echo "✗ Node failed to stop with multiple stop commands"
        kill $NODE_PID 2>/dev/null || true
    fi
else
    echo "✗ Failed to start node on port 8002"
    kill $NODE_PID 2>/dev/null || true
fi
echo ""

# Test 4: Test process cleanup
echo "Test 4: Test process cleanup verification"
echo "========================================"
echo "Checking for any remaining QuDAG processes..."
REMAINING=$(ps aux | grep "qudag-cli.*start" | grep -v grep | wc -l)
if [ $REMAINING -eq 0 ]; then
    echo "✓ No remaining QuDAG processes found"
else
    echo "✗ Found $REMAINING remaining QuDAG processes"
    ps aux | grep "qudag-cli.*start" | grep -v grep
fi
echo ""

# Final cleanup
cleanup

echo "=============================================="
echo "Stop command test suite completed"
echo "=============================================="