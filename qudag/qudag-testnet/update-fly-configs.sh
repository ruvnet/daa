#!/bin/bash
set -euo pipefail

# Script to update all fly.node*.toml files with consistent configuration
# This ensures all nodes have proper health checks, environment variables, and settings

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[UPDATING]${NC} $1"
}

# Update fly.node1.toml (Toronto - Bootstrap Node)
update_node1() {
    log_warn "Updating fly.node1.toml..."
    
    cat > nodes/fly.node1.toml << 'EOF'
# QuDAG Node 1 - Toronto (yyz) - Bootstrap Node
# Updated configuration with proper health checks and settings

app = "qudag-testnet-node1"
primary_region = "yyz"
kill_signal = "SIGINT"
kill_timeout = "5s"

[build]
  dockerfile = "../Dockerfile"

[build.args]
  NODE_TYPE = "bootstrap"
  NODE_NAME = "toronto-node"

[env]
  RUST_LOG = "info,qudag=debug"
  RUST_BACKTRACE = "1"
  QUDAG_NODE_NAME = "toronto-node"
  QUDAG_NETWORK_ID = "qudag-testnet"
  QUDAG_DARK_DOMAIN_ENABLED = "true"
  QUDAG_P2P_PORT = "4001"
  QUDAG_RPC_PORT = "8080"
  QUDAG_METRICS_PORT = "9090"
  QUDAG_CONFIG_PATH = "/data/qudag/config.toml"
  # Bootstrap node specific
  QUDAG_BOOTSTRAP_MODE = "true"
  QUDAG_REGISTRY_AUTHORITY = "true"
  QUDAG_IS_BOOTSTRAP = "true"

[experimental]
  auto_rollback = true
  enable_consul = false

[[services]]
  internal_port = 8080
  protocol = "tcp"
  auto_stop_machines = false
  auto_start_machines = true
  min_machines_running = 1
  processes = ["app"]

  [services.concurrency]
    type = "connections"
    hard_limit = 200
    soft_limit = 150

  [[services.ports]]
    port = 80
    handlers = ["http"]
    force_https = false

  [[services.ports]]
    port = 443
    handlers = ["tls", "http"]

  [[services.http_checks]]
    interval = "30s"
    timeout = "10s"
    grace_period = "45s"
    method = "GET"
    path = "/health"
    protocol = "http"
    restart_limit = 3
    [services.http_checks.headers]
      X-Node-Type = "bootstrap"

# P2P Port - TCP
[[services]]
  internal_port = 4001
  protocol = "tcp"
  processes = ["app"]
  
  [[services.ports]]
    port = 4001

# P2P Port - UDP (for QUIC transport)
[[services]]
  internal_port = 4001
  protocol = "udp"
  processes = ["app"]
  
  [[services.ports]]
    port = 4001

# Metrics endpoint
[[services]]
  internal_port = 9090
  protocol = "tcp"
  processes = ["app"]
  
  [[services.ports]]
    port = 9090
    handlers = ["http"]
  
  [[services.tcp_checks]]
    interval = "30s"
    timeout = "5s"
    grace_period = "10s"

[metrics]
  port = 9090
  path = "/metrics"

[[vm]]
  size = "shared-cpu-2x"
  cpu_kind = "shared"
  cpus = 2
  memory_mb = 4096

[mounts]
  source = "qudag_data_node1"
  destination = "/data/qudag"

[processes]
  app = """
    set -e
    echo "Starting QuDAG Bootstrap Node (Toronto)..."
    
    # Copy configuration
    cp /app/configs/node1.toml /data/qudag/config.toml
    
    # Ensure directories exist
    mkdir -p /data/qudag/tls
    mkdir -p /data/qudag/db
    
    # Start the node
    exec /app/qudag-node --config /data/qudag/config.toml
  """

[[statics]]
  guest_path = "/app/configs/node1.toml"
  url_prefix = "/"

[[regions]]
  yyz = 1  # Primary region (Toronto)
  ord = 0  # Backup region (Chicago)
EOF
    
    log_info "fly.node1.toml updated"
}

# Update fly.node2.toml (Amsterdam - Validator Node)
update_node2() {
    log_warn "Updating fly.node2.toml..."
    
    cat > nodes/fly.node2.toml << 'EOF'
# QuDAG Node 2 - Amsterdam (ams) - Validator Node
# Updated configuration with proper health checks and settings

app = "qudag-testnet-node2"
primary_region = "ams"
kill_signal = "SIGINT"
kill_timeout = "5s"

[build]
  dockerfile = "../Dockerfile"

[build.args]
  NODE_TYPE = "validator"
  NODE_NAME = "amsterdam-node"

[env]
  RUST_LOG = "info,qudag=debug"
  RUST_BACKTRACE = "1"
  QUDAG_NODE_NAME = "amsterdam-node"
  QUDAG_NETWORK_ID = "qudag-testnet"
  QUDAG_DARK_DOMAIN_ENABLED = "true"
  QUDAG_P2P_PORT = "4001"
  QUDAG_RPC_PORT = "8080"
  QUDAG_METRICS_PORT = "9090"
  QUDAG_CONFIG_PATH = "/data/qudag/config.toml"
  # Validator node specific
  QUDAG_BOOTSTRAP_MODE = "false"
  QUDAG_BOOTSTRAP_PEERS = "/dns4/qudag-testnet-node1.fly.dev/tcp/4001"

[experimental]
  auto_rollback = true
  enable_consul = false

[[services]]
  internal_port = 8080
  protocol = "tcp"
  auto_stop_machines = false
  auto_start_machines = true
  min_machines_running = 1
  processes = ["app"]

  [services.concurrency]
    type = "connections"
    hard_limit = 100
    soft_limit = 80

  [[services.ports]]
    port = 80
    handlers = ["http"]
    force_https = false

  [[services.ports]]
    port = 443
    handlers = ["tls", "http"]

  [[services.http_checks]]
    interval = "30s"
    timeout = "10s"
    grace_period = "45s"
    method = "GET"
    path = "/health"
    protocol = "http"
    restart_limit = 3
    [services.http_checks.headers]
      X-Node-Type = "validator"

# P2P Port - TCP
[[services]]
  internal_port = 4001
  protocol = "tcp"
  processes = ["app"]
  
  [[services.ports]]
    port = 4001

# P2P Port - UDP (for QUIC transport)
[[services]]
  internal_port = 4001
  protocol = "udp"
  processes = ["app"]
  
  [[services.ports]]
    port = 4001

# Metrics endpoint
[[services]]
  internal_port = 9090
  protocol = "tcp"
  processes = ["app"]
  
  [[services.ports]]
    port = 9090
    handlers = ["http"]

[metrics]
  port = 9090
  path = "/metrics"

[[vm]]
  size = "shared-cpu-1x"
  cpu_kind = "shared"
  cpus = 1
  memory_mb = 2048

[mounts]
  source = "qudag_data_node2"
  destination = "/data/qudag"

[processes]
  app = """
    set -e
    echo "Starting QuDAG Validator Node (Amsterdam)..."
    
    # Copy configuration
    cp /app/configs/node2.toml /data/qudag/config.toml
    
    # Ensure directories exist
    mkdir -p /data/qudag/tls
    mkdir -p /data/qudag/db
    
    # Wait for bootstrap node
    echo "Waiting for bootstrap node..."
    sleep 10
    
    # Start the node
    exec /app/qudag-node --config /data/qudag/config.toml
  """

[[statics]]
  guest_path = "/app/configs/node2.toml"
  url_prefix = "/"

[[regions]]
  ams = 1  # Primary region (Amsterdam)
  fra = 0  # Backup region (Frankfurt)
EOF
    
    log_info "fly.node2.toml updated"
}

# Update fly.node3.toml (Singapore - Validator Node)
update_node3() {
    log_warn "Updating fly.node3.toml..."
    
    cat > nodes/fly.node3.toml << 'EOF'
# QuDAG Node 3 - Singapore (sin) - Validator Node
# Updated configuration with proper health checks and settings

app = "qudag-testnet-node3"
primary_region = "sin"
kill_signal = "SIGINT"
kill_timeout = "5s"

[build]
  dockerfile = "../Dockerfile"

[build.args]
  NODE_TYPE = "validator"
  NODE_NAME = "singapore-node"

[env]
  RUST_LOG = "info,qudag=debug"
  RUST_BACKTRACE = "1"
  QUDAG_NODE_NAME = "singapore-node"
  QUDAG_NETWORK_ID = "qudag-testnet"
  QUDAG_DARK_DOMAIN_ENABLED = "true"
  QUDAG_P2P_PORT = "4001"
  QUDAG_RPC_PORT = "8080"
  QUDAG_METRICS_PORT = "9090"
  QUDAG_CONFIG_PATH = "/data/qudag/config.toml"
  # Validator node specific
  QUDAG_BOOTSTRAP_MODE = "false"
  QUDAG_BOOTSTRAP_PEERS = "/dns4/qudag-testnet-node1.fly.dev/tcp/4001"

[experimental]
  auto_rollback = true
  enable_consul = false

[[services]]
  internal_port = 8080
  protocol = "tcp"
  auto_stop_machines = false
  auto_start_machines = true
  min_machines_running = 1
  processes = ["app"]

  [services.concurrency]
    type = "connections"
    hard_limit = 100
    soft_limit = 80

  [[services.ports]]
    port = 80
    handlers = ["http"]
    force_https = false

  [[services.ports]]
    port = 443
    handlers = ["tls", "http"]

  [[services.http_checks]]
    interval = "30s"
    timeout = "10s"
    grace_period = "45s"
    method = "GET"
    path = "/health"
    protocol = "http"
    restart_limit = 3
    [services.http_checks.headers]
      X-Node-Type = "validator"

# P2P Port - TCP
[[services]]
  internal_port = 4001
  protocol = "tcp"
  processes = ["app"]
  
  [[services.ports]]
    port = 4001

# P2P Port - UDP (for QUIC transport)
[[services]]
  internal_port = 4001
  protocol = "udp"
  processes = ["app"]
  
  [[services.ports]]
    port = 4001

# Metrics endpoint
[[services]]
  internal_port = 9090
  protocol = "tcp"
  processes = ["app"]
  
  [[services.ports]]
    port = 9090
    handlers = ["http"]

[metrics]
  port = 9090
  path = "/metrics"

[[vm]]
  size = "shared-cpu-1x"
  cpu_kind = "shared"
  cpus = 1
  memory_mb = 2048

[mounts]
  source = "qudag_data_node3"
  destination = "/data/qudag"

[processes]
  app = """
    set -e
    echo "Starting QuDAG Validator Node (Singapore)..."
    
    # Copy configuration
    cp /app/configs/node3.toml /data/qudag/config.toml
    
    # Ensure directories exist
    mkdir -p /data/qudag/tls
    mkdir -p /data/qudag/db
    
    # Wait for bootstrap node
    echo "Waiting for bootstrap node..."
    sleep 15
    
    # Start the node
    exec /app/qudag-node --config /data/qudag/config.toml
  """

[[statics]]
  guest_path = "/app/configs/node3.toml"
  url_prefix = "/"

[[regions]]
  sin = 1  # Primary region (Singapore)
  nrt = 0  # Backup region (Tokyo)
EOF
    
    log_info "fly.node3.toml updated"
}

# Update fly.node4.toml (San Francisco - Validator Node)
update_node4() {
    log_warn "Updating fly.node4.toml..."
    
    cat > nodes/fly.node4.toml << 'EOF'
# QuDAG Node 4 - San Francisco (sfo) - Validator Node
# Updated configuration with proper health checks and settings

app = "qudag-testnet-node4"
primary_region = "sjc"
kill_signal = "SIGINT"
kill_timeout = "5s"

[build]
  dockerfile = "../Dockerfile"

[build.args]
  NODE_TYPE = "validator"
  NODE_NAME = "sanfrancisco-node"

[env]
  RUST_LOG = "info,qudag=debug"
  RUST_BACKTRACE = "1"
  QUDAG_NODE_NAME = "sanfrancisco-node"
  QUDAG_NETWORK_ID = "qudag-testnet"
  QUDAG_DARK_DOMAIN_ENABLED = "true"
  QUDAG_P2P_PORT = "4001"
  QUDAG_RPC_PORT = "8080"
  QUDAG_METRICS_PORT = "9090"
  QUDAG_CONFIG_PATH = "/data/qudag/config.toml"
  # Validator node specific
  QUDAG_BOOTSTRAP_MODE = "false"
  QUDAG_BOOTSTRAP_PEERS = "/dns4/qudag-testnet-node1.fly.dev/tcp/4001"

[experimental]
  auto_rollback = true
  enable_consul = false

[[services]]
  internal_port = 8080
  protocol = "tcp"
  auto_stop_machines = false
  auto_start_machines = true
  min_machines_running = 1
  processes = ["app"]

  [services.concurrency]
    type = "connections"
    hard_limit = 100
    soft_limit = 80

  [[services.ports]]
    port = 80
    handlers = ["http"]
    force_https = false

  [[services.ports]]
    port = 443
    handlers = ["tls", "http"]

  [[services.http_checks]]
    interval = "30s"
    timeout = "10s"
    grace_period = "45s"
    method = "GET"
    path = "/health"
    protocol = "http"
    restart_limit = 3
    [services.http_checks.headers]
      X-Node-Type = "validator"

# P2P Port - TCP
[[services]]
  internal_port = 4001
  protocol = "tcp"
  processes = ["app"]
  
  [[services.ports]]
    port = 4001

# P2P Port - UDP (for QUIC transport)
[[services]]
  internal_port = 4001
  protocol = "udp"
  processes = ["app"]
  
  [[services.ports]]
    port = 4001

# Metrics endpoint
[[services]]
  internal_port = 9090
  protocol = "tcp"
  processes = ["app"]
  
  [[services.ports]]
    port = 9090
    handlers = ["http"]

[metrics]
  port = 9090
  path = "/metrics"

[[vm]]
  size = "shared-cpu-1x"
  cpu_kind = "shared"
  cpus = 1
  memory_mb = 2048

[mounts]
  source = "qudag_data_node4"
  destination = "/data/qudag"

[processes]
  app = """
    set -e
    echo "Starting QuDAG Validator Node (San Francisco)..."
    
    # Copy configuration
    cp /app/configs/node4.toml /data/qudag/config.toml
    
    # Ensure directories exist
    mkdir -p /data/qudag/tls
    mkdir -p /data/qudag/db
    
    # Wait for bootstrap node
    echo "Waiting for bootstrap node..."
    sleep 20
    
    # Start the node
    exec /app/qudag-node --config /data/qudag/config.toml
  """

[[statics]]
  guest_path = "/app/configs/node4.toml"
  url_prefix = "/"

[[regions]]
  sjc = 1  # Primary region (San Jose/SF)
  sea = 0  # Backup region (Seattle)
EOF
    
    log_info "fly.node4.toml updated"
}

# Main execution
main() {
    log_info "Updating all fly.node*.toml configurations..."
    
    # Create backup directory
    mkdir -p nodes/backup
    
    # Backup existing files
    for node in node1 node2 node3 node4; do
        if [ -f "nodes/fly.$node.toml" ]; then
            cp "nodes/fly.$node.toml" "nodes/backup/fly.$node.toml.$(date +%Y%m%d-%H%M%S)"
        fi
    done
    
    # Update all configurations
    update_node1
    update_node2
    update_node3
    update_node4
    
    log_info "All fly.node*.toml files have been updated!"
    log_info "Backups saved in nodes/backup/"
    log_info ""
    log_info "Key updates made:"
    log_info "  - Consistent health check configuration"
    log_info "  - Proper environment variables for all nodes"
    log_info "  - Unique volume names for each node"
    log_info "  - Increased grace periods for startup"
    log_info "  - Bootstrap peer addresses simplified"
    log_info "  - Staggered startup delays for validators"
}

# Run main function
main "$@"