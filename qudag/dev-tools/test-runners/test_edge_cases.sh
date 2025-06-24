#!/bin/bash

echo "Testing QuDAG CLI Stop Command Edge Cases"
echo "=========================================="

# Function to check if a process is running on a port
check_port() {
    local port=$1
    netstat -tuln | grep ":$port " > /dev/null 2>&1
    return $?
}

# Test 1: Different port configurations
echo ""
echo "Test 1: Different port configurations"
echo "===================================="

echo "Testing default port (8000)..."
cargo run -p qudag-cli -- stop
echo ""

echo "Testing high port number (65000)..."
cargo run -p qudag-cli -- stop --port 65000
echo ""

echo "Testing port 1 (privileged port)..."
cargo run -p qudag-cli -- stop --port 1
echo ""

# Test 2: Quick succession start/stop
echo "Test 2: Rapid start/stop cycle"
echo "==============================="
echo "Starting node on port 8003..."
cargo run -p qudag-cli -- start --port 8003 &
NODE_PID=$!
sleep 1

echo "Stopping immediately..."
cargo run -p qudag-cli -- stop --port 8003
sleep 1

# Cleanup
wait $NODE_PID 2>/dev/null || true
echo "✓ Rapid start/stop completed"
echo ""

# Test 3: Resource usage during stop
echo "Test 3: Testing stop command resource usage"
echo "============================================"
echo "Starting node on port 8004..."
cargo run -p qudag-cli -- start --port 8004 &
NODE_PID=$!
sleep 2

echo "Memory usage before stop:"
ps -o pid,vsz,rss,comm -p $NODE_PID 2>/dev/null || echo "Process not found"

echo "Sending stop command..."
time cargo run -p qudag-cli -- stop --port 8004

echo "Verifying process cleanup..."
sleep 1
ps -p $NODE_PID 2>/dev/null && echo "Process still running" || echo "✓ Process cleaned up"

# Cleanup
wait $NODE_PID 2>/dev/null || true
echo ""

# Test 4: Network connectivity during stop
echo "Test 4: Network connectivity test"
echo "=================================="
echo "Starting node on port 8005..."
cargo run -p qudag-cli -- start --port 8005 &
NODE_PID=$!
sleep 2

echo "Testing port accessibility before stop:"
nc -z localhost 8005 && echo "✓ Port 8005 accessible" || echo "✗ Port 8005 not accessible"

echo "Sending stop command..."
cargo run -p qudag-cli -- stop --port 8005 &
STOP_PID=$!

# Wait for stop to complete
wait $STOP_PID

echo "Testing port accessibility after stop:"
sleep 1
nc -z localhost 8005 && echo "✗ Port 8005 still accessible" || echo "✓ Port 8005 properly closed"

# Cleanup
wait $NODE_PID 2>/dev/null || true
echo ""

# Test 5: Invalid command scenarios
echo "Test 5: Invalid command scenarios"
echo "================================="

echo "Testing invalid port (0):"
cargo run -p qudag-cli -- stop --port 0 2>/dev/null || echo "✓ Handled invalid port correctly"

echo "Testing invalid port (70000):"
cargo run -p qudag-cli -- stop --port 70000 2>/dev/null || echo "✓ Handled out-of-range port correctly"

echo ""
echo "========================================"
echo "Edge case testing completed"
echo "========================================"