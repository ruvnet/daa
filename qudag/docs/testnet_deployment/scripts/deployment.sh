#!/bin/bash
set -euo pipefail

# QuDAG Testnet Deployment Script for Fly.io
# This script deploys a 4-node QuDAG testnet across multiple regions

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
NODES_DIR="$PROJECT_ROOT/nodes"
CONFIGS_DIR="$PROJECT_ROOT/configs"

# Node configuration
declare -A NODES=(
    ["node1"]="toronto:yyz:qudag-testnet-node1"
    ["node2"]="amsterdam:ams:qudag-testnet-node2"
    ["node3"]="singapore:sin:qudag-testnet-node3"
    ["node4"]="sanfrancisco:sjc:qudag-testnet-node4"
)

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
    
    # Check for flyctl
    if ! command -v flyctl &> /dev/null; then
        log_error "flyctl is not installed. Please install it from https://fly.io/docs/hands-on/install-flyctl/"
        exit 1
    fi
    
    # Check if logged in to Fly.io
    if ! flyctl auth whoami &> /dev/null; then
        log_error "Not logged in to Fly.io. Please run 'flyctl auth login'"
        exit 1
    fi
    
    # Check for required files
    if [ ! -f "$PROJECT_ROOT/Dockerfile" ]; then
        log_error "Dockerfile not found at $PROJECT_ROOT/Dockerfile"
        exit 1
    fi
    
    log_success "Prerequisites check passed"
}

create_apps() {
    log_info "Creating Fly.io applications..."
    
    for node in "${!NODES[@]}"; do
        IFS=':' read -r location region app_name <<< "${NODES[$node]}"
        
        if flyctl apps list | grep -q "$app_name"; then
            log_warning "App $app_name already exists, skipping creation"
        else
            log_info "Creating app $app_name in region $region..."
            flyctl apps create "$app_name" --org personal || {
                log_error "Failed to create app $app_name"
                exit 1
            }
            log_success "Created app $app_name"
        fi
    done
}

create_volumes() {
    log_info "Creating persistent volumes..."
    
    for node in "${!NODES[@]}"; do
        IFS=':' read -r location region app_name <<< "${NODES[$node]}"
        volume_name="qudag_data_$node"
        
        # Check if volume exists
        if flyctl volumes list -a "$app_name" 2>/dev/null | grep -q "$volume_name"; then
            log_warning "Volume $volume_name already exists for $app_name, skipping"
        else
            log_info "Creating volume $volume_name for $app_name in region $region..."
            flyctl volumes create "$volume_name" \
                --app "$app_name" \
                --region "$region" \
                --size 10 \
                --yes || {
                log_error "Failed to create volume for $app_name"
                exit 1
            }
            log_success "Created volume $volume_name"
        fi
    done
}

setup_secrets() {
    log_info "Setting up secrets..."
    
    # Run the setup-secrets script
    if [ -f "$SCRIPT_DIR/setup-secrets.sh" ]; then
        bash "$SCRIPT_DIR/setup-secrets.sh" || {
            log_error "Failed to setup secrets"
            exit 1
        }
    else
        log_warning "setup-secrets.sh not found, skipping secrets setup"
    fi
}

deploy_nodes() {
    log_info "Deploying QuDAG nodes..."
    
    # Get bootstrap peers from first deployed node
    BOOTSTRAP_PEERS=""
    
    for node in node1 node2 node3 node4; do
        IFS=':' read -r location region app_name <<< "${NODES[$node]}"
        
        log_info "Deploying $app_name to $region..."
        
        # Deploy using the specific fly.toml
        cd "$PROJECT_ROOT"
        
        if [ -n "$BOOTSTRAP_PEERS" ]; then
            flyctl secrets set BOOTSTRAP_PEERS="$BOOTSTRAP_PEERS" -a "$app_name"
        fi
        
        flyctl deploy \
            --app "$app_name" \
            --config "$NODES_DIR/fly.$node.toml" \
            --wait-timeout 300 || {
            log_error "Failed to deploy $app_name"
            exit 1
        }
        
        log_success "Deployed $app_name"
        
        # Get the peer address of the deployed node for bootstrap
        if [ -z "$BOOTSTRAP_PEERS" ]; then
            # Get the IPv6 address and peer ID
            NODE_IP=$(flyctl ips list -a "$app_name" --json | jq -r '.[0].Address // empty')
            if [ -n "$NODE_IP" ]; then
                # In production, you'd get the actual peer ID from the node
                # For now, we'll construct a multiaddr
                BOOTSTRAP_PEERS="/ip6/$NODE_IP/tcp/4001/p2p/QmNodePeerID$node"
                log_info "Bootstrap peer: $BOOTSTRAP_PEERS"
            fi
        fi
    done
}

verify_deployment() {
    log_info "Verifying deployment..."
    
    local all_healthy=true
    
    for node in "${!NODES[@]}"; do
        IFS=':' read -r location region app_name <<< "${NODES[$node]}"
        
        log_info "Checking health of $app_name..."
        
        # Check app status
        if flyctl status -a "$app_name" | grep -q "Deployed"; then
            log_success "$app_name is deployed"
            
            # Check health endpoint
            APP_URL="https://$app_name.fly.dev"
            if curl -sf "$APP_URL/health" > /dev/null; then
                log_success "$app_name health check passed"
            else
                log_warning "$app_name health check failed"
                all_healthy=false
            fi
        else
            log_error "$app_name is not deployed properly"
            all_healthy=false
        fi
    done
    
    if [ "$all_healthy" = true ]; then
        log_success "All nodes are healthy!"
    else
        log_warning "Some nodes are not healthy. Check the logs with 'flyctl logs -a <app-name>'"
    fi
}

print_summary() {
    echo
    echo "========================================="
    echo "QuDAG Testnet Deployment Summary"
    echo "========================================="
    echo
    echo "Deployed Nodes:"
    for node in "${!NODES[@]}"; do
        IFS=':' read -r location region app_name <<< "${NODES[$node]}"
        echo "  - $location ($region): https://$app_name.fly.dev"
    done
    echo
    echo "Useful Commands:"
    echo "  - Check status: flyctl status -a <app-name>"
    echo "  - View logs: flyctl logs -a <app-name>"
    echo "  - SSH into node: flyctl ssh console -a <app-name>"
    echo "  - Monitor all: ./monitor-nodes.sh"
    echo "  - Cleanup: ./cleanup.sh"
    echo
    echo "API Endpoints:"
    echo "  - Health: https://<app-name>.fly.dev/health"
    echo "  - Metrics: https://<app-name>.fly.dev:9090/metrics"
    echo "  - RPC: https://<app-name>.fly.dev/rpc"
    echo
}

# Main execution
main() {
    log_info "Starting QuDAG Testnet deployment..."
    
    check_prerequisites
    create_apps
    create_volumes
    setup_secrets
    deploy_nodes
    verify_deployment
    print_summary
    
    log_success "Deployment completed!"
}

# Handle command line arguments
case "${1:-deploy}" in
    "deploy")
        main
        ;;
    "verify")
        verify_deployment
        ;;
    "status")
        print_summary
        ;;
    *)
        echo "Usage: $0 [deploy|verify|status]"
        exit 1
        ;;
esac