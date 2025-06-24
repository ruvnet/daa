#!/bin/bash
set -euo pipefail

# QuDAG Testnet Secret Generation Script
# Generates cryptographic keys, API tokens, and peer IDs for all nodes

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
SECRETS_DIR="$PROJECT_ROOT/.secrets"

# Load environment variables
if [ -f "$PROJECT_ROOT/.env" ]; then
    source "$PROJECT_ROOT/.env"
fi

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

check_prerequisites() {
    log_info "Checking prerequisites..."
    
    # Check for openssl
    if ! command -v openssl &> /dev/null; then
        log_error "openssl is not installed. Please install it for key generation."
        exit 1
    fi
    
    # Check for base64
    if ! command -v base64 &> /dev/null; then
        log_error "base64 is not installed. Please install it."
        exit 1
    fi
    
    log_success "Prerequisites check passed"
}

create_directories() {
    log_info "Creating secrets directories..."
    
    mkdir -p "$SECRETS_DIR"/{node_keys,api_tokens,peer_ids,tls}
    chmod 700 "$SECRETS_DIR"
    
    log_success "Created secrets directories"
}

generate_ed25519_keys() {
    log_info "Generating Ed25519 keypairs for nodes..."
    
    for node in node1 node2 node3 node4; do
        KEY_FILE="$SECRETS_DIR/node_keys/${node}_key.pem"
        PUB_FILE="$SECRETS_DIR/node_keys/${node}_pub.pem"
        
        if [ -f "$KEY_FILE" ]; then
            log_warning "Key for $node already exists, skipping"
        else
            log_info "Generating keypair for $node..."
            
            # Generate Ed25519 private key
            openssl genpkey -algorithm Ed25519 -out "$KEY_FILE"
            
            # Extract public key
            openssl pkey -in "$KEY_FILE" -pubout -out "$PUB_FILE"
            
            # Set secure permissions
            chmod 600 "$KEY_FILE"
            chmod 644 "$PUB_FILE"
            
            log_success "Generated keypair for $node"
        fi
    done
}

generate_api_tokens() {
    log_info "Generating API authentication tokens..."
    
    for node in node1 node2 node3 node4; do
        TOKEN_FILE="$SECRETS_DIR/api_tokens/${node}_token.txt"
        
        if [ -f "$TOKEN_FILE" ]; then
            log_warning "API token for $node already exists, skipping"
        else
            log_info "Generating API token for $node..."
            
            # Generate secure random token (32 bytes, base64 encoded)
            TOKEN=$(openssl rand -base64 32 | tr -d '\n')
            echo "$TOKEN" > "$TOKEN_FILE"
            chmod 600 "$TOKEN_FILE"
            
            log_success "Generated API token for $node"
        fi
    done
}

generate_peer_ids() {
    log_info "Generating peer IDs..."
    
    # Note: In a real implementation, peer IDs would be derived from the public keys
    # using the libp2p peer ID generation algorithm. This is a placeholder.
    
    for node in node1 node2 node3 node4; do
        PEER_ID_FILE="$SECRETS_DIR/peer_ids/${node}_peer_id.txt"
        
        if [ -f "$PEER_ID_FILE" ]; then
            log_warning "Peer ID for $node already exists, skipping"
        else
            log_info "Generating peer ID for $node..."
            
            # Generate a placeholder peer ID
            # In production, this would be derived from the Ed25519 public key
            PEER_ID="12D3KooW$(openssl rand -hex 19)"
            echo "$PEER_ID" > "$PEER_ID_FILE"
            chmod 644 "$PEER_ID_FILE"
            
            log_success "Generated peer ID for $node: $PEER_ID"
        fi
    done
}

generate_tls_certificates() {
    log_info "Generating TLS certificates (optional)..."
    
    if [ "${ENABLE_TLS:-true}" != "true" ]; then
        log_info "TLS is disabled, skipping certificate generation"
        return
    fi
    
    for node in node1 node2 node3 node4; do
        CERT_DIR="$SECRETS_DIR/tls/$node"
        mkdir -p "$CERT_DIR"
        
        KEY_FILE="$CERT_DIR/tls_key.pem"
        CERT_FILE="$CERT_DIR/tls_cert.pem"
        
        if [ -f "$CERT_FILE" ]; then
            log_warning "TLS certificate for $node already exists, skipping"
        else
            log_info "Generating TLS certificate for $node..."
            
            # Generate self-signed certificate
            openssl req -x509 -newkey rsa:4096 -nodes \
                -keyout "$KEY_FILE" \
                -out "$CERT_FILE" \
                -days 365 \
                -subj "/C=US/ST=State/L=City/O=QuDAG/CN=qudag-testnet-${node}.fly.dev" \
                2>/dev/null
            
            chmod 600 "$KEY_FILE"
            chmod 644 "$CERT_FILE"
            
            log_success "Generated TLS certificate for $node"
        fi
    done
}

generate_bootstrap_config() {
    log_info "Generating bootstrap configuration..."
    
    BOOTSTRAP_FILE="$SECRETS_DIR/bootstrap_config.json"
    
    # Read peer IDs
    NODE1_PEER_ID=$(cat "$SECRETS_DIR/peer_ids/node1_peer_id.txt" 2>/dev/null || echo "QmNode1PeerID")
    
    # Create bootstrap configuration
    cat > "$BOOTSTRAP_FILE" <<EOF
{
  "bootstrap_peers": [
    "/dns4/qudag-testnet-node1.fly.dev/tcp/4001/p2p/$NODE1_PEER_ID"
  ],
  "nodes": {
    "node1": {
      "peer_id": "$NODE1_PEER_ID",
      "region": "yyz",
      "role": "bootstrap"
    },
    "node2": {
      "peer_id": "$(cat $SECRETS_DIR/peer_ids/node2_peer_id.txt 2>/dev/null || echo 'QmNode2PeerID')",
      "region": "ams",
      "role": "validator"
    },
    "node3": {
      "peer_id": "$(cat $SECRETS_DIR/peer_ids/node3_peer_id.txt 2>/dev/null || echo 'QmNode3PeerID')",
      "region": "sin",
      "role": "validator"
    },
    "node4": {
      "peer_id": "$(cat $SECRETS_DIR/peer_ids/node4_peer_id.txt 2>/dev/null || echo 'QmNode4PeerID')",
      "region": "sjc",
      "role": "validator"
    }
  }
}
EOF
    
    chmod 644 "$BOOTSTRAP_FILE"
    log_success "Generated bootstrap configuration"
}

rotate_keys() {
    log_warning "Rotating keys will invalidate existing deployments!"
    read -p "Are you sure you want to rotate all keys? (y/N) " -n 1 -r
    echo
    
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        log_info "Backing up existing secrets..."
        
        BACKUP_DIR="$SECRETS_DIR.backup.$(date +%Y%m%d-%H%M%S)"
        mv "$SECRETS_DIR" "$BACKUP_DIR"
        
        log_info "Generating new secrets..."
        create_directories
        generate_ed25519_keys
        generate_api_tokens
        generate_peer_ids
        generate_tls_certificates
        generate_bootstrap_config
        
        log_success "Key rotation completed. Old keys backed up to: $BACKUP_DIR"
    else
        log_info "Key rotation cancelled"
    fi
}

print_summary() {
    echo
    echo "========================================="
    echo "Secret Generation Summary"
    echo "========================================="
    echo
    echo "Generated files in $SECRETS_DIR:"
    echo
    echo "Node Keys:"
    for node in node1 node2 node3 node4; do
        if [ -f "$SECRETS_DIR/node_keys/${node}_key.pem" ]; then
            echo "  ✓ ${node}_key.pem"
        fi
    done
    echo
    echo "API Tokens:"
    for node in node1 node2 node3 node4; do
        if [ -f "$SECRETS_DIR/api_tokens/${node}_token.txt" ]; then
            echo "  ✓ ${node}_token.txt"
        fi
    done
    echo
    echo "Peer IDs:"
    for node in node1 node2 node3 node4; do
        if [ -f "$SECRETS_DIR/peer_ids/${node}_peer_id.txt" ]; then
            PEER_ID=$(cat "$SECRETS_DIR/peer_ids/${node}_peer_id.txt")
            echo "  ✓ $node: $PEER_ID"
        fi
    done
    echo
    echo "Bootstrap Configuration:"
    if [ -f "$SECRETS_DIR/bootstrap_config.json" ]; then
        echo "  ✓ bootstrap_config.json"
    fi
    echo
    echo "IMPORTANT: Keep these files secure and never commit them to git!"
    echo
}

# Main execution
main() {
    log_info "Starting secret generation..."
    
    check_prerequisites
    
    # Check if secrets already exist
    if [ -d "$SECRETS_DIR" ] && [ "$(ls -A $SECRETS_DIR 2>/dev/null)" ]; then
        log_warning "Secrets directory already exists and contains files"
        read -p "Do you want to continue and potentially overwrite? (y/N) " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            log_info "Secret generation cancelled"
            exit 0
        fi
    fi
    
    create_directories
    generate_ed25519_keys
    generate_api_tokens
    generate_peer_ids
    generate_tls_certificates
    generate_bootstrap_config
    print_summary
    
    log_success "Secret generation completed!"
}

# Handle command line arguments
case "${1:-generate}" in
    "generate")
        main
        ;;
    "rotate"|"--rotate-keys")
        rotate_keys
        ;;
    "show"|"summary")
        print_summary
        ;;
    "help"|"--help"|"-h")
        echo "Usage: $0 [generate|rotate|show|help]"
        echo ""
        echo "Commands:"
        echo "  generate     - Generate new secrets (default)"
        echo "  rotate       - Rotate all keys (backs up existing)"
        echo "  show         - Show summary of generated secrets"
        echo "  help         - Show this help message"
        exit 0
        ;;
    *)
        log_error "Unknown command: $1"
        echo "Usage: $0 [generate|rotate|show|help]"
        exit 1
        ;;
esac