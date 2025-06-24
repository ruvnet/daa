#!/bin/bash
set -euo pipefail

# QuDAG Testnet TLS Certificate Setup Script
# Generates proper TLS certificates for secure communication

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
TLS_DIR="$PROJECT_ROOT/tls"

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Create TLS directory structure
create_directories() {
    log_info "Creating TLS directories..."
    mkdir -p "$TLS_DIR"/{ca,server,client}
    chmod 700 "$TLS_DIR"
}

# Generate Certificate Authority
generate_ca() {
    log_info "Generating Certificate Authority..."
    
    # Generate CA private key
    openssl genrsa -out "$TLS_DIR/ca/ca-key.pem" 4096
    
    # Generate CA certificate
    openssl req -new -x509 -days 3650 -key "$TLS_DIR/ca/ca-key.pem" \
        -out "$TLS_DIR/ca/ca.pem" \
        -subj "/C=US/ST=State/L=City/O=QuDAG/OU=Testnet/CN=QuDAG Testnet CA"
    
    # Create CA certificate bundle
    cp "$TLS_DIR/ca/ca.pem" "$TLS_DIR/ca-bundle.pem"
    
    log_success "Generated Certificate Authority"
}

# Generate server certificate for a node
generate_node_cert() {
    local node_name=$1
    local node_domain=$2
    
    log_info "Generating certificate for $node_name..."
    
    # Create node directory
    mkdir -p "$TLS_DIR/server/$node_name"
    
    # Generate private key
    openssl genrsa -out "$TLS_DIR/server/$node_name/key.pem" 4096
    
    # Create certificate configuration
    cat > "$TLS_DIR/server/$node_name/cert.conf" <<EOF
[req]
distinguished_name = req_distinguished_name
req_extensions = v3_req
prompt = no

[req_distinguished_name]
C = US
ST = State
L = City
O = QuDAG
OU = Testnet
CN = $node_domain

[v3_req]
keyUsage = critical, digitalSignature, keyEncipherment
extendedKeyUsage = serverAuth, clientAuth
basicConstraints = CA:FALSE
subjectAltName = @alt_names

[alt_names]
DNS.1 = $node_domain
DNS.2 = localhost
DNS.3 = $node_name
DNS.4 = qudag-$node_name
IP.1 = 127.0.0.1
IP.2 = ::1
EOF

    # Add local Docker IPs for local testing
    if [[ "$node_name" == "node1" ]]; then
        echo "IP.3 = 172.20.0.10" >> "$TLS_DIR/server/$node_name/cert.conf"
    elif [[ "$node_name" == "node2" ]]; then
        echo "IP.3 = 172.20.0.11" >> "$TLS_DIR/server/$node_name/cert.conf"
    elif [[ "$node_name" == "node3" ]]; then
        echo "IP.3 = 172.20.0.12" >> "$TLS_DIR/server/$node_name/cert.conf"
    elif [[ "$node_name" == "node4" ]]; then
        echo "IP.3 = 172.20.0.13" >> "$TLS_DIR/server/$node_name/cert.conf"
    fi
    
    # Generate certificate request
    openssl req -new -key "$TLS_DIR/server/$node_name/key.pem" \
        -out "$TLS_DIR/server/$node_name/csr.pem" \
        -config "$TLS_DIR/server/$node_name/cert.conf"
    
    # Sign certificate with CA
    openssl x509 -req -in "$TLS_DIR/server/$node_name/csr.pem" \
        -CA "$TLS_DIR/ca/ca.pem" \
        -CAkey "$TLS_DIR/ca/ca-key.pem" \
        -CAcreateserial \
        -out "$TLS_DIR/server/$node_name/cert.pem" \
        -days 365 \
        -extensions v3_req \
        -extfile "$TLS_DIR/server/$node_name/cert.conf"
    
    # Create certificate chain
    cat "$TLS_DIR/server/$node_name/cert.pem" "$TLS_DIR/ca/ca.pem" \
        > "$TLS_DIR/server/$node_name/fullchain.pem"
    
    # Set permissions
    chmod 600 "$TLS_DIR/server/$node_name/key.pem"
    chmod 644 "$TLS_DIR/server/$node_name/cert.pem"
    chmod 644 "$TLS_DIR/server/$node_name/fullchain.pem"
    
    log_success "Generated certificate for $node_name"
}

# Generate client certificates for API access
generate_client_cert() {
    local client_name=$1
    
    log_info "Generating client certificate for $client_name..."
    
    mkdir -p "$TLS_DIR/client/$client_name"
    
    # Generate private key
    openssl genrsa -out "$TLS_DIR/client/$client_name/key.pem" 4096
    
    # Generate certificate request
    openssl req -new -key "$TLS_DIR/client/$client_name/key.pem" \
        -out "$TLS_DIR/client/$client_name/csr.pem" \
        -subj "/C=US/ST=State/L=City/O=QuDAG/OU=Testnet Client/CN=$client_name"
    
    # Sign certificate
    openssl x509 -req -in "$TLS_DIR/client/$client_name/csr.pem" \
        -CA "$TLS_DIR/ca/ca.pem" \
        -CAkey "$TLS_DIR/ca/ca-key.pem" \
        -CAcreateserial \
        -out "$TLS_DIR/client/$client_name/cert.pem" \
        -days 365
    
    # Create PKCS12 bundle for easy import
    openssl pkcs12 -export \
        -out "$TLS_DIR/client/$client_name/client.p12" \
        -inkey "$TLS_DIR/client/$client_name/key.pem" \
        -in "$TLS_DIR/client/$client_name/cert.pem" \
        -certfile "$TLS_DIR/ca/ca.pem" \
        -passout pass:changeme
    
    chmod 600 "$TLS_DIR/client/$client_name/key.pem"
    chmod 644 "$TLS_DIR/client/$client_name/cert.pem"
    
    log_success "Generated client certificate for $client_name"
}

# Create symlinks for Docker Compose
create_symlinks() {
    log_info "Creating TLS symlinks for Docker Compose..."
    
    # Create generic cert/key symlinks
    ln -sf "$TLS_DIR/server/node1/cert.pem" "$TLS_DIR/cert.pem"
    ln -sf "$TLS_DIR/server/node1/key.pem" "$TLS_DIR/key.pem"
    ln -sf "$TLS_DIR/ca/ca.pem" "$TLS_DIR/ca.pem"
    
    log_success "Created TLS symlinks"
}

# Generate TLS verification script
generate_verify_script() {
    cat > "$TLS_DIR/verify-tls.sh" <<'EOF'
#!/bin/bash
# Verify TLS configuration for QuDAG nodes

echo "Verifying TLS certificates..."

# Function to verify a node's certificate
verify_node() {
    local node=$1
    local domain=$2
    local port=${3:-8443}
    
    echo -n "Checking $node ($domain:$port)... "
    
    if openssl s_client -connect "$domain:$port" \
        -CAfile "$(dirname "$0")/ca/ca.pem" \
        -servername "$domain" \
        </dev/null 2>/dev/null | grep -q "Verify return code: 0"; then
        echo "✓ Valid"
    else
        echo "✗ Invalid"
    fi
}

# Verify local nodes
if [ "$1" == "local" ]; then
    verify_node "node1" "localhost" "8443"
    verify_node "node2" "localhost" "8444"
    verify_node "node3" "localhost" "8445"
    verify_node "node4" "localhost" "8446"
else
    # Verify production nodes
    verify_node "node1" "qudag-testnet-node1.fly.dev" "443"
    verify_node "node2" "qudag-testnet-node2.fly.dev" "443"
    verify_node "node3" "qudag-testnet-node3.fly.dev" "443"
    verify_node "node4" "qudag-testnet-node4.fly.dev" "443"
fi
EOF
    
    chmod +x "$TLS_DIR/verify-tls.sh"
    log_success "Generated TLS verification script"
}

# Main execution
main() {
    log_info "Setting up TLS certificates for QuDAG testnet..."
    
    # Check if TLS already exists
    if [ -d "$TLS_DIR" ] && [ -f "$TLS_DIR/ca/ca.pem" ]; then
        log_info "TLS certificates already exist"
        read -p "Do you want to regenerate all certificates? (y/N) " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            log_info "TLS setup cancelled"
            exit 0
        fi
        # Backup existing certificates
        mv "$TLS_DIR" "$TLS_DIR.backup.$(date +%Y%m%d-%H%M%S)"
    fi
    
    create_directories
    generate_ca
    
    # Generate server certificates for all nodes
    generate_node_cert "node1" "qudag-testnet-node1.fly.dev"
    generate_node_cert "node2" "qudag-testnet-node2.fly.dev"
    generate_node_cert "node3" "qudag-testnet-node3.fly.dev"
    generate_node_cert "node4" "qudag-testnet-node4.fly.dev"
    
    # Generate client certificates
    generate_client_cert "admin"
    generate_client_cert "monitoring"
    
    create_symlinks
    generate_verify_script
    
    # Print summary
    echo
    echo "========================================="
    echo "TLS Setup Complete"
    echo "========================================="
    echo
    echo "Certificate Authority:"
    echo "  CA Certificate: $TLS_DIR/ca/ca.pem"
    echo
    echo "Server Certificates:"
    for node in node1 node2 node3 node4; do
        echo "  $node:"
        echo "    Certificate: $TLS_DIR/server/$node/cert.pem"
        echo "    Private Key: $TLS_DIR/server/$node/key.pem"
        echo "    Full Chain: $TLS_DIR/server/$node/fullchain.pem"
    done
    echo
    echo "Client Certificates:"
    echo "  Admin: $TLS_DIR/client/admin/client.p12 (password: changeme)"
    echo "  Monitoring: $TLS_DIR/client/monitoring/client.p12 (password: changeme)"
    echo
    echo "Verification:"
    echo "  Run: $TLS_DIR/verify-tls.sh [local|production]"
    echo
    
    log_success "TLS setup completed!"
}

# Handle command line arguments
case "${1:-setup}" in
    "setup")
        main
        ;;
    "verify")
        if [ -f "$TLS_DIR/verify-tls.sh" ]; then
            "$TLS_DIR/verify-tls.sh" "${2:-production}"
        else
            log_error "TLS verification script not found. Run setup first."
            exit 1
        fi
        ;;
    "help"|"--help"|"-h")
        echo "Usage: $0 [setup|verify|help]"
        echo ""
        echo "Commands:"
        echo "  setup    - Generate TLS certificates (default)"
        echo "  verify   - Verify TLS configuration"
        echo "  help     - Show this help message"
        exit 0
        ;;
    *)
        log_error "Unknown command: $1"
        echo "Usage: $0 [setup|verify|help]"
        exit 1
        ;;
esac