#!/bin/bash
set -euo pipefail

# Enhanced QuDAG Testnet Deployment Script
# Handles proper secret management, TLS setup, and peer configuration

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
NODES_DIR="$PROJECT_ROOT/nodes"
CONFIGS_DIR="$PROJECT_ROOT/configs"
SECRETS_DIR="$PROJECT_ROOT/.secrets"
TLS_DIR="$PROJECT_ROOT/tls"

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Load environment variables
if [ -f "$PROJECT_ROOT/.env" ]; then
    source "$PROJECT_ROOT/.env"
else
    log_error ".env file not found. Creating from template..."
    cp "$PROJECT_ROOT/.env.example" "$PROJECT_ROOT/.env"
    echo "Please configure $PROJECT_ROOT/.env and run again"
    exit 1
fi

# Node configuration
declare -A NODES=(
    ["node1"]="toronto:yyz:qudag-testnet-node1"
    ["node2"]="amsterdam:ams:qudag-testnet-node2"
    ["node3"]="singapore:sin:qudag-testnet-node3"
    ["node4"]="sanfrancisco:sjc:qudag-testnet-node4"
)

# Logging functions
log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
log_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Enhanced prerequisite check
check_prerequisites() {
    log_info "Checking prerequisites..."
    
    local missing_deps=()
    
    # Check required tools
    for tool in flyctl jq openssl curl docker; do
        if ! command -v "$tool" &> /dev/null; then
            missing_deps+=("$tool")
        fi
    done
    
    if [ ${#missing_deps[@]} -ne 0 ]; then
        log_error "Missing dependencies: ${missing_deps[*]}"
        echo "Please install missing dependencies and try again"
        exit 1
    fi
    
    # Check Fly.io authentication
    if ! flyctl auth whoami &> /dev/null; then
        log_error "Not logged in to Fly.io. Please run 'flyctl auth login'"
        exit 1
    fi
    
    # Validate environment variables
    if [ -z "${FLY_API_TOKEN:-}" ]; then
        log_warning "FLY_API_TOKEN not set in .env file"
    fi
    
    if [ -z "${GRAFANA_ADMIN_PASSWORD:-}" ]; then
        log_warning "GRAFANA_ADMIN_PASSWORD not set, generating secure password..."
        GRAFANA_ADMIN_PASSWORD=$(openssl rand -base64 32)
        echo "GRAFANA_ADMIN_PASSWORD=$GRAFANA_ADMIN_PASSWORD" >> "$PROJECT_ROOT/.env"
    fi
    
    log_success "Prerequisites check passed"
}

# Setup all security components
setup_security() {
    log_info "Setting up security components..."
    
    # Generate secrets if not exists
    if [ ! -d "$SECRETS_DIR" ]; then
        bash "$SCRIPT_DIR/setup-secrets.sh" || {
            log_error "Failed to setup secrets"
            exit 1
        }
    fi
    
    # Setup TLS certificates
    if [ ! -d "$TLS_DIR" ]; then
        bash "$SCRIPT_DIR/setup-tls.sh" || {
            log_error "Failed to setup TLS"
            exit 1
        }
    fi
    
    log_success "Security components configured"
}

# Update configurations with actual values
update_configurations() {
    log_info "Updating node configurations..."
    
    # Read bootstrap peer ID
    local bootstrap_peer_id=""
    if [ -f "$SECRETS_DIR/peer_ids/node1_peer_id.txt" ]; then
        bootstrap_peer_id=$(cat "$SECRETS_DIR/peer_ids/node1_peer_id.txt")
    else
        log_error "Bootstrap peer ID not found"
        exit 1
    fi
    
    # Update each node configuration
    for node in node1 node2 node3 node4; do
        local config_file="$CONFIGS_DIR/$node.toml"
        local temp_file="$config_file.tmp"
        
        if [ -f "$config_file" ]; then
            # Replace placeholders
            sed -e "s/PEER_ID_PLACEHOLDER/$bootstrap_peer_id/g" \
                -e "s/\${NODE_ID}/$node/g" \
                -e "s/\${NODE_NAME}/${node}-${QUDAG_ENVIRONMENT}/g" \
                -e "s/\${NODE_ROLE}/$([ "$node" == "node1" ] && echo "bootstrap" || echo "validator")/g" \
                -e "s/\${IS_BOOTSTRAP}/$([ "$node" == "node1" ] && echo "true" || echo "false")/g" \
                "$config_file" > "$temp_file"
            
            mv "$temp_file" "$config_file"
            log_success "Updated $node configuration"
        fi
    done
}

# Enhanced app creation with proper configuration
create_apps() {
    log_info "Creating Fly.io applications..."
    
    for node in "${!NODES[@]}"; do
        IFS=':' read -r location region app_name <<< "${NODES[$node]}"
        
        if flyctl apps list | grep -q "$app_name"; then
            log_warning "App $app_name already exists"
        else
            log_info "Creating app $app_name in region $region..."
            
            # Create app with proper configuration
            flyctl apps create "$app_name" \
                --org "${FLY_ORG:-personal}" || {
                log_error "Failed to create app $app_name"
                exit 1
            }
            
            # Set app configuration
            flyctl config save -a "$app_name" \
                --config "$NODES_DIR/fly.$node.toml"
            
            log_success "Created app $app_name"
        fi
    done
}

# Enhanced secret management
setup_app_secrets() {
    log_info "Setting up application secrets..."
    
    for node in "${!NODES[@]}"; do
        IFS=':' read -r location region app_name <<< "${NODES[$node]}"
        
        log_info "Setting secrets for $app_name..."
        
        # Prepare secrets
        local secrets_args=""
        
        # Node private key
        if [ -f "$SECRETS_DIR/node_keys/${node}_key.pem" ]; then
            secrets_args+="NODE_PRIVATE_KEY='$(cat "$SECRETS_DIR/node_keys/${node}_key.pem")' "
        fi
        
        # API token
        if [ -f "$SECRETS_DIR/api_tokens/${node}_token.txt" ]; then
            secrets_args+="API_TOKEN='$(cat "$SECRETS_DIR/api_tokens/${node}_token.txt")' "
        fi
        
        # TLS certificates
        if [ -f "$TLS_DIR/server/$node/cert.pem" ]; then
            secrets_args+="TLS_CERT='$(cat "$TLS_DIR/server/$node/cert.pem")' "
            secrets_args+="TLS_KEY='$(cat "$TLS_DIR/server/$node/key.pem")' "
            secrets_args+="TLS_CA='$(cat "$TLS_DIR/ca/ca.pem")' "
        fi
        
        # Bootstrap configuration (for non-bootstrap nodes)
        if [ "$node" != "node1" ] && [ -f "$SECRETS_DIR/bootstrap_config.json" ]; then
            local bootstrap_peers=$(jq -r '.production.bootstrap_peers[0]' "$SECRETS_DIR/bootstrap_config.json")
            secrets_args+="BOOTSTRAP_PEERS='$bootstrap_peers' "
        fi
        
        # Common secrets
        secrets_args+="QUDAG_NETWORK_ID='${QUDAG_NETWORK_ID}' "
        secrets_args+="GRAFANA_ADMIN_PASSWORD='${GRAFANA_ADMIN_PASSWORD}' "
        secrets_args+="ALERT_WEBHOOK_URL='${ALERT_WEBHOOK_URL:-}' "
        
        # Set all secrets at once
        eval "flyctl secrets set $secrets_args -a '$app_name'" || {
            log_error "Failed to set secrets for $app_name"
            exit 1
        }
        
        log_success "Secrets configured for $app_name"
    done
}

# Deploy with proper health checks
deploy_nodes() {
    log_info "Deploying QuDAG nodes..."
    
    # Deploy bootstrap node first
    deploy_single_node "node1" true
    
    # Wait for bootstrap node to be healthy
    log_info "Waiting for bootstrap node to be healthy..."
    sleep 30
    
    # Get actual bootstrap peer address
    local bootstrap_peer_id=$(get_deployed_peer_id "qudag-testnet-node1")
    if [ -n "$bootstrap_peer_id" ]; then
        log_info "Bootstrap peer ID: $bootstrap_peer_id"
        # Update other nodes with actual bootstrap peer
        for node in node2 node3 node4; do
            IFS=':' read -r location region app_name <<< "${NODES[$node]}"
            flyctl secrets set \
                BOOTSTRAP_PEERS="/dns4/qudag-testnet-node1.fly.dev/tcp/4001/p2p/$bootstrap_peer_id" \
                -a "$app_name"
        done
    fi
    
    # Deploy remaining nodes in parallel
    log_info "Deploying validator nodes..."
    for node in node2 node3 node4; do
        deploy_single_node "$node" false &
    done
    
    # Wait for all deployments
    wait
    
    log_success "All nodes deployed"
}

# Deploy a single node
deploy_single_node() {
    local node=$1
    local is_bootstrap=$2
    IFS=':' read -r location region app_name <<< "${NODES[$node]}"
    
    log_info "Deploying $app_name to $region..."
    
    cd "$PROJECT_ROOT"
    
    flyctl deploy \
        --app "$app_name" \
        --config "$NODES_DIR/fly.$node.toml" \
        --strategy rolling \
        --wait-timeout 300 || {
        log_error "Failed to deploy $app_name"
        return 1
    }
    
    log_success "Deployed $app_name"
}

# Get peer ID from deployed node
get_deployed_peer_id() {
    local app_name=$1
    flyctl ssh console -a "$app_name" -C "qudag-node peer-id 2>/dev/null || echo ''" 2>/dev/null
}

# Enhanced health verification
verify_deployment() {
    log_info "Verifying deployment health..."
    
    local all_healthy=true
    local health_report=""
    
    for node in "${!NODES[@]}"; do
        IFS=':' read -r location region app_name <<< "${NODES[$node]}"
        
        log_info "Checking $app_name..."
        
        # Check app status
        local app_status=$(flyctl status -a "$app_name" --json 2>/dev/null | jq -r '.Status // "unknown"')
        
        # Check HTTPS health endpoint
        local health_status="unhealthy"
        local health_response=$(curl -sf --max-time 10 \
            --cacert "$TLS_DIR/ca/ca.pem" \
            "https://$app_name.fly.dev/health" 2>/dev/null || echo "{}")
        
        if [ -n "$health_response" ] && echo "$health_response" | jq -e '.status == "healthy"' &>/dev/null; then
            health_status="healthy"
            local peer_count=$(echo "$health_response" | jq -r '.peers // 0')
            local block_height=$(echo "$health_response" | jq -r '.height // 0')
            health_report+="\n  $app_name: ✓ Healthy (peers: $peer_count, height: $block_height)"
        else
            all_healthy=false
            health_report+="\n  $app_name: ✗ Unhealthy (status: $app_status)"
        fi
    done
    
    echo -e "\nHealth Report:$health_report\n"
    
    if [ "$all_healthy" = true ]; then
        log_success "All nodes are healthy!"
        
        # Run TLS verification
        if [ -f "$TLS_DIR/verify-tls.sh" ]; then
            log_info "Verifying TLS configuration..."
            "$TLS_DIR/verify-tls.sh" production
        fi
    else
        log_warning "Some nodes are unhealthy. Check logs with 'flyctl logs -a <app-name>'"
    fi
}

# Print enhanced summary
print_summary() {
    echo
    echo "========================================="
    echo "QuDAG Testnet Deployment Summary"
    echo "========================================="
    echo
    echo "Deployed Nodes:"
    for node in "${!NODES[@]}"; do
        IFS=':' read -r location region app_name <<< "${NODES[$node]}"
        echo "  - $location ($region):"
        echo "    URL: https://$app_name.fly.dev"
        echo "    Health: https://$app_name.fly.dev/health"
        echo "    Metrics: https://$app_name.fly.dev:9090/metrics"
    done
    echo
    echo "Security:"
    echo "  - TLS: Enabled (TLS 1.3)"
    echo "  - API Auth: Bearer Token / API Key"
    echo "  - CORS: Restricted to testnet domains"
    echo
    echo "Monitoring:"
    echo "  - Grafana: http://localhost:3000"
    echo "    Username: ${GRAFANA_ADMIN_USER:-admin}"
    echo "    Password: Check .env file"
    echo "  - Prometheus: http://localhost:9094"
    echo
    echo "Management Commands:"
    echo "  - Status: flyctl status -a <app-name>"
    echo "  - Logs: flyctl logs -a <app-name>"
    echo "  - SSH: flyctl ssh console -a <app-name>"
    echo "  - Monitor: ./scripts/monitor-nodes.sh"
    echo "  - Update peers: ./scripts/update-bootstrap-peers.sh"
    echo
    echo "Next Steps:"
    echo "  1. Verify all nodes are healthy"
    echo "  2. Check P2P connectivity between nodes"
    echo "  3. Test dark domain registration"
    echo "  4. Monitor consensus formation"
    echo
}

# Main execution
main() {
    log_info "Starting enhanced QuDAG Testnet deployment..."
    
    check_prerequisites
    setup_security
    update_configurations
    create_apps
    create_volumes
    setup_app_secrets
    deploy_nodes
    
    # Run bootstrap peer update after deployment
    if [ -f "$SCRIPT_DIR/update-bootstrap-peers.sh" ]; then
        bash "$SCRIPT_DIR/update-bootstrap-peers.sh"
    fi
    
    verify_deployment
    print_summary
    
    log_success "Enhanced deployment completed!"
}

# Command line argument handling
case "${1:-deploy}" in
    "deploy")
        main
        ;;
    "verify")
        verify_deployment
        print_summary
        ;;
    "update-peers")
        bash "$SCRIPT_DIR/update-bootstrap-peers.sh"
        ;;
    "status")
        print_summary
        ;;
    "help"|"--help"|"-h")
        echo "Usage: $0 [deploy|verify|update-peers|status|help]"
        echo ""
        echo "Commands:"
        echo "  deploy       - Deploy the complete testnet (default)"
        echo "  verify       - Verify deployment health"
        echo "  update-peers - Update bootstrap peer configurations"
        echo "  status       - Show deployment summary"
        echo "  help         - Show this help message"
        exit 0
        ;;
    *)
        log_error "Unknown command: $1"
        echo "Usage: $0 [deploy|verify|update-peers|status|help]"
        exit 1
        ;;
esac