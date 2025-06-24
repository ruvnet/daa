#!/bin/bash
# QuDAG Testnet Setup Script

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TESTNET_DIR="$(dirname "$SCRIPT_DIR")"
KEY_DIR="$TESTNET_DIR/keys"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check dependencies
check_dependencies() {
    log_info "Checking dependencies..."
    
    local deps=("docker" "docker-compose" "openssl" "jq")
    local missing=()
    
    for dep in "${deps[@]}"; do
        if ! command -v "$dep" &> /dev/null; then
            missing+=("$dep")
        fi
    done
    
    if [ ${#missing[@]} -ne 0 ]; then
        log_error "Missing dependencies: ${missing[*]}"
        log_info "Please install the missing dependencies and try again."
        exit 1
    fi
    
    log_success "All dependencies are installed"
}

# Generate node keys
generate_keys() {
    log_info "Generating cryptographic keys for nodes..."
    
    # Create key directories
    for i in {1..4}; do
        mkdir -p "$KEY_DIR/node$i"
    done
    
    # Generate Ed25519 keypairs for each node
    for i in {1..4}; do
        log_info "Generating keys for node$i..."
        
        # Generate private key
        openssl genpkey -algorithm Ed25519 -out "$KEY_DIR/node$i/private.pem"
        
        # Generate public key
        openssl pkey -in "$KEY_DIR/node$i/private.pem" -pubout -out "$KEY_DIR/node$i/public.pem"
        
        # Generate node ID (using SHA256 of public key)
        PEER_ID=$(openssl pkey -in "$KEY_DIR/node$i/public.pem" -pubin -noout -text | \
                  grep -A1 "pub:" | tail -1 | tr -d ' \n:' | \
                  xxd -r -p | sha256sum | cut -d' ' -f1)
        
        echo "$PEER_ID" > "$KEY_DIR/node$i/peer_id.txt"
        
        # Set proper permissions
        chmod 600 "$KEY_DIR/node$i/private.pem"
        chmod 644 "$KEY_DIR/node$i/public.pem"
        chmod 644 "$KEY_DIR/node$i/peer_id.txt"
        
        log_success "Generated keys for node$i (Peer ID: ${PEER_ID:0:16}...)"
    done
}

# Update bootstrap peer configuration
update_bootstrap_config() {
    log_info "Updating bootstrap peer configuration..."
    
    # Get node1 peer ID
    NODE1_PEER_ID=$(cat "$KEY_DIR/node1/peer_id.txt")
    
    # Update docker-compose.yml with actual peer ID
    sed -i.bak "s/NODE1_PEER_ID/$NODE1_PEER_ID/g" "$TESTNET_DIR/docker-compose.yml"
    
    # Update node config files
    for i in {2..4}; do
        sed -i.bak "s/NODE1_PEER_ID/$NODE1_PEER_ID/g" "$TESTNET_DIR/configs/node$i.toml"
    done
    
    log_success "Updated bootstrap configuration with peer ID: ${NODE1_PEER_ID:0:16}..."
}

# Build Docker image
build_image() {
    log_info "Building QuDAG Docker image..."
    
    cd "$TESTNET_DIR"
    docker build -t qudag:testnet-latest -f Dockerfile .. || {
        log_error "Failed to build Docker image"
        exit 1
    }
    
    log_success "Docker image built successfully"
}

# Create environment file
create_env_file() {
    log_info "Creating environment configuration..."
    
    cat > "$TESTNET_DIR/.env" << EOF
# QuDAG Testnet Environment Configuration
COMPOSE_PROJECT_NAME=qudag-testnet
GRAFANA_ADMIN_PASSWORD=admin123

# Testnet Configuration
QUDAG_NETWORK_ID=qudag-testnet-local
QUDAG_DARK_DOMAIN_ENABLED=true

# Node Configuration
QUDAG_P2P_PORT=4001
QUDAG_RPC_PORT=8080
QUDAG_METRICS_PORT=9090

# Performance Settings
RUST_LOG=info,qudag=debug
RUST_BACKTRACE=1

# Monitoring
PROMETHEUS_RETENTION_TIME=30d
GRAFANA_ADMIN_USER=admin
EOF
    
    log_success "Created environment file"
}

# Validate configuration
validate_config() {
    log_info "Validating configuration..."
    
    # Check if all required files exist
    local required_files=(
        "$TESTNET_DIR/Dockerfile"
        "$TESTNET_DIR/docker-compose.yml"
        "$TESTNET_DIR/monitoring/prometheus.yml"
        "$TESTNET_DIR/monitoring/alerts.yml"
    )
    
    for file in "${required_files[@]}"; do
        if [ ! -f "$file" ]; then
            log_error "Required file not found: $file"
            exit 1
        fi
    done
    
    # Check node configurations
    for i in {1..4}; do
        if [ ! -f "$TESTNET_DIR/configs/node$i.toml" ]; then
            log_error "Node configuration not found: node$i.toml"
            exit 1
        fi
        
        if [ ! -d "$KEY_DIR/node$i" ]; then
            log_error "Keys directory not found for node$i"
            exit 1
        fi
    done
    
    log_success "Configuration validation passed"
}

# Main setup function
main() {
    log_info "Starting QuDAG testnet setup..."
    
    check_dependencies
    generate_keys
    update_bootstrap_config
    create_env_file
    validate_config
    build_image
    
    log_success "QuDAG testnet setup completed!"
    echo
    log_info "You can now start the testnet with:"
    echo "  cd $TESTNET_DIR"
    echo "  docker-compose up -d"
    echo
    log_info "Access monitoring dashboards at:"
    echo "  Grafana:    http://localhost:3000 (admin/admin123)"
    echo "  Prometheus: http://localhost:9094"
    echo
    log_info "Node APIs available at:"
    for i in {1..4}; do
        echo "  Node $i: http://localhost:808$((i-1))"
    done
}

# Handle script arguments
case "${1:-setup}" in
    "setup")
        main
        ;;
    "keys-only")
        generate_keys
        update_bootstrap_config
        ;;
    "build-only")
        build_image
        ;;
    "validate")
        validate_config
        ;;
    *)
        echo "Usage: $0 [setup|keys-only|build-only|validate]"
        exit 1
        ;;
esac