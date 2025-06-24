#!/bin/bash
set -euo pipefail

# QuDAG Testnet Backup Script
# Creates backups of all node data and configurations

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

# Settings
COMPRESS=true
ENCRYPT=false
RETENTION_DAYS=7
INCLUDE_SECRETS=false

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
    
    # Check for encryption tools if needed
    if [ "$ENCRYPT" = true ] && ! command -v gpg &> /dev/null; then
        log_error "gpg is not installed. Please install it for encryption support."
        exit 1
    fi
    
    log_success "Prerequisites check passed"
}

create_backup_dir() {
    mkdir -p "$BACKUP_DIR"
    chmod 755 "$BACKUP_DIR"
}

backup_node_data() {
    local app_name="$1"
    local node="$2"
    local location="$3"
    
    log_info "Backing up $app_name ($location)..."
    
    # Check if app exists and is running
    if ! flyctl status -a "$app_name" &>/dev/null; then
        log_warning "App $app_name not found or not running, skipping"
        return 1
    fi
    
    local timestamp=$(date +%Y%m%d_%H%M%S)
    local backup_name="${node}_${location}_${timestamp}"
    local backup_file="$BACKUP_DIR/${backup_name}.tar.gz"
    
    # Create remote backup
    log_info "Creating backup archive on $app_name..."
    if ! flyctl ssh console -a "$app_name" -C "cd /data && tar -czf /tmp/node_backup.tar.gz qudag/" 2>/dev/null; then
        log_error "Failed to create backup archive on $app_name"
        return 1
    fi
    
    # Download backup
    log_info "Downloading backup from $app_name..."
    if ! flyctl ssh sftp get -a "$app_name" /tmp/node_backup.tar.gz "$backup_file" 2>/dev/null; then
        log_error "Failed to download backup from $app_name"
        return 1
    fi
    
    # Clean up remote backup file
    flyctl ssh console -a "$app_name" -C "rm -f /tmp/node_backup.tar.gz" 2>/dev/null || true
    
    # Get file size
    local size=$(du -h "$backup_file" | cut -f1)
    
    # Encrypt if requested
    if [ "$ENCRYPT" = true ]; then
        log_info "Encrypting backup..."
        gpg --symmetric --cipher-algo AES256 "$backup_file" && rm "$backup_file"
        backup_file="${backup_file}.gpg"
    fi
    
    log_success "Backup created: $backup_file ($size)"
    return 0
}

backup_configurations() {
    local timestamp=$(date +%Y%m%d_%H%M%S)
    local config_backup="$BACKUP_DIR/configurations_${timestamp}.tar.gz"
    
    log_info "Backing up configurations..."
    
    # Create configuration backup
    cd "$PROJECT_ROOT"
    tar -czf "$config_backup" \
        configs/ \
        nodes/ \
        scripts/ \
        .env.example \
        docker-compose.yml \
        Dockerfile \
        README.md \
        DEPLOYMENT_FILES.md \
        2>/dev/null || true
    
    if [ -f "$config_backup" ]; then
        local size=$(du -h "$config_backup" | cut -f1)
        log_success "Configuration backup created: $config_backup ($size)"
    else
        log_error "Failed to create configuration backup"
    fi
}

backup_secrets() {
    if [ "$INCLUDE_SECRETS" = false ]; then
        return
    fi
    
    if [ ! -d "$PROJECT_ROOT/.secrets" ]; then
        log_warning "No secrets directory found, skipping"
        return
    fi
    
    local timestamp=$(date +%Y%m%d_%H%M%S)
    local secrets_backup="$BACKUP_DIR/secrets_${timestamp}.tar.gz.gpg"
    
    log_info "Backing up secrets (encrypted)..."
    
    # Always encrypt secrets
    cd "$PROJECT_ROOT"
    tar -czf - .secrets/ | gpg --symmetric --cipher-algo AES256 --output "$secrets_backup" || {
        log_error "Failed to create encrypted secrets backup"
        return
    }
    
    local size=$(du -h "$secrets_backup" | cut -f1)
    log_success "Encrypted secrets backup created: $secrets_backup ($size)"
}

cleanup_old_backups() {
    if [ "$RETENTION_DAYS" -le 0 ]; then
        return
    fi
    
    log_info "Cleaning up backups older than $RETENTION_DAYS days..."
    
    local deleted_count=0
    
    # Find and remove old backups
    while IFS= read -r -d '' backup_file; do
        rm "$backup_file"
        ((deleted_count++))
    done < <(find "$BACKUP_DIR" -name "*.tar.gz*" -mtime +"$RETENTION_DAYS" -print0 2>/dev/null)
    
    if [ "$deleted_count" -gt 0 ]; then
        log_success "Cleaned up $deleted_count old backup files"
    else
        log_info "No old backup files to clean up"
    fi
}

generate_backup_manifest() {
    local manifest_file="$BACKUP_DIR/backup_manifest_$(date +%Y%m%d_%H%M%S).json"
    
    log_info "Generating backup manifest..."
    
    cat > "$manifest_file" << EOF
{
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "backup_location": "$BACKUP_DIR",
  "files": [
EOF
    
    local first=true
    while IFS= read -r -d '' backup_file; do
        if [ "$first" = false ]; then
            echo "," >> "$manifest_file"
        fi
        first=false
        
        local filename=$(basename "$backup_file")
        local size=$(du -b "$backup_file" | cut -f1)
        local checksum=$(sha256sum "$backup_file" | cut -d' ' -f1)
        
        cat >> "$manifest_file" << EOF
    {
      "filename": "$filename",
      "size_bytes": $size,
      "sha256": "$checksum",
      "created": "$(stat -c %Y "$backup_file")"
    }EOF
    done < <(find "$BACKUP_DIR" -name "*.tar.gz*" -type f -print0 2>/dev/null | head -20)
    
    cat >> "$manifest_file" << EOF

  ],
  "retention_days": $RETENTION_DAYS,
  "encrypted": $ENCRYPT,
  "includes_secrets": $INCLUDE_SECRETS
}
EOF
    
    log_success "Backup manifest created: $manifest_file"
}

print_summary() {
    echo
    echo "=========================================="
    echo "Backup Summary"
    echo "=========================================="
    echo
    
    local backup_count=0
    local total_size=0
    
    if [ -d "$BACKUP_DIR" ]; then
        while IFS= read -r -d '' backup_file; do
            local filename=$(basename "$backup_file")
            local size=$(du -b "$backup_file" | cut -f1)
            local size_human=$(du -h "$backup_file" | cut -f1)
            
            echo "✓ $filename ($size_human)"
            ((backup_count++))
            ((total_size += size))
        done < <(find "$BACKUP_DIR" -name "*.tar.gz*" -type f -print0 2>/dev/null | sort -z)
    fi
    
    echo
    echo "Total: $backup_count files"
    echo "Total size: $(numfmt --to=iec $total_size)"
    echo "Location: $BACKUP_DIR"
    
    if [ "$ENCRYPT" = true ]; then
        echo "Encryption: Enabled"
    fi
    
    echo
    echo "To restore a backup:"
    echo "  ./scripts/restore-node.sh <node> <backup-file>"
    echo
}

show_help() {
    cat << EOF
QuDAG Testnet Backup Script

Usage: $0 [OPTIONS]

Options:
  --no-compress       Don't compress backup files
  --encrypt           Encrypt backup files with GPG
  --include-secrets   Include secrets in backup (always encrypted)
  --retention DAYS    Keep backups for N days (default: 7, 0 = no cleanup)
  -h, --help          Show this help message

Examples:
  $0                  # Basic backup
  $0 --encrypt        # Encrypted backup
  $0 --include-secrets --encrypt  # Full backup with secrets
  $0 --retention 30   # Keep backups for 30 days

Files backed up:
  - Node data from all running nodes
  - Configuration files
  - Deployment scripts
  - Secrets (if requested)

EOF
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --no-compress)
            COMPRESS=false
            shift
            ;;
        --encrypt)
            ENCRYPT=true
            shift
            ;;
        --include-secrets)
            INCLUDE_SECRETS=true
            shift
            ;;
        --retention)
            RETENTION_DAYS="$2"
            shift 2
            ;;
        -h|--help)
            show_help
            exit 0
            ;;
        *)
            log_error "Unknown option: $1"
            show_help
            exit 1
            ;;
    esac
done

# Main execution
main() {
    log_info "Starting QuDAG testnet backup..."
    
    check_prerequisites
    create_backup_dir
    
    local success_count=0
    local total_nodes=${#NODES[@]}
    
    # Backup each node
    for node in "${!NODES[@]}"; do
        IFS=':' read -r location region app_name <<< "${NODES[$node]}"
        
        if backup_node_data "$app_name" "$node" "$location"; then
            ((success_count++))
        fi
    done
    
    # Backup configurations
    backup_configurations
    
    # Backup secrets if requested
    backup_secrets
    
    # Generate manifest
    generate_backup_manifest
    
    # Cleanup old backups
    cleanup_old_backups
    
    print_summary
    
    log_success "Backup completed! ($success_count/$total_nodes nodes backed up)"
}

main