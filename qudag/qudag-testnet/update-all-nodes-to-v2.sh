#!/bin/bash
# Update all QuDAG testnet nodes to MCP v2 with intro page

set -e

echo "Updating all QuDAG testnet nodes to MCP v2..."

# Build the Docker image if not already built
if ! docker images | grep -q "qudag-mcp-node-v2"; then
    echo "Building MCP v2 Docker image..."
    docker build -f Dockerfile.mcp-v2 -t qudag-mcp-node-v2:latest .
fi

# Deploy to each node
for i in 2 3 4; do
    NODE="qudag-testnet-node$i"
    echo "Deploying to $NODE..."
    
    # Tag the image
    docker tag qudag-mcp-node-v2:latest registry.fly.io/$NODE:deployment-mcp-v2
    
    # Push to registry
    docker push registry.fly.io/$NODE:deployment-mcp-v2
    
    # Deploy
    fly deploy --app $NODE --image registry.fly.io/$NODE:deployment-mcp-v2
    
    echo "$NODE updated successfully!"
    echo
done

echo "All nodes updated to MCP v2!"
echo
echo "Verifying deployments..."
for i in 1 2 3 4; do
    echo -n "Node $i: "
    curl -s https://qudag-testnet-node$i.fly.dev/health | jq -r '.version' 2>/dev/null || echo "No response"
done