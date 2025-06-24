#!/bin/bash
set -euo pipefail

# QuDAG Testnet Secrets Setup Script
# This script manages node keys and secrets for the QuDAG testnet

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
SECRETS_DIR="$PROJECT_ROOT/.secrets"
ENV_FILE="$PROJECT_ROOT/.env"

# Node apps
declare -a NODES=("qudag-testnet-node1" "qudag-testnet-node2" "qudag-testnet-node3" "qudag-testnet-node4")

# Functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

check_env_file() {
    if [ ! -f "$ENV_FILE" ]; then
        log_warning ".env file not found. Creating from .env.example..."
        if [ -f "$PROJECT_ROOT/.env.example" ]; then
            cp "$PROJECT_ROOT/.env.example" "$ENV_FILE"
            log_info "Created .env file. Please update it with your values."
            log_error "Please edit $ENV_FILE and run this script again."
            exit 1
        else
            log_error ".env.example not found!"
            exit 1
        fi
    fi
}

create_secrets_dir() {
    if [ ! -d "$SECRETS_DIR" ]; then
        log_info "Creating secrets directory..."
        mkdir -p "$SECRETS_DIR"
        chmod 700 "$SECRETS_DIR"
        
        # Add to .gitignore if not already there
        if [ -f "$PROJECT_ROOT/.gitignore" ] && ! grep -q "^.secrets" "$PROJECT_ROOT/.gitignore"; then
            echo ".secrets/" >> "$PROJECT_ROOT/.gitignore"
        fi
    fi
}

generate_node_key() {
    local node_name=$1
    local key_file="$SECRETS_DIR/${node_name}_key.json"
    
    if [ -f "$key_file" ]; then
        log_warning "Key for $node_name already exists, skipping generation"
        return
    fi
    
    log_info "Generating node key for $node_name..."
    
    # Generate Ed25519 keypair using OpenSSL
    openssl genpkey -algorithm Ed25519 -out "$SECRETS_DIR/${node_name}.pem" 2>/dev/null
    
    # Extract the private key in hex format
    PRIVATE_KEY=$(openssl pkey -in "$SECRETS_DIR/${node_name}.pem" -text -noout | grep priv -A 3 | tail -n 3 | tr -d ' \n:' | head -c 64)
    
    # Create the key file in the format expected by libp2p
    cat > "$key_file" <<EOF
{
  "key_type": "ed25519",
  "private_key": "$PRIVATE_KEY"
}
EOF
    
    chmod 600 "$key_file"
    log_success "Generated key for $node_name"
}

generate_api_keys() {
    log_info "Generating API keys..."
    
    # Generate secure API key
    API_KEY=$(openssl rand -hex 32)
    echo "API_KEY=$API_KEY" > "$SECRETS_DIR/api_key.txt"
    
    # Generate MCP token if enabled
    if grep -q "QUDAG_MCP_ENABLED=true" "$ENV_FILE"; then
        MCP_TOKEN=$(openssl rand -hex 32)
        echo "MCP_TOKEN=$MCP_TOKEN" > "$SECRETS_DIR/mcp_token.txt"
    fi
    
    chmod 600 "$SECRETS_DIR"/*.txt
    log_success "Generated API keys"
}

set_fly_secrets() {
    log_info "Setting secrets in Fly.io..."
    
    # Load environment variables
    source "$ENV_FILE"
    
    # Check if flyctl is available
    if ! command -v flyctl &> /dev/null; then
        log_error "flyctl not found. Please install it first."
        exit 1
    fi
    
    # Set secrets for each node
    for i in "${!NODES[@]}"; do
        local app_name="${NODES[$i]}"
        local node_num=$((i + 1))
        
        log_info "Setting secrets for $app_name..."
        
        # Check if app exists
        if ! flyctl apps list | grep -q "$app_name"; then
            log_warning "App $app_name doesn't exist yet, skipping secrets"
            continue
        fi
        
        # Read node key
        local key_file="$SECRETS_DIR/${app_name}_key.json"
        if [ -f "$key_file" ]; then
            NODE_PRIVATE_KEY=$(jq -r '.private_key' "$key_file")
            
            # Set multiple secrets at once
            flyctl secrets set \
                NODE_PRIVATE_KEY="$NODE_PRIVATE_KEY" \
                API_KEY="$(cat "$SECRETS_DIR/api_key.txt" | cut -d= -f2)" \
                NODE_ID="node$node_num" \
                -a "$app_name" || {
                log_error "Failed to set secrets for $app_name"
                continue
            }
            
            # Set MCP token if enabled
            if [ -f "$SECRETS_DIR/mcp_token.txt" ]; then
                MCP_TOKEN=$(cat "$SECRETS_DIR/mcp_token.txt" | cut -d= -f2)
                flyctl secrets set MCP_TOKEN="$MCP_TOKEN" -a "$app_name"
            fi
            
            log_success "Set secrets for $app_name"
        else
            log_warning "No key file found for $app_name"
        fi
    done
}

generate_peer_ids() {
    log_info "Generating peer IDs for bootstrap configuration..."
    
    # This would normally use the actual libp2p library to generate peer IDs
    # For now, we'll create placeholder IDs
    cat > "$SECRETS_DIR/peer_ids.txt" <<EOF
# QuDAG Testnet Peer IDs
# Generated on $(date)

NODE1_PEER_ID=12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp
NODE2_PEER_ID=12D3KooWHKkHiNhZtKceQehHhPqwU5gy4uJkLd4tB8wZyYBxgFqg
NODE3_PEER_ID=12D3KooWLRPJAA5o6QyM5GwXiYgKWsdcFbBCjyxkJHsYJZnSocUn
NODE4_PEER_ID=12D3KooWPjceQrSwdWXPyLLeABRXmuqt69Rg3sBYbU1Nft9HyQ6X

# Bootstrap multiaddrs (will be updated after deployment)
BOOTSTRAP_PEERS=""
EOF
    
    chmod 600 "$SECRETS_DIR/peer_ids.txt"
}

verify_secrets() {
    log_info "Verifying secrets..."
    
    local all_good=true
    
    # Check node keys
    for app_name in "${NODES[@]}"; do
        if [ ! -f "$SECRETS_DIR/${app_name}_key.json" ]; then
            log_error "Missing key for $app_name"
            all_good=false
        fi
    done
    
    # Check API keys
    if [ ! -f "$SECRETS_DIR/api_key.txt" ]; then
        log_error "Missing API key"
        all_good=false
    fi
    
    if [ "$all_good" = true ]; then
        log_success "All secrets verified!"
    else
        log_error "Some secrets are missing"
        exit 1
    fi
}

print_summary() {
    echo
    echo "========================================="
    echo "Secrets Setup Summary"
    echo "========================================="
    echo
    echo "Secrets directory: $SECRETS_DIR"
    echo
    echo "Generated files:"
    ls -la "$SECRETS_DIR" 2>/dev/null | grep -E '\.(json|txt|pem)$' | awk '{print "  - " $9}'
    echo
    echo "Next steps:"
    echo "  1. Review the generated secrets"
    echo "  2. Run the deployment script: ./deployment.sh"
    echo "  3. Monitor the deployment: ./monitor-nodes.sh"
    echo
    log_warning "Keep your secrets safe! Do not commit them to version control."
}

# Main execution
main() {
    log_info "Starting QuDAG Testnet secrets setup..."
    
    check_env_file
    create_secrets_dir
    
    # Generate keys for each node
    for node in "${NODES[@]}"; do
        generate_node_key "$node"
    done
    
    generate_api_keys
    generate_peer_ids
    
    # Only set Fly secrets if --deploy flag is passed
    if [[ "${1:-}" == "--deploy" ]]; then
        set_fly_secrets
    else
        log_info "Skipping Fly.io secrets upload. Run with --deploy to upload."
    fi
    
    verify_secrets
    print_summary
    
    log_success "Secrets setup completed!"
}

# Run main function
main "$@"