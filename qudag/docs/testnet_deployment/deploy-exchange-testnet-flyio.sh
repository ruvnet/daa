#!/bin/bash
# QuDAG Exchange Testnet Deployment Script for Fly.io
# Deploys a 4-node testnet with full Exchange functionality and immutable deployment options

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
DEPLOYMENT_NAME="${DEPLOYMENT_NAME:-qudag-exchange-testnet}"

# Node configurations
declare -A NODES=(
    ["bootstrap"]="yyz:qudag-testnet-bootstrap:15:10:1"
    ["exchange-full"]="yul:qudag-testnet-exchange-full:15:8:1"
    ["validator"]="ord:qudag-testnet-validator:12:6:1"
    ["light"]="ewr:qudag-testnet-light:10:4:1"
)

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1" >&2
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

# Check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."
    
    local missing_deps=()
    
    # Check for required commands
    for cmd in flyctl jq curl; do
        if ! command -v $cmd &> /dev/null; then
            missing_deps+=($cmd)
        fi
    done
    
    if [ ${#missing_deps[@]} -ne 0 ]; then
        log_error "Missing dependencies: ${missing_deps[*]}"
        log_error "Please install missing dependencies and try again."
        exit 1
    fi
    
    # Check Fly.io authentication
    if ! flyctl auth whoami &> /dev/null; then
        log_error "Not authenticated with Fly.io. Run 'flyctl auth login' first."
        exit 1
    fi
    
    log_success "All prerequisites met"
}

# Generate cryptographic keys for Exchange
generate_exchange_keys() {
    log_info "Generating Exchange cryptographic keys..."
    
    local keys_dir="${PROJECT_ROOT}/testnet-keys"
    mkdir -p "${keys_dir}"
    
    # Generate master Exchange keys
    cd "${PROJECT_ROOT}"
    
    # Bootstrap node keys
    ./target/release/qudag key generate --algorithm ml-dsa --purpose exchange-master --output "${keys_dir}/exchange-master.key"
    ./target/release/qudag key generate --algorithm ml-dsa --purpose immutable-governance --output "${keys_dir}/governance.key"
    ./target/release/qudag key generate --algorithm ml-dsa --purpose fee-configuration --output "${keys_dir}/fee-admin.key"
    
    # Network shared keys
    ./target/release/qudag key generate --algorithm ml-dsa --purpose network-shared --output "${keys_dir}/network-shared.key"
    ./target/release/qudag key generate --algorithm ml-dsa --purpose consensus-shared --output "${keys_dir}/consensus-shared.key"
    
    # Agent verification keys
    ./target/release/qudag key generate --algorithm ml-dsa --purpose agent-verification --output "${keys_dir}/agent-verification.key"
    
    log_success "Generated Exchange cryptographic keys"
}

# Initialize Fly.io applications
initialize_fly_apps() {
    log_info "Initializing Fly.io applications..."
    
    cd "${SCRIPT_DIR}"
    
    for node_type in "${!NODES[@]}"; do
        local node_info="${NODES[$node_type]}"
        IFS=':' read -r region app_name qudag_size exchange_size keys_size <<< "$node_info"
        
        log_info "Initializing $node_type node: $app_name in region $region"
        
        # Copy appropriate fly.toml for each node type
        local fly_config=""
        case $node_type in
            "bootstrap")
                fly_config="fly-bootstrap.toml"
                ;;
            "exchange-full")
                fly_config="fly-exchange-full.toml"
                ;;
            "validator")
                fly_config="fly-validator.toml"
                ;;
            "light")
                fly_config="fly-light.toml"
                ;;
        esac
        
        # Launch app without deploying
        cp "$fly_config" "fly.toml"
        flyctl apps create "$app_name" --org personal 2>/dev/null || true
        
        log_success "Initialized $app_name"
    done
    
    log_success "All Fly.io applications initialized"
}

# Create volumes for Exchange data
create_volumes() {
    log_info "Creating volumes for Exchange data..."
    
    for node_type in "${!NODES[@]}"; do
        local node_info="${NODES[$node_type]}"
        IFS=':' read -r region app_name qudag_size exchange_size keys_size <<< "$node_info"
        
        log_info "Creating volumes for $app_name"
        
        # Create volumes
        flyctl volumes create qudag_data --size "$qudag_size" --region "$region" --app "$app_name" --yes
        flyctl volumes create exchange_data --size "$exchange_size" --region "$region" --app "$app_name" --yes
        flyctl volumes create exchange_keys --size "$keys_size" --region "$region" --app "$app_name" --yes
        
        log_success "Created volumes for $app_name"
    done
    
    log_success "All volumes created"
}

# Set secrets for Exchange operations
set_secrets() {
    log_info "Setting Exchange secrets..."
    
    local keys_dir="${PROJECT_ROOT}/testnet-keys"
    
    # Read generated keys
    local exchange_master_key=$(cat "${keys_dir}/exchange-master.key")
    local governance_key=$(cat "${keys_dir}/governance.key")
    local fee_admin_key=$(cat "${keys_dir}/fee-admin.key")
    local network_shared_key=$(cat "${keys_dir}/network-shared.key")
    local consensus_shared_key=$(cat "${keys_dir}/consensus-shared.key")
    local agent_verification_key=$(cat "${keys_dir}/agent-verification.key")
    
    # Set secrets for all nodes
    for node_type in "${!NODES[@]}"; do
        local node_info="${NODES[$node_type]}"
        IFS=':' read -r region app_name qudag_size exchange_size keys_size <<< "$node_info"
        
        log_info "Setting secrets for $app_name"
        
        # Core secrets for all nodes
        flyctl secrets set NODE_PRIVATE_KEY="$(openssl rand -hex 32)" --app "$app_name" --stage
        flyctl secrets set PEER_AUTH_TOKEN="$(openssl rand -hex 32)" --app "$app_name" --stage
        flyctl secrets set EXCHANGE_NETWORK_KEY="$network_shared_key" --app "$app_name" --stage
        flyctl secrets set CONSENSUS_SIGNATURE_KEY="$consensus_shared_key" --app "$app_name" --stage
        
        # Bootstrap node specific secrets
        if [ "$node_type" = "bootstrap" ]; then
            flyctl secrets set EXCHANGE_MASTER_KEY="$exchange_master_key" --app "$app_name" --stage
            flyctl secrets set EXCHANGE_GENESIS_SEED="$(openssl rand -hex 32)" --app "$app_name" --stage
            flyctl secrets set IMMUTABLE_GOVERNANCE_KEY="$governance_key" --app "$app_name" --stage
            flyctl secrets set FEE_ADMIN_KEY="$fee_admin_key" --app "$app_name" --stage
            flyctl secrets set AGENT_VERIFICATION_KEY="$agent_verification_key" --app "$app_name" --stage
        fi
        
        log_success "Set secrets for $app_name"
    done
    
    log_success "All secrets configured"
}

# Deploy applications in correct order
deploy_applications() {
    log_info "Deploying Exchange-enabled applications..."
    
    cd "${SCRIPT_DIR}"
    
    # Deploy bootstrap node first
    log_info "Deploying bootstrap node..."
    cp "fly-bootstrap.toml" "fly.toml"
    flyctl deploy --app "qudag-testnet-bootstrap" --build-arg BUILDKIT_INLINE_CACHE=1
    
    # Wait for bootstrap to be healthy
    log_info "Waiting for bootstrap node to be healthy..."
    local attempts=0
    while [ $attempts -lt 30 ]; do
        if flyctl status --app "qudag-testnet-bootstrap" | grep -q "healthy"; then
            log_success "Bootstrap node is healthy"
            break
        fi
        sleep 10
        ((attempts++))
    done
    
    if [ $attempts -eq 30 ]; then
        log_error "Bootstrap node failed to become healthy"
        return 1
    fi
    
    # Deploy other nodes in parallel
    log_info "Deploying other nodes..."
    
    # Exchange full node
    cp "fly-exchange-full.toml" "fly.toml"
    flyctl deploy --app "qudag-testnet-exchange-full" --build-arg BUILDKIT_INLINE_CACHE=1 &
    
    # Validator node
    cp "fly-validator.toml" "fly.toml" 
    flyctl deploy --app "qudag-testnet-validator" --build-arg BUILDKIT_INLINE_CACHE=1 &
    
    # Light node
    cp "fly-light.toml" "fly.toml"
    flyctl deploy --app "qudag-testnet-light" --build-arg BUILDKIT_INLINE_CACHE=1 &
    
    # Wait for all deployments
    wait
    
    log_success "All applications deployed"
}

# Initialize Exchange system
initialize_exchange() {
    log_info "Initializing Exchange system..."
    
    # Connect to bootstrap node and initialize
    log_info "Setting up genesis accounts..."
    flyctl ssh console --app "qudag-testnet-bootstrap" --command "qudag exchange create-account --name genesis-treasury"
    flyctl ssh console --app "qudag-testnet-bootstrap" --command "qudag exchange mint --account genesis-treasury --amount 10000000"
    
    # Create test accounts
    flyctl ssh console --app "qudag-testnet-bootstrap" --command "qudag exchange create-account --name alice"
    flyctl ssh console --app "qudag-testnet-bootstrap" --command "qudag exchange create-account --name bob"
    flyctl ssh console --app "qudag-testnet-bootstrap" --command "qudag exchange transfer --from genesis-treasury --to alice --amount 50000"
    flyctl ssh console --app "qudag-testnet-bootstrap" --command "qudag exchange transfer --from genesis-treasury --to bob --amount 30000"
    
    log_success "Exchange system initialized"
}

# Configure fee model across all nodes
configure_fee_model() {
    log_info "Configuring fee model across all nodes..."
    
    for node_type in "${!NODES[@]}"; do
        local node_info="${NODES[$node_type]}"
        IFS=':' read -r region app_name qudag_size exchange_size keys_size <<< "$node_info"
        
        log_info "Configuring fees for $app_name"
        
        flyctl ssh console --app "$app_name" --command "qudag exchange configure-fees \
            --f-min 0.001 \
            --f-max 0.010 \
            --f-min-verified 0.0025 \
            --f-max-verified 0.005 \
            --time-constant-days 90 \
            --usage-threshold 10000"
    done
    
    log_success "Fee model configured across all nodes"
}

# Enable immutable deployment (optional)
enable_immutable_deployment() {
    if [ "${ENABLE_IMMUTABLE:-false}" = "true" ]; then
        log_info "Enabling immutable deployment..."
        
        flyctl ssh console --app "qudag-testnet-bootstrap" --command "qudag exchange deploy-immutable --grace-period 24 --key-path /keys/governance.pem"
        
        log_success "Immutable deployment enabled with 24-hour grace period"
    else
        log_info "Immutable deployment not enabled (set ENABLE_IMMUTABLE=true to enable)"
    fi
}

# Verify deployment
verify_deployment() {
    log_info "Verifying Exchange deployment..."
    
    for node_type in "${!NODES[@]}"; do
        local node_info="${NODES[$node_type]}"
        IFS=':' read -r region app_name qudag_size exchange_size keys_size <<< "$node_info"
        
        log_info "Testing $app_name..."
        
        # Test node status
        flyctl ssh console --app "$app_name" --command "qudag status" || log_warning "Status check failed for $app_name"
        
        # Test Exchange functionality
        flyctl ssh console --app "$app_name" --command "qudag exchange status" || log_warning "Exchange status check failed for $app_name"
        
        # Test fee system
        flyctl ssh console --app "$app_name" --command "qudag exchange fee-status --examples" || log_warning "Fee status check failed for $app_name"
    done
    
    log_success "Deployment verification completed"
}

# Display deployment information
display_deployment_info() {
    log_info "Exchange Testnet Deployment Summary:"
    echo
    echo "🚀 Deployed Applications:"
    
    for node_type in "${!NODES[@]}"; do
        local node_info="${NODES[$node_type]}"
        IFS=':' read -r region app_name qudag_size exchange_size keys_size <<< "$node_info"
        
        echo "  📦 $node_type ($app_name):"
        echo "    Region: $region"
        echo "    QuDAG API: https://$app_name.fly.dev"
        echo "    Exchange API: https://$app_name.fly.dev:8081"
        echo "    Metrics: https://$app_name.fly.dev:9090"
        echo "    Storage: ${qudag_size}GB DAG + ${exchange_size}GB Exchange + ${keys_size}GB Keys"
        echo
    done
    
    echo "💰 Exchange Information:"
    echo "  Genesis Supply: 10,000,000 rUv"
    echo "  Test Accounts: alice (50,000 rUv), bob (30,000 rUv)"
    echo "  Fee Model: Dynamic tiered (0.1% - 1.0% unverified, 0.25% - 0.5% verified)"
    echo "  Immutable Mode: ${ENABLE_IMMUTABLE:-false}"
    echo
    
    echo "🔧 Management Commands:"
    echo "  flyctl ssh console --app qudag-testnet-bootstrap"
    echo "  qudag exchange accounts --format json"
    echo "  qudag exchange transfer --from alice --to bob --amount 1000"
    echo "  qudag exchange verify-agent --account alice --proof-path /keys/alice-kyc.json"
    echo "  qudag exchange deploy-immutable --grace-period 24"
    echo
}

# Clean up deployment
cleanup_deployment() {
    log_info "Cleaning up Exchange testnet deployment..."
    
    for node_type in "${!NODES[@]}"; do
        local node_info="${NODES[$node_type]}"
        IFS=':' read -r region app_name qudag_size exchange_size keys_size <<< "$node_info"
        
        log_info "Destroying $app_name..."
        flyctl apps destroy "$app_name" --yes || log_warning "Failed to destroy $app_name"
    done
    
    # Clean up local files
    rm -rf "${PROJECT_ROOT}/testnet-keys"
    rm -f "${SCRIPT_DIR}/fly.toml"
    
    log_success "Cleanup completed"
}

# Main execution
main() {
    case "${1:-deploy}" in
        deploy)
            check_prerequisites
            generate_exchange_keys
            initialize_fly_apps
            create_volumes
            set_secrets
            deploy_applications
            sleep 30  # Allow services to start
            initialize_exchange
            configure_fee_model
            enable_immutable_deployment
            verify_deployment
            display_deployment_info
            ;;
        clean)
            cleanup_deployment
            ;;
        info)
            display_deployment_info
            ;;
        verify)
            verify_deployment
            ;;
        *)
            echo "Usage: $0 {deploy|clean|info|verify}"
            echo
            echo "Commands:"
            echo "  deploy   - Deploy Exchange-enabled testnet (default)"
            echo "  clean    - Clean up testnet and remove all resources"
            echo "  info     - Display testnet information"
            echo "  verify   - Verify deployment health"
            echo
            echo "Environment variables:"
            echo "  ENABLE_IMMUTABLE - Enable immutable deployment (default: false)"
            echo "  DEPLOYMENT_NAME - Name prefix for deployment (default: qudag-exchange-testnet)"
            exit 1
            ;;
    esac
}

# Run main function
main "$@"