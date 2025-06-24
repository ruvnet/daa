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

# Load environment variables
if [ -f "$PROJECT_ROOT/.env" ]; then
    source "$PROJECT_ROOT/.env"
else
    log_error ".env file not found. Please copy .env.example to .env and configure it."
    exit 1
fi

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
    
    # Check for jq
    if ! command -v jq &> /dev/null; then
        log_error "jq is not installed. Please install it for JSON processing."
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
            flyctl apps create "$app_name" --org "${FLY_ORG:-personal}" || {
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
        volume_name="qudag_data"
        
        # Check if volume exists
        if flyctl volumes list -a "$app_name" 2>/dev/null | grep -q "$volume_name"; then
            log_warning "Volume $volume_name already exists for $app_name, skipping"
        else
            log_info "Creating volume $volume_name for $app_name in region $region..."
            flyctl volumes create "$volume_name" \
                --app "$app_name" \
                --region "$region" \
                --size "${VOLUME_SIZE_GB:-10}" \
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
    
    # Run the setup-secrets script if it hasn't been run
    if [ ! -d "$PROJECT_ROOT/.secrets" ]; then
        if [ -f "$SCRIPT_DIR/setup-secrets.sh" ]; then
            bash "$SCRIPT_DIR/setup-secrets.sh" || {
                log_error "Failed to setup secrets"
                exit 1
            }
        else
            log_error "setup-secrets.sh not found"
            exit 1
        fi
    fi
    
    # Set secrets for each app
    for node in "${!NODES[@]}"; do
        IFS=':' read -r location region app_name <<< "${NODES[$node]}"
        
        log_info "Setting secrets for $app_name..."
        
        # Read the node's private key
        if [ -f "$PROJECT_ROOT/.secrets/node_keys/${node}_key.pem" ]; then
            flyctl secrets set NODE_PRIVATE_KEY="$(cat $PROJECT_ROOT/.secrets/node_keys/${node}_key.pem)" -a "$app_name"
        fi
        
        # Set API token
        if [ -f "$PROJECT_ROOT/.secrets/api_tokens/${node}_token.txt" ]; then
            flyctl secrets set API_TOKEN="$(cat $PROJECT_ROOT/.secrets/api_tokens/${node}_token.txt)" -a "$app_name"
        fi
        
        # Set common secrets
        flyctl secrets set \
            QUDAG_NETWORK_ID="${QUDAG_NETWORK_ID}" \
            QUDAG_DARK_DOMAIN_ENABLED="${QUDAG_DARK_DOMAIN_ENABLED}" \
            -a "$app_name"
    done
}

deploy_nodes() {
    log_info "Deploying QuDAG nodes..."
    
    # Get bootstrap peers from first deployed node
    BOOTSTRAP_PEERS=""
    
    for node in node1 node2 node3 node4; do
        IFS=':' read -r location region app_name <<< "${NODES[$node]}"
        
        log_info "Deploying $app_name to $region..."
        
        # Set bootstrap peers for non-bootstrap nodes
        if [ -n "$BOOTSTRAP_PEERS" ] && [ "$node" != "node1" ]; then
            flyctl secrets set BOOTSTRAP_PEERS="$BOOTSTRAP_PEERS" -a "$app_name"
        fi
        
        # Deploy using the specific fly.toml
        cd "$PROJECT_ROOT"
        
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
            # Get the IPv6 address
            NODE_IP=$(flyctl ips list -a "$app_name" --json | jq -r '.[] | select(.Type == "v6") | .Address // empty')
            if [ -n "$NODE_IP" ]; then
                # Get peer ID from the deployed node
                PEER_ID=$(flyctl ssh console -a "$app_name" -C "qudag-node peer-id" 2>/dev/null || echo "QmNode1PeerID")
                BOOTSTRAP_PEERS="/ip6/$NODE_IP/tcp/4001/p2p/$PEER_ID"
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
            
            # Check health endpoint with proper error handling
            APP_URL="https://$app_name.fly.dev"
            if curl -sfk --max-time 10 "$APP_URL/health" > /dev/null 2>&1; then
                log_success "$app_name health check passed"
            else
                # Try HTTP fallback
                HTTP_URL="http://$app_name.fly.dev"
                if curl -sf --max-time 10 "$HTTP_URL/health" > /dev/null 2>&1; then
                    log_warning "$app_name health check passed (HTTP only)"
                else
                    log_warning "$app_name health check failed (may need more time to start)"
                    all_healthy=false
                fi
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
    echo "  - Monitor all: ./scripts/monitor-nodes.sh"
    echo "  - Cleanup: ./scripts/cleanup.sh"
    echo
    echo "API Endpoints:"
    echo "  - Health: http://<app-name>.fly.dev/health (or https with -k flag)"
    echo "  - Metrics: http://<app-name>.fly.dev:9090/metrics"
    echo "  - RPC: http://<app-name>.fly.dev/api/v1/rpc"
    echo
    echo "Monitoring:"
    echo "  - Prometheus: http://localhost:9094 (local)"
    echo "  - Grafana: http://localhost:3000 (local)"
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
        print_summary
        ;;
    "status")
        print_summary
        ;;
    "help"|"--help"|"-h")
        echo "Usage: $0 [deploy|verify|status|help]"
        echo ""
        echo "Commands:"
        echo "  deploy  - Deploy the complete testnet (default)"
        echo "  verify  - Verify deployment health"
        echo "  status  - Show deployment summary"
        echo "  help    - Show this help message"
        exit 0
        ;;
    *)
        log_error "Unknown command: $1"
        echo "Usage: $0 [deploy|verify|status|help]"
        exit 1
        ;;
esac