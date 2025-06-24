# QuDAG MCP HTTPS Fix Guide

## Problem Summary

The HTTPS endpoint `https://qudag-testnet-node1.fly.dev/mcp` times out because:
1. The custom TCP socket implementation doesn't properly handle HTTP/2 requests from Fly.io's proxy
2. Missing proper HTTP headers and connection management
3. No keep-alive support for HTTP/1.1 connections

## Solution Options

### 1. Quick Fix: Update fly.toml Configuration (Recommended First Step)

Use the optimized fly.toml configuration that removes the dedicated MCP service and integrates everything through port 8080:

```bash
# Copy the optimized configuration
cp fly.toml.mcp-optimized nodes/fly.node1.toml

# Redeploy node1
fly deploy -a qudag-testnet-node1 --config nodes/fly.node1.toml
```

Key changes:
- Removed dedicated MCP service on port 3333
- Added HTTP/2 support with ALPN negotiation
- Increased timeouts to 30s
- Better connection handling

### 2. Deploy Enhanced MCP Node v3 (Best Long-term Solution)

The v3 node includes proper HTTP/2 support and enhanced request handling:

```bash
# Build the v3 Docker image
docker build -f Dockerfile.mcp-v3 -t qudag-mcp-node-v3:latest .

# Tag for Fly.io registry
docker tag qudag-mcp-node-v3:latest registry.fly.io/qudag-testnet-node1:deployment-mcp-v3

# Push to registry
docker push registry.fly.io/qudag-testnet-node1:deployment-mcp-v3

# Deploy with optimized fly.toml
fly deploy -a qudag-testnet-node1 \
  --image registry.fly.io/qudag-testnet-node1:deployment-mcp-v3 \
  --config fly.toml.mcp-optimized
```

Features:
- Proper HTTP/2 and HTTP/1.1 support
- Keep-alive connections
- Enhanced request parsing
- Better timeout handling
- Proper Content-Length headers

### 3. Create Dedicated MCP Service (Alternative Approach)

Deploy MCP as a separate Fly.io app:

```bash
# Create new app
fly launch --config fly.mcp-dedicated.toml --name qudag-mcp-service

# Build and deploy
fly deploy -a qudag-mcp-service --config fly.mcp-dedicated.toml
```

Access via: `https://qudag-mcp-service.fly.dev/mcp`

Benefits:
- Isolated from main node operations
- Can scale independently
- Easier to debug and monitor

### 4. Add Nginx Reverse Proxy (Zero Code Change)

If you prefer not to modify the Rust code:

```dockerfile
# Create Dockerfile.nginx-mcp
FROM qudag-mcp-node:latest as base

FROM nginx:alpine
COPY nginx-mcp-proxy.conf /etc/nginx/conf.d/default.conf
COPY --from=base /usr/local/bin/qudag-mcp-node /usr/local/bin/

# Start both nginx and the MCP node
CMD nginx && /usr/local/bin/qudag-mcp-node --config /data/qudag/config.toml
```

## Testing the Fix

After deploying any solution:

```bash
# Test HTTPS access
curl -v https://qudag-testnet-node1.fly.dev/mcp
curl -v https://qudag-testnet-node1.fly.dev/mcp/info
curl -v https://qudag-testnet-node1.fly.dev/mcp/tools

# Test with timeout
timeout 10 curl -s https://qudag-testnet-node1.fly.dev/mcp | jq '.'

# Monitor logs
fly logs -a qudag-testnet-node1 --tail
```

## Update MCP Configuration

Once HTTPS is working, update `.roo/mcp.json`:

```json
"qudag-testnet": {
  "command": "node",
  "args": [
    "/workspaces/QuDAG/.roo/mcp-http-proxy.js",
    "https://qudag-testnet-node1.fly.dev"
  ],
  // ... rest of config
}
```

## Recommended Deployment Order

1. **First**: Try the optimized fly.toml (no code changes needed)
2. **If that doesn't work**: Deploy v3 node with enhanced HTTP support
3. **Alternative**: Create dedicated MCP service
4. **Last resort**: Add nginx proxy layer

## Expected Results

After implementing the fix:
- ✅ `https://qudag-testnet-node1.fly.dev/` - Intro page works
- ✅ `https://qudag-testnet-node1.fly.dev/health` - Health check works
- ✅ `https://qudag-testnet-node1.fly.dev/mcp` - MCP discovery works
- ✅ `https://qudag-testnet-node1.fly.dev/mcp/tools` - Tools listing works
- ✅ All MCP endpoints accessible via HTTPS

## Monitoring

```bash
# Check deployment status
fly status -a qudag-testnet-node1

# Monitor HTTP/2 connections
fly ssh console -a qudag-testnet-node1
netstat -an | grep :8080

# Check MCP endpoint internally
curl -v http://localhost:8080/mcp
```