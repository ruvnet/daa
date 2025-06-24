#!/bin/bash
set -euo pipefail

# Script to update bootstrap peer configurations after deployment
# This resolves the chicken-and-egg problem of peer ID generation

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
CONFIGS_DIR="$PROJECT_ROOT/configs"
SECRETS_DIR="$PROJECT_ROOT/.secrets"

# Color codes
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

# Function to get peer ID from deployed node
get_peer_id() {
    local app_name=$1
    local peer_id=""
    
    # Try to get peer ID from running node
    if command -v flyctl &> /dev/null; then
        peer_id=$(flyctl ssh console -a "$app_name" -C "/usr/local/bin/qudag-node --show-peer-id" 2>/dev/null || echo "")
    fi
    
    # Fallback to generated peer ID from secrets
    if [ -z "$peer_id" ] && [ -f "$SECRETS_DIR/peer_ids/${app_name#qudag-testnet-}_peer_id.txt" ]; then
        peer_id=$(cat "$SECRETS_DIR/peer_ids/${app_name#qudag-testnet-}_peer_id.txt")
    fi
    
    echo "$peer_id"
}

# Function to update node configuration with actual bootstrap peers
update_node_config() {
    local node_name=$1
    local bootstrap_peer=$2
    local config_file="$CONFIGS_DIR/$node_name.toml"
    
    if [ -f "$config_file" ]; then
        # Create backup
        cp "$config_file" "$config_file.bak"
        
        # Update bootstrap peers
        sed -i "s|PEER_ID_PLACEHOLDER|$bootstrap_peer|g" "$config_file"
        
        log_success "Updated $node_name configuration with bootstrap peer"
    fi
}

# Main execution
main() {
    log_info "Updating bootstrap peer configurations..."
    
    # Get bootstrap node (node1) peer ID
    BOOTSTRAP_PEER_ID=$(get_peer_id "qudag-testnet-node1")
    
    if [ -z "$BOOTSTRAP_PEER_ID" ]; then
        log_info "Bootstrap peer ID not found, using generated ID"
        BOOTSTRAP_PEER_ID=$(cat "$SECRETS_DIR/peer_ids/node1_peer_id.txt" 2>/dev/null || echo "12D3KooWBootstrapNode")
    fi
    
    log_info "Bootstrap peer ID: $BOOTSTRAP_PEER_ID"
    
    # Update configurations for nodes 2-4
    for node in node2 node3 node4; do
        # For production deployment (Fly.io)
        update_node_config "$node" "$BOOTSTRAP_PEER_ID"
        
        # Update docker-compose.yml for local testing
        if [ -f "$PROJECT_ROOT/docker-compose.yml" ]; then
            sed -i "s|QmNode1PeerID|$BOOTSTRAP_PEER_ID|g" "$PROJECT_ROOT/docker-compose.yml"
        fi
    done
    
    # Generate updated bootstrap configuration
    cat > "$SECRETS_DIR/bootstrap_peers.json" <<EOF
{
  "production": {
    "bootstrap_peers": [
      "/dns4/qudag-testnet-node1.fly.dev/tcp/4001/p2p/$BOOTSTRAP_PEER_ID"
    ]
  },
  "local": {
    "bootstrap_peers": [
      "/ip4/172.20.0.10/tcp/4001/p2p/$BOOTSTRAP_PEER_ID"
    ]
  }
}
EOF
    
    log_success "Bootstrap peer configurations updated successfully"
}

main "$@"