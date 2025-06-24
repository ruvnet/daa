#!/bin/bash
set -euo pipefail

# QuDAG Testnet Restore Script
# Restores node data from backup files

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
BACKUP_DIR="$PROJECT_ROOT/backups"

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
    
    log_success "Prerequisites check passed"
}

validate_backup_file() {
    local backup_file="$1"
    
    if [ ! -f "$backup_file" ]; then
        log_error "Backup file does not exist: $backup_file"
        return 1
    fi
    
    # Check if file is encrypted
    if [[ "$backup_file" == *.gpg ]]; then
        if ! command -v gpg &> /dev/null; then
            log_error "gpg is not installed. Please install it to decrypt backup files."
            return 1
        fi
        
        log_info "Backup file is encrypted and will be decrypted during restore"
    fi
    
    # Check file size
    local size=$(du -h "$backup_file" | cut -f1)
    log_info "Backup file size: $size"
    
    return 0
}

validate_node() {
    local node="$1"
    
    if [ -z "${NODES[$node]:-}" ]; then
        log_error "Invalid node: $node"
        log_info "Valid nodes: ${!NODES[*]}"
        return 1
    fi
    
    return 0
}

stop_node() {
    local app_name="$1"
    
    log_info "Stopping $app_name for restore..."
    
    # Scale down to 0 instances
    if ! flyctl scale count 0 -a "$app_name" --yes; then
        log_warning "Failed to scale down $app_name, continuing anyway"
    fi
    
    # Wait a moment for the scale down to take effect
    sleep 5
    
    log_success "Stopped $app_name"
}

start_node() {
    local app_name="$1"
    
    log_info "Starting $app_name after restore..."
    
    # Scale back up to 1 instance
    if ! flyctl scale count 1 -a "$app_name" --yes; then
        log_error "Failed to start $app_name"
        return 1
    fi
    
    log_success "Started $app_name"
    return 0
}

decrypt_backup() {
    local encrypted_file="$1"
    local decrypted_file="$2"
    
    log_info "Decrypting backup file..."
    
    if ! gpg --decrypt --output "$decrypted_file" "$encrypted_file"; then
        log_error "Failed to decrypt backup file"
        return 1
    fi
    
    log_success "Backup file decrypted"
    return 0
}

restore_node_data() {
    local app_name="$1"
    local backup_file="$2"
    local node="$3"
    
    log_info "Restoring data to $app_name from $backup_file..."
    
    # Check if app exists
    if ! flyctl status -a "$app_name" &>/dev/null; then
        log_error "App $app_name does not exist. Please deploy it first."
        return 1
    fi
    
    # Prepare backup file
    local restore_file="$backup_file"
    local temp_file=""
    
    # Decrypt if needed
    if [[ "$backup_file" == *.gpg ]]; then
        temp_file="/tmp/$(basename "$backup_file" .gpg)"
        if ! decrypt_backup "$backup_file" "$temp_file"; then
            return 1
        fi
        restore_file="$temp_file"
    fi
    
    # Stop the node
    stop_node "$app_name"
    
    # Upload backup file
    log_info "Uploading backup file to $app_name..."
    if ! flyctl ssh sftp put -a "$app_name" "$restore_file" /tmp/restore_backup.tar.gz; then
        log_error "Failed to upload backup file"
        # Clean up temp file
        [ -n "$temp_file" ] && rm -f "$temp_file"
        return 1
    fi
    
    # Clear existing data directory (with confirmation)
    log_warning "This will replace ALL existing data in $app_name"
    read -p "Continue? (y/N) " -n 1 -r
    echo
    
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        log_info "Restore cancelled"
        # Clean up temp file
        [ -n "$temp_file" ] && rm -f "$temp_file"
        return 1
    fi
    
    # Extract backup on the node\n    log_info \"Extracting backup on $app_name...\"\n    if ! flyctl ssh console -a \"$app_name\" -C \"cd /data && rm -rf qudag/* && tar -xzf /tmp/restore_backup.tar.gz\"; then\n        log_error \"Failed to extract backup on $app_name\"\n        # Clean up temp file\n        [ -n \"$temp_file\" ] && rm -f \"$temp_file\"\n        return 1\n    fi\n    \n    # Clean up remote backup file\n    flyctl ssh console -a \"$app_name\" -C \"rm -f /tmp/restore_backup.tar.gz\" 2>/dev/null || true\n    \n    # Clean up local temp file\n    [ -n \"$temp_file\" ] && rm -f \"$temp_file\"\n    \n    # Start the node\n    if ! start_node \"$app_name\"; then\n        return 1\n    fi\n    \n    # Wait for health check\n    log_info \"Waiting for $app_name to become healthy...\"\n    local attempts=0\n    local max_attempts=30\n    \n    while [ $attempts -lt $max_attempts ]; do\n        if curl -sf \"https://$app_name.fly.dev/health\" &>/dev/null; then\n            log_success \"$app_name is healthy\"\n            break\n        fi\n        \n        sleep 10\n        ((attempts++))\n        \n        if [ $attempts -eq $max_attempts ]; then\n            log_warning \"$app_name did not become healthy within timeout\"\n            log_info \"Check logs with: flyctl logs -a $app_name\"\n            return 1\n        fi\n        \n        echo -n \".\"\n    done\n    \n    log_success \"Successfully restored $app_name from backup\"\n    return 0\n}\n\nlist_available_backups() {\n    echo\n    echo \"Available backup files:\"\n    echo \"========================\"\n    \n    if [ ! -d \"$BACKUP_DIR\" ] || [ -z \"$(ls -A \"$BACKUP_DIR\" 2>/dev/null)\" ]; then\n        echo \"No backup files found in $BACKUP_DIR\"\n        return\n    fi\n    \n    local count=0\n    \n    while IFS= read -r -d '' backup_file; do\n        local filename=$(basename \"$backup_file\")\n        local size=$(du -h \"$backup_file\" | cut -f1)\n        local date=$(stat -c %y \"$backup_file\" | cut -d' ' -f1)\n        \n        echo \"  $filename ($size, $date)\"\n        ((count++))\n    done < <(find \"$BACKUP_DIR\" -name \"*.tar.gz*\" -type f -print0 | sort -z)\n    \n    echo\n    echo \"Total: $count backup files\"\n    echo\n}\n\nshow_help() {\n    cat << EOF\nQuDAG Testnet Restore Script\n\nUsage: $0 <node> <backup-file>\n       $0 --list\n\nArguments:\n  node          Node to restore (node1, node2, node3, node4)\n  backup-file   Path to backup file or filename in backups directory\n\nOptions:\n  --list        List available backup files\n  -h, --help    Show this help message\n\nExamples:\n  $0 node1 backups/node1_toronto_20240615_143022.tar.gz\n  $0 node2 node2_amsterdam_20240615_143025.tar.gz\n  $0 --list\n\nNotes:\n  - The target node will be stopped during restore\n  - All existing data will be replaced\n  - Encrypted backups (.gpg) will be decrypted automatically\n  - The node will be restarted after restore\n\nEOF\n}\n\n# Parse command line arguments\nif [ $# -eq 0 ]; then\n    show_help\n    exit 1\nfi\n\ncase \"$1\" in\n    --list)\n        list_available_backups\n        exit 0\n        ;;\n    -h|--help)\n        show_help\n        exit 0\n        ;;\nesac\n\nif [ $# -ne 2 ]; then\n    log_error \"Invalid number of arguments\"\n    show_help\n    exit 1\nfi\n\nNODE=\"$1\"\nBACKUP_FILE=\"$2\"\n\n# Resolve backup file path\nif [ ! -f \"$BACKUP_FILE\" ]; then\n    # Try in backup directory\n    local alt_path=\"$BACKUP_DIR/$BACKUP_FILE\"\n    if [ -f \"$alt_path\" ]; then\n        BACKUP_FILE=\"$alt_path\"\n    else\n        log_error \"Backup file not found: $BACKUP_FILE\"\n        log_info \"Use --list to see available backups\"\n        exit 1\n    fi\nfi\n\n# Main execution\nmain() {\n    log_info \"Starting restore for $NODE from $BACKUP_FILE...\"\n    \n    check_prerequisites\n    \n    if ! validate_node \"$NODE\"; then\n        exit 1\n    fi\n    \n    if ! validate_backup_file \"$BACKUP_FILE\"; then\n        exit 1\n    fi\n    \n    IFS=':' read -r location region app_name <<< \"${NODES[$NODE]}\"\n    \n    echo\n    log_warning \"RESTORE CONFIRMATION\"\n    echo \"====================\"\n    echo\n    echo \"Node: $NODE ($location)\"\n    echo \"App: $app_name\"\n    echo \"Backup: $(basename \"$BACKUP_FILE\")\"\n    echo \"Size: $(du -h \"$BACKUP_FILE\" | cut -f1)\"\n    echo\n    echo \"This will:\"\n    echo \"1. Stop the running node\"\n    echo \"2. Replace ALL existing data\"\n    echo \"3. Restart the node\"\n    echo\n    read -p \"Continue with restore? (y/N) \" -n 1 -r\n    echo\n    \n    if [[ ! $REPLY =~ ^[Yy]$ ]]; then\n        log_info \"Restore cancelled\"\n        exit 0\n    fi\n    \n    if restore_node_data \"$app_name\" \"$BACKUP_FILE\" \"$NODE\"; then\n        echo\n        log_success \"Restore completed successfully!\"\n        echo\n        echo \"Next steps:\"\n        echo \"1. Verify node health: flyctl status -a $app_name\"\n        echo \"2. Check logs: flyctl logs -a $app_name\"\n        echo \"3. Monitor with: ./scripts/monitor-nodes.sh\"\n        echo\n    else\n        log_error \"Restore failed!\"\n        echo\n        echo \"Troubleshooting:\"\n        echo \"1. Check app status: flyctl status -a $app_name\"\n        echo \"2. View logs: flyctl logs -a $app_name\"\n        echo \"3. Try manual start: flyctl scale count 1 -a $app_name\"\n        echo\n        exit 1\n    fi\n}\n\nmain