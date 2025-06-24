#!/bin/bash
set -euo pipefail

# QuDAG Testnet Cleanup Script
# Enhanced version with safe teardown and recovery options

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
WHITE='\033[1;37m'
NC='\033[0m'

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TESTNET_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
BACKUP_DIR="$TESTNET_ROOT/backups/cleanup-$(date +%Y%m%d-%H%M%S)"
LOGS_DIR="$TESTNET_ROOT/logs"
CLEANUP_LOG="$LOGS_DIR/cleanup-$(date +%Y%m%d-%H%M%S).log"

# Node configuration
declare -A NODES=(
    ["node1"]="toronto:yyz:qudag-testnet-node1:primary"
    ["node2"]="amsterdam:ams:qudag-testnet-node2:secondary"
    ["node3"]="singapore:sin:qudag-testnet-node3:secondary"
    ["node4"]="sanfrancisco:sjc:qudag-testnet-node4:secondary"
)

# Cleanup options
FORCE_MODE=false
BACKUP_DATA=true
REMOVE_VOLUMES=false
REMOVE_APPS=false
CLEANUP_LOCAL=false
CLEANUP_SECRETS=false
INTERACTIVE_MODE=true
DRY_RUN=false
PRESERVE_LOGS=true

# Safety settings
CONFIRMATION_REQUIRED=true
SAFETY_WAIT=5

# Create necessary directories
mkdir -p "$LOGS_DIR"

# Logging functions
log_to_file() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $*" >> "$CLEANUP_LOG"
}

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
    log_to_file "INFO: $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
    log_to_file "SUCCESS: $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
    log_to_file "WARNING: $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
    log_to_file "ERROR: $1"
}

log_step() {
    echo -e "${CYAN}[STEP]${NC} $1"
    log_to_file "STEP: $1"
}

# Enhanced confirmation with safety features
confirm_action() {
    local message=$1
    local danger_level=${2:-normal}  # normal, high, critical
    
    if [[ "$FORCE_MODE" == "true" ]]; then
        log_warning "Force mode enabled - skipping confirmation"
        return 0
    fi
    
    if [[ "$INTERACTIVE_MODE" == "false" ]]; then
        log_error "Non-interactive mode requires --force for dangerous operations"
        return 1
    fi
    
    # Color code based on danger level
    local color=$YELLOW
    case "$danger_level" in
        high) color=$RED ;;
        critical) 
            color=$RED
            echo -e "${RED}╔══════════════════════════════════════════════════════════════╗${NC}"
            echo -e "${RED}║                     CRITICAL WARNING                         ║${NC}"
            echo -e "${RED}║  This action will permanently delete data and resources!    ║${NC}"
            echo -e "${RED}╚══════════════════════════════════════════════════════════════╝${NC}"
            ;;
    esac
    
    echo -ne "${color}$message (y/N): ${NC}"
    read -r response
    
    case "$response" in
        [yY][eE][sS]|[yY])
            if [[ "$danger_level" == "critical" ]]; then
                echo -e "${RED}Final confirmation required. Type 'DELETE' to proceed:${NC}"
                read -r final_response
                if [[ "$final_response" != "DELETE" ]]; then
                    log_info "Cleanup cancelled by user"
                    return 1
                fi
                
                log_warning "Safety wait: ${SAFETY_WAIT}s (Ctrl+C to cancel)"
                sleep "$SAFETY_WAIT"
            fi
            return 0
            ;;
        *)
            return 1
            ;;
    esac
}

# Check prerequisites
check_prerequisites() {
    log_step "Checking prerequisites..."
    
    local missing_tools=()
    local required=("flyctl" "jq" "curl" "tar" "gzip")
    
    for tool in "${required[@]}"; do
        if ! command -v "$tool" &> /dev/null; then
            missing_tools+=("$tool")
        fi
    done
    
    if [[ ${#missing_tools[@]} -gt 0 ]]; then
        log_error "Missing required tools: ${missing_tools[*]}"
        exit 1
    fi
    
    if ! flyctl auth whoami &> /dev/null; then
        log_error "Not logged in to Fly.io"
        exit 1
    fi
    
    log_success "Prerequisites check passed"
}

# Create comprehensive backup
create_backup_dir() {
    if [[ "$BACKUP_DATA" != "true" ]]; then
        return
    fi
    
    log_step "Creating backup directory..."
    
    mkdir -p "$BACKUP_DIR"
    chmod 700 "$BACKUP_DIR"
    
    # Create backup manifest
    cat > "$BACKUP_DIR/manifest.json" <<EOF
{
    "backup_id": "$(uuidgen || date +%s)",
    "timestamp": "$(date -Iseconds)",
    "testnet_version": "$(git rev-parse HEAD 2>/dev/null || echo 'unknown')",
    "nodes": {
$(for node in "${!NODES[@]}"; do
    IFS=':' read -r location region app_name role <<< "${NODES[$node]}"
    echo "        \"$node\": {"
    echo "            \"app_name\": \"$app_name\","
    echo "            \"region\": \"$region\","
    echo "            \"role\": \"$role\""
    echo "        },"
done | sed '$ s/,$//')
    },
    "cleanup_options": {
        "remove_volumes": $REMOVE_VOLUMES,
        "remove_apps": $REMOVE_APPS,
        "cleanup_local": $CLEANUP_LOCAL,
        "cleanup_secrets": $CLEANUP_SECRETS
    }
}
EOF
    
    log_success "Backup directory created: $BACKUP_DIR"
}

# Backup node data with enhanced information
backup_node_data() {
    local node=$1
    IFS=':' read -r location region app_name role <<< "${NODES[$node]}"
    
    if [[ "$BACKUP_DATA" != "true" ]]; then
        return
    fi
    
    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "[DRY RUN] Would backup data from $app_name"
        return
    fi
    
    log_info "Backing up comprehensive data from $app_name..."
    
    local node_backup_dir="$BACKUP_DIR/$node"
    mkdir -p "$node_backup_dir"
    
    # Save deployment configuration
    if flyctl config show -a "$app_name" > "$node_backup_dir/fly.toml" 2>/dev/null; then
        log_success "Backed up Fly.io configuration for $app_name"
    fi
    
    # Save secrets (names only for security)
    if flyctl secrets list -a "$app_name" > "$node_backup_dir/secrets.txt" 2>/dev/null; then
        log_success "Backed up secrets list for $app_name"
    fi
    
    # Get comprehensive logs
    local log_files=("recent.log" "errors.log" "full.log")
    
    # Recent logs (last 1000 lines)
    flyctl logs -a "$app_name" --no-tail -n 1000 > "$node_backup_dir/recent.log" 2>/dev/null || true
    
    # Error logs only
    flyctl logs -a "$app_name" --no-tail -n 5000 2>/dev/null | grep -E "(ERROR|FATAL|PANIC)" > "$node_backup_dir/errors.log" || true
    
    # Full logs (limited to reasonable size)
    flyctl logs -a "$app_name" --no-tail -n 10000 > "$node_backup_dir/full.log" 2>/dev/null || true
    
    # Application information
    flyctl info -a "$app_name" --json > "$node_backup_dir/app_info.json" 2>/dev/null || true
    
    # Deployment history
    flyctl releases -a "$app_name" --json > "$node_backup_dir/releases.json" 2>/dev/null || true
    
    # Volume information
    flyctl volumes list -a "$app_name" --json > "$node_backup_dir/volumes.json" 2>/dev/null || true
    
    # IP addresses
    flyctl ips list -a "$app_name" --json > "$node_backup_dir/ips.json" 2>/dev/null || true
    
    # Metrics snapshot
    local metrics_url="https://$app_name.fly.dev:9090/metrics"
    if curl -sf --max-time 10 "$metrics_url" > "$node_backup_dir/metrics.txt" 2>/dev/null; then
        log_success "Backed up metrics snapshot for $app_name"
    fi
    
    # Health check data
    local health_url="https://$app_name.fly.dev/health"
    if curl -sf --max-time 10 -w "\nResponse-Time: %{time_total}s\nHTTP-Code: %{http_code}\n" \
        "$health_url" > "$node_backup_dir/health.txt" 2>/dev/null; then
        log_success "Backed up health check data for $app_name"
    fi
    
    # Create node summary
    cat > "$node_backup_dir/summary.txt" <<EOF
Node Backup Summary
==================
Node: $node
App Name: $app_name
Location: $location
Region: $region
Role: $role
Backup Time: $(date)
Backup Directory: $node_backup_dir

Files Backed Up:
$(ls -la "$node_backup_dir" | tail -n +2)

Total Size: $(du -sh "$node_backup_dir" | cut -f1)
EOF
    
    log_success "Comprehensive backup completed for $app_name"
}

# Graceful node shutdown
graceful_shutdown_node() {
    local node=$1
    IFS=':' read -r location region app_name role <<< "${NODES[$node]}"
    
    log_info "Gracefully shutting down $app_name..."
    
    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "[DRY RUN] Would gracefully shutdown $app_name"
        return
    fi
    
    # Try to send shutdown signal via API
    local shutdown_url="https://$app_name.fly.dev/admin/shutdown"
    if curl -sf --max-time 30 -X POST "$shutdown_url" &>/dev/null; then
        log_success "Sent graceful shutdown signal to $app_name"
        sleep 10  # Wait for graceful shutdown
    else
        log_warning "Could not send graceful shutdown signal to $app_name"
    fi
    
    # Scale down instances
    local current_count=$(flyctl scale show -a "$app_name" --json 2>/dev/null | \
        jq -r '.[] | select(.Process == "app") | .Count' || echo "0")
    
    if [[ "$current_count" -gt 0 ]]; then
        log_info "Scaling down $app_name from $current_count to 0 instances..."
        
        if flyctl scale count 0 -a "$app_name" --yes 2>&1 | tee -a "$CLEANUP_LOG"; then
            log_success "Scaled down $app_name"
            
            # Wait for instances to stop
            local wait_count=0
            while [[ $wait_count -lt 30 ]]; do
                local running=$(flyctl status -a "$app_name" --json 2>/dev/null | \
                    jq -r '.Instances | length' || echo "0")
                
                if [[ "$running" -eq 0 ]]; then
                    break
                fi
                
                sleep 2
                ((wait_count++))
            done
            
            if [[ $wait_count -ge 30 ]]; then
                log_warning "Timeout waiting for $app_name instances to stop"
            else
                log_success "All instances stopped for $app_name"
            fi
        else
            log_error "Failed to scale down $app_name"
        fi
    else
        log_warning "$app_name already scaled to 0"
    fi
}

# Scale down all nodes with coordination
scale_down_nodes() {
    log_step "Scaling down nodes with coordination..."
    
    # First, scale down secondary nodes
    for node in "${!NODES[@]}"; do
        IFS=':' read -r location region app_name role <<< "${NODES[$node]}"
        
        if [[ "$role" == "secondary" ]]; then
            graceful_shutdown_node "$node"
        fi
    done
    
    # Wait a bit for network to stabilize
    if [[ "$DRY_RUN" != "true" ]]; then
        log_info "Waiting for network to stabilize after secondary nodes shutdown..."
        sleep 10
    fi
    
    # Finally, scale down primary node
    for node in "${!NODES[@]}"; do
        IFS=':' read -r location region app_name role <<< "${NODES[$node]}"
        
        if [[ "$role" == "primary" ]]; then
            graceful_shutdown_node "$node"
        fi
    done
}

# Remove volumes with safety checks
remove_volumes() {
    if [[ "$REMOVE_VOLUMES" != "true" ]]; then
        log_info "Skipping volume removal (use --remove-volumes to delete)"
        return
    fi
    
    if ! confirm_action "Remove all persistent volumes? This will delete all node data!" "critical"; then
        log_info "Volume removal cancelled"
        return
    fi
    
    log_step "Removing persistent volumes..."
    
    for node in "${!NODES[@]}"; do
        IFS=':' read -r location region app_name role <<< "${NODES[$node]}"
        local volume_name="qudag_data_$node"
        
        if [[ "$DRY_RUN" == "true" ]]; then
            log_info "[DRY RUN] Would remove volume $volume_name from $app_name"
            continue
        fi
        
        log_info "Removing volume $volume_name from $app_name..."
        
        # Get all volumes for the app
        local volumes=$(flyctl volumes list -a "$app_name" --json 2>/dev/null || echo "[]")
        
        # Process each volume
        echo "$volumes" | jq -r '.[] | select(.Name | contains("qudag_data")) | .ID' | while read -r volume_id; do
            if [[ -n "$volume_id" ]]; then
                log_info "Removing volume ID: $volume_id"
                
                if flyctl volumes destroy "$volume_id" -a "$app_name" --yes 2>&1 | tee -a "$CLEANUP_LOG"; then
                    log_success "Removed volume $volume_id"
                else
                    log_error "Failed to remove volume $volume_id"
                fi
            fi
        done
    done
}

# Remove apps with safety checks
remove_apps() {
    if [[ "$REMOVE_APPS" != "true" ]]; then
        log_info "Skipping app removal (use --remove-apps to delete)"
        return
    fi
    
    if ! confirm_action "Remove all Fly.io applications? This action cannot be undone!" "critical"; then
        log_info "App removal cancelled"
        return
    fi
    
    log_step "Removing Fly.io applications..."
    
    for node in "${!NODES[@]}"; do
        IFS=':' read -r location region app_name role <<< "${NODES[$node]}"
        
        if [[ "$DRY_RUN" == "true" ]]; then
            log_info "[DRY RUN] Would remove app $app_name"
            continue
        fi
        
        log_info "Removing app $app_name..."
        
        # Check if app exists
        if ! flyctl apps list | grep -q "$app_name"; then
            log_warning "App $app_name does not exist"
            continue
        fi
        
        if flyctl apps destroy "$app_name" --yes 2>&1 | tee -a "$CLEANUP_LOG"; then
            log_success "Removed app $app_name"
        else
            log_error "Failed to remove app $app_name"
        fi
    done
}

# Cleanup local resources
cleanup_local() {
    if [[ "$CLEANUP_LOCAL" != "true" ]]; then
        return
    fi
    
    log_step "Cleaning up local resources..."
    
    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "[DRY RUN] Would cleanup local Docker and project resources"
        return
    fi
    
    # Stop Docker containers
    if command -v docker-compose &> /dev/null; then
        cd "$PROJECT_ROOT/docs/testnet_deployment" || true
        
        if [[ -f "docker-compose.yml" ]]; then
            log_info "Stopping Docker containers..."
            docker-compose down -v --remove-orphans 2>/dev/null || true
            log_success "Stopped Docker containers"
        fi
    fi
    
    # Clean up Docker images
    if confirm_action "Remove QuDAG Docker images?" "normal"; then
        docker images | grep "qudag" | awk '{print $3}' | xargs docker rmi -f 2>/dev/null || true
        log_success "Removed QuDAG Docker images"
    fi
    
    # Clean up build artifacts
    if [[ -d "$PROJECT_ROOT/target" ]]; then
        if confirm_action "Remove Rust build artifacts?" "normal"; then
            rm -rf "$PROJECT_ROOT/target"
            log_success "Removed build artifacts"
        fi
    fi
}

# Cleanup secrets with backup
cleanup_secrets() {
    if [[ "$CLEANUP_SECRETS" != "true" ]]; then
        return
    fi
    
    if ! confirm_action "Remove local secrets? (A backup will be created)" "high"; then
        log_info "Secrets cleanup cancelled"
        return
    fi
    
    log_step "Cleaning up local secrets..."
    
    local secrets_dir="$TESTNET_ROOT/.secrets"
    
    if [[ ! -d "$secrets_dir" ]]; then
        log_info "No secrets directory found"
        return
    fi
    
    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "[DRY RUN] Would cleanup secrets in $secrets_dir"
        return
    fi
    
    # Backup secrets first
    if [[ "$BACKUP_DATA" == "true" ]]; then
        log_info "Backing up secrets before cleanup..."
        cp -r "$secrets_dir" "$BACKUP_DIR/secrets-backup"
        log_success "Secrets backed up to $BACKUP_DIR/secrets-backup"
    fi
    
    # Remove secrets
    rm -rf "$secrets_dir"
    log_success "Removed local secrets directory"
}

# Generate cleanup report
generate_cleanup_report() {
    local report_file="$LOGS_DIR/cleanup-report-$(date +%Y%m%d-%H%M%S).json"
    
    log_step "Generating cleanup report..."
    
    local end_time=$(date -Iseconds)
    local duration=$(($(date +%s) - $(date -d "$start_time" +%s) || 0))
    
    cat > "$report_file" <<EOF
{
    "cleanup_id": "$(uuidgen || date +%s)",
    "start_time": "$start_time",
    "end_time": "$end_time", 
    "duration_seconds": $duration,
    "options": {
        "force_mode": $FORCE_MODE,
        "backup_data": $BACKUP_DATA,
        "remove_volumes": $REMOVE_VOLUMES,
        "remove_apps": $REMOVE_APPS,
        "cleanup_local": $CLEANUP_LOCAL,
        "cleanup_secrets": $CLEANUP_SECRETS,
        "dry_run": $DRY_RUN
    },
    "backup_location": "${BACKUP_DIR}",
    "nodes_processed": ${#NODES[@]},
    "cleanup_log": "$CLEANUP_LOG"
}
EOF
    
    log_success "Cleanup report saved to: $report_file"
}

# Print comprehensive summary
print_summary() {
    echo
    echo -e "${CYAN}╔════════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${CYAN}║              QuDAG Testnet Cleanup Summary                     ║${NC}"
    echo -e "${CYAN}╚════════════════════════════════════════════════════════════════╝${NC}"
    echo
    
    local end_time=$(date)
    echo "Cleanup completed at: $end_time"
    echo "Total duration: $(($(date +%s) - $(date -d "$start_time" +%s) || 0)) seconds"
    echo "Log file: $CLEANUP_LOG"
    
    if [[ "$BACKUP_DATA" == "true" ]]; then
        echo "Backup location: $BACKUP_DIR"
        echo "Backup size: $(du -sh "$BACKUP_DIR" 2>/dev/null | cut -f1 || echo 'unknown')"
    fi
    
    echo
    echo -e "${YELLOW}Actions performed:${NC}"
    echo "─────────────────────────────────────────────────────────────────"
    echo "✓ Gracefully shut down all nodes"
    
    if [[ "$REMOVE_VOLUMES" == "true" ]]; then
        echo "✓ Removed persistent volumes"
    else
        echo "○ Preserved persistent volumes"
    fi
    
    if [[ "$REMOVE_APPS" == "true" ]]; then
        echo "✓ Removed Fly.io applications"
    else
        echo "○ Preserved Fly.io applications"
    fi
    
    if [[ "$CLEANUP_LOCAL" == "true" ]]; then
        echo "✓ Cleaned up local resources"
    else
        echo "○ Preserved local resources"
    fi
    
    if [[ "$CLEANUP_SECRETS" == "true" ]]; then
        echo "✓ Cleaned up local secrets"
    else
        echo "○ Preserved local secrets"
    fi
    
    echo
    echo -e "${YELLOW}Recovery options:${NC}"
    echo "─────────────────────────────────────────────────────────────────"
    
    if [[ "$REMOVE_APPS" != "true" ]]; then
        echo "• Scale up nodes: flyctl scale count 1 -a <app-name>"
        echo "• Check status: ./monitor-nodes.sh"
    fi
    
    if [[ "$BACKUP_DATA" == "true" ]]; then
        echo "• Restore from backup: Use files in $BACKUP_DIR"
    fi
    
    if [[ "$REMOVE_APPS" == "true" ]]; then
        echo "• Full redeploy: ./deployment.sh"
    fi
    
    echo
    echo -e "${YELLOW}Useful commands:${NC}"
    echo "─────────────────────────────────────────────────────────────────"
    echo "• View cleanup log: cat $CLEANUP_LOG"
    echo "• Check Fly.io apps: flyctl apps list"
    echo "• Redeploy testnet: ./deployment.sh"
    echo
}

# Show usage
show_usage() {
    echo "Usage: $0 [OPTIONS]"
    echo
    echo "Options:"
    echo "  -f, --force           Skip confirmation prompts"
    echo "  -n, --no-backup       Skip data backup"
    echo "  -v, --remove-volumes  Remove persistent volumes"
    echo "  -a, --remove-apps     Remove Fly.io applications"
    echo "  -l, --local           Clean up local Docker resources"
    echo "  -s, --secrets         Clean up local secrets"
    echo "  --all                 Remove everything (volumes, apps, local, secrets)"
    echo "  --dry-run             Show what would be done without executing"
    echo "  --non-interactive     Run without user prompts (requires --force)"
    echo "  --no-logs             Don't preserve logs during cleanup"
    echo "  -h, --help            Show this help message"
    echo
    echo "Safety levels:"
    echo "  Safe:      $0                    # Scale down only"
    echo "  Medium:    $0 -v                # Remove volumes too"
    echo "  High:      $0 -a -v             # Remove apps and volumes"
    echo "  Complete:  $0 --all -f          # Remove everything"
    echo
    echo "Examples:"
    echo "  $0                     # Safe cleanup (scale down only)"
    echo "  $0 --dry-run --all     # See what complete cleanup would do"
    echo "  $0 -v -l               # Remove volumes and local resources"
    echo "  $0 --all --force       # Complete cleanup without prompts"
    echo
    echo "Environment Variables:"
    echo "  SAFETY_WAIT           Wait time for critical operations (default: 5s)"
    echo "  BACKUP_DATA           Enable/disable backups (default: true)"
    echo
    echo -e "${RED}WARNING: Data removal is permanent! Always backup first.${NC}"
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
        -s|--secrets)
            CLEANUP_SECRETS=true
            shift
            ;;
        --all)
            REMOVE_VOLUMES=true
            REMOVE_APPS=true
            CLEANUP_LOCAL=true
            CLEANUP_SECRETS=true
            shift
            ;;
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --non-interactive)
            INTERACTIVE_MODE=false
            shift
            ;;
        --no-logs)
            PRESERVE_LOGS=false
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
    local start_time=$(date -Iseconds)
    
    if [[ "$DRY_RUN" == "true" ]]; then
        log_info "Starting QuDAG Testnet cleanup (DRY RUN)..."
    else
        log_info "Starting QuDAG Testnet cleanup..."
    fi
    
    log_info "Cleanup mode: ${FORCE_MODE:+Force }${DRY_RUN:+Dry-run }${INTERACTIVE_MODE:+Interactive}"
    
    # Show warning for destructive operations
    if [[ "$REMOVE_APPS" == "true" ]] || [[ "$REMOVE_VOLUMES" == "true" ]] || [[ "$CLEANUP_LOCAL" == "true" ]]; then
        if [[ "$DRY_RUN" != "true" ]]; then
            log_warning "This will permanently delete resources!"
            
            if [[ "$INTERACTIVE_MODE" == "true" ]] && ! confirm_action "Are you sure you want to continue?" "high"; then
                log_info "Cleanup cancelled by user"
                exit 0
            fi
        fi
    fi
    
    # Prerequisites
    check_prerequisites
    
    # Create backup
    create_backup_dir
    
    # Backup data from each node
    if [[ "$BACKUP_DATA" == "true" ]]; then
        for node in "${!NODES[@]}"; do
            backup_node_data "$node"
        done
    fi
    
    # Perform cleanup operations
    scale_down_nodes
    remove_volumes
    remove_apps
    cleanup_local
    cleanup_secrets
    
    # Generate report
    generate_cleanup_report
    
    # Show summary
    print_summary
    
    if [[ "$DRY_RUN" == "true" ]]; then
        log_success "Dry run completed!"
    else
        log_success "Cleanup completed!"
    fi
}

# Record start time
start_time=$(date -Iseconds)

# Run main function
main