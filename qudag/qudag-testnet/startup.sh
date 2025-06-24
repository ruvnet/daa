#!/bin/bash
# QuDAG startup script - selects binary based on NODE_TYPE environment variable

set -e

# Default to node type if not specified
NODE_TYPE="${NODE_TYPE:-node}"

# Function to handle signals
trap_handler() {
    echo "Received shutdown signal..."
    exit 0
}

# Set up signal handlers
trap trap_handler SIGTERM SIGINT

echo "Starting QuDAG with NODE_TYPE: $NODE_TYPE"

case "$NODE_TYPE" in
    "exchange")
        echo "Starting QuDAG Exchange Server..."
        # Exchange server might use different config path or parameters
        if [ -f "/data/qudag/exchange.toml" ]; then
            echo "Using exchange configuration from /data/qudag/exchange.toml"
            exec qudag-exchange-server --config /data/qudag/exchange.toml "$@"
        else
            exec qudag-exchange-server "$@"
        fi
        ;;
    "node"|"bootstrap"|*)
        echo "Starting QuDAG Node..."
        exec qudag-node "$@"
        ;;
esac