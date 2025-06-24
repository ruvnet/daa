#!/bin/bash
set -euo pipefail

# QuDAG Testnet Prerequisites Check Script
# Verifies all required tools and configurations are in place

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Tracking
ISSUES_FOUND=0

# Logging functions
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
    ISSUES_FOUND=$((ISSUES_FOUND + 1))
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
    ISSUES_FOUND=$((ISSUES_FOUND + 1))
}

log_success() {
    echo -e "${GREEN}[✓]${NC} $1"
}

log_fail() {
    echo -e "${RED}[✗]${NC} $1"
}

log_section() {
    echo -e "\n${BLUE}========== $1 ==========${NC}\n"
}

# Check Docker
check_docker() {
    log_section "Docker Prerequisites"
    
    if command -v docker &> /dev/null; then
        local docker_version=$(docker --version | cut -d' ' -f3 | tr -d ',')
        log_success "Docker installed: $docker_version"
        
        # Check if Docker daemon is running
        if docker info &> /dev/null; then
            log_success "Docker daemon is running"
        else
            log_error "Docker daemon is not running. Start Docker service."
        fi
    else
        log_error "Docker not installed. Install from: https://docs.docker.com/get-docker/"
    fi
    
    # Check docker-compose
    if command -v docker-compose &> /dev/null; then
        local compose_version=$(docker-compose --version | cut -d' ' -f3 | tr -d ',')
        log_success "Docker Compose installed: $compose_version"
    else
        log_error "Docker Compose not installed. Install from: https://docs.docker.com/compose/install/"
    fi
}

# Check Fly.io CLI
check_fly() {
    log_section "Fly.io Prerequisites"
    
    if command -v fly &> /dev/null; then
        local fly_version=$(fly version | head -1)
        log_success "Fly CLI installed: $fly_version"
        
        # Check if logged in
        if fly auth whoami &> /dev/null; then
            local fly_user=$(fly auth whoami)
            log_success "Logged in to Fly.io as: $fly_user"
        else
            log_warn "Not logged in to Fly.io. Run: fly auth login"
        fi
    else
        log_warn "Fly CLI not installed. Install from: https://fly.io/docs/hands-on/install-flyctl/"
        log_info "Note: Fly CLI is only needed for production deployment, not local testing"
    fi
}

# Check required tools
check_tools() {
    log_section "Required Tools"
    
    # Check curl
    if command -v curl &> /dev/null; then
        log_success "curl installed"
    else
        log_error "curl not installed. Install with: apt-get install curl"
    fi
    
    # Check jq
    if command -v jq &> /dev/null; then
        log_success "jq installed"
    else
        log_warn "jq not installed. Install with: apt-get install jq"
        log_info "Note: jq is optional but recommended for JSON parsing"
    fi
    
    # Check openssl
    if command -v openssl &> /dev/null; then
        log_success "openssl installed"
    else
        log_warn "openssl not installed. Install with: apt-get install openssl"
        log_info "Note: openssl is optional, used for TLS certificate verification"
    fi
}

# Check file structure
check_files() {
    log_section "File Structure"
    
    # Check Dockerfile
    if [ -f "Dockerfile" ]; then
        log_success "Dockerfile exists"
    else
        log_error "Dockerfile not found in current directory"
    fi
    
    # Check docker-compose.yml
    if [ -f "docker-compose.yml" ]; then
        log_success "docker-compose.yml exists"
    else
        log_error "docker-compose.yml not found"
    fi
    
    # Check node configurations
    local missing_configs=0
    for i in {1..4}; do
        if [ -f "configs/node$i.toml" ]; then
            log_success "configs/node$i.toml exists"
        else
            log_error "configs/node$i.toml not found"
            missing_configs=$((missing_configs + 1))
        fi
    done
    
    if [ $missing_configs -eq 0 ]; then
        log_success "All node configuration files present"
    fi
    
    # Check fly.toml files
    local missing_fly_configs=0
    for i in {1..4}; do
        if [ -f "nodes/fly.node$i.toml" ]; then
            log_success "nodes/fly.node$i.toml exists"
        else
            log_error "nodes/fly.node$i.toml not found"
            missing_fly_configs=$((missing_fly_configs + 1))
        fi
    done
    
    if [ $missing_fly_configs -eq 0 ]; then
        log_success "All Fly.io configuration files present"
    fi
    
    # Check scripts
    local scripts=("test-local.sh" "deploy-fixed.sh" "verify-deployment.sh" "update-fly-configs.sh")
    for script in "${scripts[@]}"; do
        if [ -f "$script" ]; then
            if [ -x "$script" ]; then
                log_success "$script exists and is executable"
            else
                log_warn "$script exists but is not executable. Run: chmod +x $script"
            fi
        else
            log_error "$script not found"
        fi
    done
}

# Check network
check_network() {
    log_section "Network Connectivity"
    
    # Check Docker Hub connectivity
    if curl -sf https://hub.docker.com &> /dev/null; then
        log_success "Docker Hub accessible"
    else
        log_warn "Cannot reach Docker Hub. May have issues pulling base images."
    fi
    
    # Check Fly.io connectivity
    if curl -sf https://fly.io &> /dev/null; then
        log_success "Fly.io accessible"
    else
        log_warn "Cannot reach Fly.io. May have issues with deployment."
    fi
}

# Check system resources
check_resources() {
    log_section "System Resources"
    
    # Check available disk space
    local available_space=$(df -BG . | awk 'NR==2 {print $4}' | tr -d 'G')
    if [ "$available_space" -ge 10 ]; then
        log_success "Sufficient disk space: ${available_space}GB available"
    else
        log_warn "Low disk space: ${available_space}GB available. Recommend at least 10GB."
    fi
    
    # Check available memory
    if command -v free &> /dev/null; then
        local available_mem=$(free -g | awk 'NR==2 {print $7}')
        if [ "$available_mem" -ge 4 ]; then
            log_success "Sufficient memory: ${available_mem}GB available"
        else
            log_warn "Low memory: ${available_mem}GB available. Recommend at least 4GB for local testing."
        fi
    fi
}

# Main execution
main() {
    log_info "QuDAG Testnet Prerequisites Check"
    log_info "Working directory: $(pwd)"
    
    # Run all checks
    check_docker
    check_fly
    check_tools
    check_files
    check_network
    check_resources
    
    # Summary
    log_section "Summary"
    
    if [ $ISSUES_FOUND -eq 0 ]; then
        log_success "All prerequisites satisfied! You're ready to run the testnet."
        echo
        log_info "Next steps:"
        log_info "1. For local testing: ./test-local.sh"
        log_info "2. For deployment: ./deploy-fixed.sh"
        log_info "3. To verify deployment: ./verify-deployment.sh"
        exit 0
    else
        log_error "Found $ISSUES_FOUND issue(s). Please address them before proceeding."
        echo
        log_info "Critical issues must be resolved for:"
        log_info "- Local testing: Docker, docker-compose, Dockerfile, docker-compose.yml"
        log_info "- Deployment: Fly CLI, fly auth login, fly.node*.toml files"
        exit 1
    fi
}

# Run main function
main "$@"