#!/bin/bash
set -euo pipefail

# QuDAG Testnet Cleanup Script
# This script safely tears down the QuDAG testnet deployment

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
BACKUP_DIR="$PROJECT_ROOT/backups/$(date +%Y%m%d_%H%M%S)"

# Node configuration
declare -A NODES=(
    ["node1"]="toronto:yyz:qudag-testnet-node1"
    ["node2"]="amsterdam:ams:qudag-testnet-node2"
    ["node3"]="singapore:sin:qudag-testnet-node3"
    ["node4"]="sanfrancisco:sjc:qudag-testnet-node4"
)

# Cleanup options
FORCE_MODE=false
BACKUP_DATA=true
REMOVE_VOLUMES=false
REMOVE_APPS=false
CLEANUP_LOCAL=false

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

confirm_action() {
    local message=$1
    
    if [ "$FORCE_MODE" = true ]; then
        return 0
    fi
    
    echo -ne "${YELLOW}$message (y/N): ${NC}"
    read -r response
    
    case "$response" in
        [yY][eE][sS]|[yY])
            return 0
            ;;
        *)
            return 1
            ;;
    esac
}

check_prerequisites() {
    log_info "Checking prerequisites..."
    
    if ! command -v flyctl &> /dev/null; then
        log_error "flyctl is not installed"
        exit 1
    fi
    
    if ! flyctl auth whoami &> /dev/null; then
        log_error "Not logged in to Fly.io"
        exit 1
    fi
}

create_backup_dir() {
    if [ "$BACKUP_DATA" = true ]; then
        log_info "Creating backup directory..."
        mkdir -p "$BACKUP_DIR"
        log_success "Backup directory created: $BACKUP_DIR"
    fi
}

backup_node_data() {
    local app_name=$1
    
    if [ "$BACKUP_DATA" != true ]; then
        return
    fi
    
    log_info "Backing up data from $app_name..."
    
    # Create node backup directory
    local node_backup_dir="$BACKUP_DIR/$app_name"
    mkdir -p "$node_backup_dir"
    
    # Save app configuration
    flyctl config show -a "$app_name" > "$node_backup_dir/config.toml" 2>/dev/null || true
    
    # Save secrets (names only, not values)
    flyctl secrets list -a "$app_name" > "$node_backup_dir/secrets.txt" 2>/dev/null || true
    
    # Get recent logs
    flyctl logs -a "$app_name" --no-tail -n 1000 > "$node_backup_dir/logs.txt" 2>/dev/null || true
    
    # Get app info
    flyctl info -a "$app_name" --json > "$node_backup_dir/info.json" 2>/dev/null || true
    
    log_success "Backed up $app_name data"
}

scale_down_nodes() {
    log_info "Scaling down nodes..."
    
    for node in "${!NODES[@]}"; do
        IFS=':' read -r location region app_name <<< "${NODES[$node]}"
        
        log_info "Scaling down $app_name..."
        
        if flyctl scale count 0 -a "$app_name" --yes 2>/dev/null; then
            log_success "Scaled down $app_name"
        else
            log_warning "Could not scale down $app_name (may already be stopped)"
        fi
    done
}

remove_volumes() {
    if [ "$REMOVE_VOLUMES" != true ]; then
        log_info "Skipping volume removal (use --remove-volumes to delete)"
        return
    fi
    
    log_warning "Removing persistent volumes..."
    
    for node in "${!NODES[@]}"; do
        IFS=':' read -r location region app_name <<< "${NODES[$node]}"
        local volume_name="qudag_data_$node"
        
        log_info "Removing volume $volume_name from $app_name..."
        
        # Get volume ID
        local volume_id=$(flyctl volumes list -a "$app_name" --json 2>/dev/null | \
            jq -r ".[] | select(.Name == \"$volume_name\") | .ID" || echo "")
        
        if [ -n "$volume_id" ]; then
            if flyctl volumes destroy "$volume_id" -a "$app_name" --yes 2>/dev/null; then
                log_success "Removed volume $volume_name"
            else
                log_error "Failed to remove volume $volume_name"
            fi
        else
            log_warning "Volume $volume_name not found"
        fi
    done
}

remove_apps() {
    if [ "$REMOVE_APPS" != true ]; then
        log_info "Skipping app removal (use --remove-apps to delete)"
        return
    fi
    
    log_warning "Removing Fly.io applications..."
    
    for node in "${!NODES[@]}"; do
        IFS=':' read -r location region app_name <<< "${NODES[$node]}"
        
        log_info "Removing app $app_name..."
        
        if flyctl apps destroy "$app_name" --yes 2>/dev/null; then
            log_success "Removed app $app_name"
        else
            log_warning "Could not remove $app_name (may not exist)"
        fi
    done
}

cleanup_local() {
    if [ "$CLEANUP_LOCAL" != true ]; then
        return
    fi
    
    log_info "Cleaning up local Docker resources..."
    
    # Stop and remove containers
    if command -v docker-compose &> /dev/null; then
        cd "$PROJECT_ROOT"
        docker-compose down -v --remove-orphans 2>/dev/null || true
    fi
    
    # Remove local secrets (with backup)
    if [ -d "$PROJECT_ROOT/.secrets" ]; then
        if [ "$BACKUP_DATA" = true ]; then
            cp -r "$PROJECT_ROOT/.secrets" "$BACKUP_DIR/"
        fi
        rm -rf "$PROJECT_ROOT/.secrets"
        log_success "Removed local secrets directory"
    fi
}

print_summary() {
    echo
    echo "========================================="
    echo "QuDAG Testnet Cleanup Summary"
    echo "========================================="
    echo
    
    if [ "$BACKUP_DATA" = true ]; then
        echo "Backups saved to: $BACKUP_DIR"
        echo
    fi
    
    echo "Actions performed:"
    echo "  ✓ Scaled down all nodes"
    
    if [ "$REMOVE_VOLUMES" = true ]; then
        echo "  ✓ Removed persistent volumes"
    fi
    
    if [ "$REMOVE_APPS" = true ]; then
        echo "  ✓ Removed Fly.io applications"
    fi
    
    if [ "$CLEANUP_LOCAL" = true ]; then
        echo "  ✓ Cleaned up local resources"
    fi
    
    echo
    echo "To redeploy the testnet, run: ./deployment.sh"
    echo
}

show_usage() {
    echo "Usage: $0 [OPTIONS]"
    echo
    echo "Options:"
    echo "  -f, --force           Skip confirmation prompts"
    echo "  -n, --no-backup       Skip data backup"
    echo "  -v, --remove-volumes  Remove persistent volumes"
    echo "  -a, --remove-apps     Remove Fly.io applications"
    echo "  -l, --local           Clean up local Docker resources"
    echo "  --all                 Remove everything (volumes, apps, local)"
    echo "  -h, --help            Show this help message"
    echo
    echo "Examples:"
    echo "  $0                    # Safe cleanup (scale down only)"
    echo "  $0 -v                 # Remove volumes too"
    echo "  $0 -a -v              # Remove apps and volumes"
    echo "  $0 --all -f           # Force remove everything"
    echo
    echo "WARNING: Data removal is permanent! Always backup first."
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -f|--force)
            FORCE_MODE=true
            shift
            ;;
        -n|--no-backup)
            BACKUP_DATA=false
            shift
            ;;
        -v|--remove-volumes)
            REMOVE_VOLUMES=true
            shift
            ;;
        -a|--remove-apps)
            REMOVE_APPS=true
            shift
            ;;
        -l|--local)
            CLEANUP_LOCAL=true
            shift
            ;;
        --all)
            REMOVE_VOLUMES=true
            REMOVE_APPS=true
            CLEANUP_LOCAL=true
            shift
            ;;
        -h|--help)
            show_usage
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            show_usage
            exit 1
            ;;
    esac
done

# Main execution
main() {
    log_info "Starting QuDAG Testnet cleanup..."
    
    # Show warning
    if [ "$REMOVE_APPS" = true ] || [ "$REMOVE_VOLUMES" = true ]; then
        log_warning "This will permanently delete resources!"
        if ! confirm_action "Are you sure you want to continue?"; then
            log_info "Cleanup cancelled"
            exit 0
        fi
    fi
    
    check_prerequisites
    create_backup_dir
    
    # Backup data from each node
    if [ "$BACKUP_DATA" = true ]; then
        for node in "${!NODES[@]}"; do
            IFS=':' read -r location region app_name <<< "${NODES[$node]}"
            backup_node_data "$app_name"
        done
    fi
    
    # Perform cleanup
    scale_down_nodes
    remove_volumes
    remove_apps
    cleanup_local
    
    print_summary
    log_success "Cleanup completed!"
}

# Run main function
main