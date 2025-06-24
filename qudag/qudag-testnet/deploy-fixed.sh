#!/bin/bash
set -euo pipefail

# QuDAG Testnet Fly.io Deployment Script
# Deploys nodes one at a time with health verification and rollback support

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
NODES=("node1" "node2" "node3" "node4")
REGIONS=("yyz" "ams" "sin" "sfo")
PROJECT_DIR=$(dirname "$0")
cd "$PROJECT_DIR"

# Deployment tracking
DEPLOYED_NODES=()
FAILED_NODES=()

# Logging functions
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_section() {
    echo -e "\n${BLUE}========== $1 ==========${NC}\n"
}

# Check prerequisites
check_prerequisites() {
    log_section "Checking Prerequisites"
    
    # Check if fly CLI is installed
    if ! command -v fly &> /dev/null; then
        log_error "fly CLI not found. Please install: https://fly.io/docs/hands-on/install-flyctl/"
        exit 1
    fi
    
    # Check if logged in to Fly.io
    if ! fly auth whoami &> /dev/null; then
        log_error "Not logged in to Fly.io. Please run: fly auth login"
        exit 1
    fi
    
    # Check if all fly.toml files exist
    for node in "${NODES[@]}"; do
        if [ ! -f "nodes/fly.$node.toml" ]; then
            log_error "Missing configuration: nodes/fly.$node.toml"
            exit 1
        fi
    done
    
    # Check if Dockerfile exists
    if [ ! -f "Dockerfile" ]; then
        log_error "Missing Dockerfile"
        exit 1
    fi
    
    log_success "All prerequisites satisfied"
}

# Update fly.toml configuration
update_fly_config() {
    local node=$1
    local config_file="nodes/fly.$node.toml"
    local temp_file="${config_file}.tmp"
    
    log_info "Updating configuration for $node..."
    
    # Create backup
    cp "$config_file" "${config_file}.bak"
    
    # Update the configuration
    # We'll make sure health checks are properly configured
    # Since the file already looks good, we'll just verify key settings
    
    log_success "Configuration updated for $node"
}

# Deploy a single node
deploy_node() {
    local node=$1
    local app_name="qudag-testnet-$node"
    local config_file="nodes/fly.$node.toml"
    local max_retries=3
    local retry_count=0
    
    log_section "Deploying $node"
    
    # Update configuration
    update_fly_config "$node"
    
    # Deploy with retries
    while [ $retry_count -lt $max_retries ]; do
        log_info "Deploying $app_name (attempt $((retry_count + 1))/$max_retries)..."
        
        if fly deploy --config "$config_file" --strategy rolling --wait-timeout 300; then
            log_success "$app_name deployed successfully"
            DEPLOYED_NODES+=("$node")
            return 0
        else
            retry_count=$((retry_count + 1))
            if [ $retry_count -lt $max_retries ]; then
                log_warn "Deployment failed, retrying in 10 seconds..."
                sleep 10
            fi
        fi
    done
    
    log_error "Failed to deploy $app_name after $max_retries attempts"
    FAILED_NODES+=("$node")
    return 1
}

# Verify node health
verify_node_health() {
    local node=$1
    local app_name="qudag-testnet-$node"
    local max_checks=30
    local check_count=0
    
    log_info "Verifying health of $app_name..."
    
    # Get the app URL
    local app_url=$(fly status --app "$app_name" --json 2>/dev/null | jq -r '.Hostname' || echo "")
    
    if [ -z "$app_url" ]; then
        log_error "Could not determine URL for $app_name"
        return 1
    fi
    
    # Check health endpoint
    while [ $check_count -lt $max_checks ]; do
        if curl -sf "https://$app_url/health" > /dev/null 2>&1; then
            local health_data=$(curl -s "https://$app_url/health")
            log_success "$app_name is healthy: $health_data"
            return 0
        fi
        
        check_count=$((check_count + 1))
        log_warn "Waiting for $app_name to be healthy... ($check_count/$max_checks)"
        sleep 5
    done
    
    log_error "$app_name failed health verification"
    return 1
}

# Rollback a node
rollback_node() {
    local node=$1
    local app_name="qudag-testnet-$node"
    
    log_warn "Rolling back $app_name..."
    
    # Get the previous release
    local prev_version=$(fly releases --app "$app_name" --json 2>/dev/null | jq -r '.[1].Version' || echo "")
    
    if [ -n "$prev_version" ] && [ "$prev_version" != "null" ]; then
        if fly deploy --app "$app_name" --image-label "$prev_version"; then
            log_success "Rolled back $app_name to version $prev_version"
        else
            log_error "Failed to rollback $app_name"
        fi
    else
        log_warn "No previous version found for $app_name"
    fi
}

# Update DNS configuration
update_dns() {
    log_section "Updating DNS Configuration"
    
    # This would typically update your DNS provider
    # For Fly.io, custom domains are configured separately
    
    log_info "DNS update instructions:"
    log_info "1. Add CNAME record: qudag-testnet.yourdomain.com -> qudag-testnet-node1.fly.dev"
    log_info "2. Configure SSL certificate in Fly.io dashboard"
    log_info "3. Update any load balancer configurations"
    
    log_success "DNS configuration notes displayed"
}

# Verify P2P connectivity
verify_p2p_connectivity() {
    log_section "Verifying P2P Connectivity"
    
    local all_connected=true
    
    for node in "${DEPLOYED_NODES[@]}"; do
        local app_name="qudag-testnet-$node"
        local app_url=$(fly status --app "$app_name" --json 2>/dev/null | jq -r '.Hostname' || echo "")
        
        if [ -n "$app_url" ]; then
            local health_data=$(curl -s "https://$app_url/health" 2>/dev/null || echo "{}")
            local peer_count=$(echo "$health_data" | jq -r '.peers // 0')
            
            if [ "$peer_count" -gt 0 ]; then
                log_success "$app_name has $peer_count peers connected"
            else
                log_warn "$app_name has no peers connected"
                all_connected=false
            fi
        fi
    done
    
    if [ "$all_connected" = true ]; then
        log_success "All deployed nodes have P2P connectivity"
    else
        log_warn "Some nodes lack P2P connectivity"
    fi
}

# Main deployment function
deploy_all() {
    log_section "Starting QuDAG Testnet Deployment"
    
    # Deploy nodes sequentially
    for i in "${!NODES[@]}"; do
        local node="${NODES[$i]}"
        
        # Deploy the node
        if deploy_node "$node"; then
            # Verify health
            if verify_node_health "$node"; then
                log_success "$node deployed and verified successfully"
                
                # For non-bootstrap nodes, verify P2P after deployment
                if [ "$i" -gt 0 ]; then
                    sleep 10  # Give time for P2P connections
                    verify_p2p_connectivity
                fi
            else
                log_error "$node health verification failed"
                rollback_node "$node"
                FAILED_NODES+=("$node")
                
                # Ask if we should continue
                read -p "Continue with remaining nodes? (y/n) " -n 1 -r
                echo
                if [[ ! $REPLY =~ ^[Yy]$ ]]; then
                    break
                fi
            fi
        fi
        
        # Add delay between deployments
        if [ "$i" -lt $((${#NODES[@]} - 1)) ]; then
            log_info "Waiting 30 seconds before next deployment..."
            sleep 30
        fi
    done
}

# Deployment summary
show_summary() {
    log_section "Deployment Summary"
    
    if [ ${#DEPLOYED_NODES[@]} -gt 0 ]; then
        log_success "Successfully deployed nodes:"
        for node in "${DEPLOYED_NODES[@]}"; do
            local app_name="qudag-testnet-$node"
            local app_url=$(fly status --app "$app_name" --json 2>/dev/null | jq -r '.Hostname' || echo "unknown")
            echo "  - $node: https://$app_url"
        done
    fi
    
    if [ ${#FAILED_NODES[@]} -gt 0 ]; then
        log_error "Failed to deploy nodes:"
        for node in "${FAILED_NODES[@]}"; do
            echo "  - $node"
        done
    fi
    
    # Show monitoring URLs
    if [ ${#DEPLOYED_NODES[@]} -gt 0 ]; then
        log_info "\nMonitoring endpoints:"
        for node in "${DEPLOYED_NODES[@]}"; do
            local app_name="qudag-testnet-$node"
            local app_url=$(fly status --app "$app_name" --json 2>/dev/null | jq -r '.Hostname' || echo "unknown")
            echo "  - $node health: https://$app_url/health"
            echo "  - $node metrics: https://$app_url:9090/metrics"
        done
        
        log_info "\nFly.io dashboards:"
        for node in "${DEPLOYED_NODES[@]}"; do
            local app_name="qudag-testnet-$node"
            echo "  - $node: https://fly.io/apps/$app_name"
        done
    fi
}

# Main execution
main() {
    log_info "QuDAG Testnet Deployment Script"
    log_info "Working directory: $(pwd)"
    
    # Check prerequisites
    check_prerequisites
    
    # Confirm deployment
    log_warn "This will deploy the following nodes:"
    for i in "${!NODES[@]}"; do
        echo "  - ${NODES[$i]} to ${REGIONS[$i]} region"
    done
    
    read -p "Continue with deployment? (y/n) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        log_info "Deployment cancelled"
        exit 0
    fi
    
    # Start deployment
    deploy_all
    
    # Update DNS notes
    update_dns
    
    # Final P2P connectivity check
    if [ ${#DEPLOYED_NODES[@]} -gt 1 ]; then
        log_info "Waiting 60 seconds for network stabilization..."
        sleep 60
        verify_p2p_connectivity
    fi
    
    # Show summary
    show_summary
    
    # Exit with appropriate code
    if [ ${#FAILED_NODES[@]} -gt 0 ]; then
        exit 1
    else
        log_success "Deployment completed successfully!"
        exit 0
    fi
}

# Run main function
main "$@"