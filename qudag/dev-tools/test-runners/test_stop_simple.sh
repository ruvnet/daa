#!/bin/bash

echo "Testing QuDAG CLI stop command functionality"
echo "============================================"

# Function to check if a process is running on a port
check_port() {
    local port=$1
    netstat -tuln | grep ":$port " > /dev/null 2>&1
    return $?
}

# Function to wait for port to be open
wait_for_port() {
    local port=$1
    local max_attempts=20
    local attempts=0
    
    while [ $attempts -lt $max_attempts ]; do
        if check_port $port; then
            return 0
        fi
        sleep 0.5
        attempts=$((attempts + 1))
    done
    return 1
}

# Function to wait for port to be closed
wait_for_port_close() {
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

echo ""
echo "Test 1: Stop command on non-existent node"
echo "=========================================="
cargo run -p qudag-cli -- stop --port 9999
echo ""

echo "Test 2: Start node and test normal stop"
echo "========================================"
echo "Starting node on port 8001..."

# Start node in background
cargo run -p qudag-cli -- start --port 8001 &
NODE_PID=$!

# Wait for node to start
if wait_for_port 8001; then
    echo "✓ Node started successfully on port 8001"
    
    # Test stop command
    echo "Sending stop command..."
    cargo run -p qudag-cli -- stop --port 8001 &
    STOP_PID=$!
    
    # Wait for stop command to complete
    wait $STOP_PID
    
    # Wait for port to close
    if wait_for_port_close 8001; then
        echo "✓ Node stopped successfully"
    else
        echo "✗ Node failed to stop within timeout"
        kill $NODE_PID 2>/dev/null || true
    fi
    
    # Wait for background process to finish
    wait $NODE_PID 2>/dev/null || true
else
    echo "✗ Failed to start node on port 8001"
    kill $NODE_PID 2>/dev/null || true
fi

echo ""
echo "Test 3: Multiple stop commands"
echo "==============================="
echo "Starting node on port 8002..."

# Start node in background
cargo run -p qudag-cli -- start --port 8002 &
NODE_PID=$!

# Wait for node to start
if wait_for_port 8002; then
    echo "✓ Node started successfully on port 8002"
    
    # Send multiple stop commands
    echo "Sending first stop command..."
    cargo run -p qudag-cli -- stop --port 8002 &
    STOP_PID1=$!
    
    sleep 0.5
    
    echo "Sending second stop command..."
    cargo run -p qudag-cli -- stop --port 8002 &
    STOP_PID2=$!
    
    # Wait for both stop commands
    wait $STOP_PID1
    wait $STOP_PID2
    
    # Wait for port to close
    if wait_for_port_close 8002; then
        echo "✓ Node stopped successfully with multiple stop commands"
    else
        echo "✗ Node failed to stop with multiple commands"
        kill $NODE_PID 2>/dev/null || true
    fi
    
    # Wait for background process to finish
    wait $NODE_PID 2>/dev/null || true
else
    echo "✗ Failed to start node on port 8002"
    kill $NODE_PID 2>/dev/null || true
fi

echo ""
echo "Test 4: Process cleanup verification"
echo "===================================="
echo "Checking for any remaining QuDAG processes..."
REMAINING=$(ps aux | grep "qudag.*start" | grep -v grep | wc -l)
if [ $REMAINING -eq 0 ]; then
    echo "✓ No remaining QuDAG processes found"
else
    echo "✗ Found $REMAINING remaining QuDAG processes"
    ps aux | grep "qudag.*start" | grep -v grep
    # Cleanup any remaining processes
    pkill -f "qudag.*start" 2>/dev/null || true
fi

echo ""
echo "============================================"
echo "Stop command test completed"
echo "============================================"