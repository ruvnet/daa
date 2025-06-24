#!/bin/bash
# QuDAG Testnet Deployment Script
# Orchestrates multi-node testnet deployment with monitoring

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
TESTNET_NAME="${TESTNET_NAME:-qudag-testnet}"
NODE_COUNT="${NODE_COUNT:-5}"
NETWORK_SUBNET="${NETWORK_SUBNET:-172.20.0.0/16}"

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
    for cmd in docker docker-compose jq curl; do
        if ! command -v $cmd &> /dev/null; then
            missing_deps+=($cmd)
        fi
    done
    
    if [ ${#missing_deps[@]} -ne 0 ]; then
        log_error "Missing dependencies: ${missing_deps[*]}"
        log_error "Please install missing dependencies and try again."
        exit 1
    fi
    
    # Check Docker daemon
    if ! docker info &> /dev/null; then
        log_error "Docker daemon is not running"
        exit 1
    fi
    
    log_success "All prerequisites met"
}

# Generate node configurations
generate_node_configs() {
    log_info "Generating node configurations..."
    
    local config_dir="${PROJECT_ROOT}/testnet-configs"
    mkdir -p "${config_dir}"
    
    # Generate bootstrap node config
    cat > "${config_dir}/bootstrap.toml" << EOF
[node]
id = "bootstrap-node"
name = "QuDAG Bootstrap Node"
role = "bootstrap"

[network]
listen_address = "/ip4/0.0.0.0/tcp/4001"
external_address = "/dns4/bootstrap-node/tcp/4001"
enable_mdns = true
enable_kad = true
enable_gossipsub = true

[network.nat]
enable_upnp = true
enable_pcp = true
enable_nat_pmp = true

[consensus]
algorithm = "qr-avalanche"
finality_threshold = 0.8
sample_size = 20

[crypto]
algorithm = "ml-dsa"
key_rotation_interval = 86400

[storage]
path = "/data"
cache_size = "1GB"

[api]
listen_address = "0.0.0.0:8080"
enable_cors = true
max_request_size = "10MB"

[metrics]
enable = true
listen_address = "0.0.0.0:9090"
EOF

    # Generate configs for other nodes
    for i in $(seq 1 $((NODE_COUNT - 1))); do
        cat > "${config_dir}/node-${i}.toml" << EOF
[node]
id = "node-${i}"
name = "QuDAG Node ${i}"
role = "full"

[network]
listen_address = "/ip4/0.0.0.0/tcp/4001"
external_address = "/dns4/node-${i}/tcp/4001"
bootstrap_peers = ["/dns4/bootstrap-node/tcp/4001"]
enable_mdns = true
enable_kad = true
enable_gossipsub = true

[network.nat]
enable_upnp = true
enable_pcp = true
enable_nat_pmp = true

[consensus]
algorithm = "qr-avalanche"
finality_threshold = 0.8
sample_size = 20

[crypto]
algorithm = "ml-dsa"
key_rotation_interval = 86400

[storage]
path = "/data"
cache_size = "512MB"

[api]
listen_address = "0.0.0.0:8080"
enable_cors = true
max_request_size = "10MB"

[metrics]
enable = true
listen_address = "0.0.0.0:9090"
EOF
    done
    
    log_success "Generated configurations for ${NODE_COUNT} nodes"
}

# Generate docker-compose for testnet
generate_docker_compose() {
    log_info "Generating docker-compose configuration..."
    
    cat > "${PROJECT_ROOT}/docker-compose.testnet.yml" << EOF
version: '3.8'

services:
  # Bootstrap node
  bootstrap-node:
    build:
      context: .
      dockerfile: Dockerfile
    container_name: ${TESTNET_NAME}-bootstrap
    hostname: bootstrap-node
    restart: unless-stopped
    environment:
      - NODE_ID=bootstrap-node
      - RUST_LOG=info,qudag=debug
    ports:
      - "4001:4001"
      - "8080:8080"
      - "9090:9090"
    volumes:
      - bootstrap-data:/data
      - ./testnet-configs/bootstrap.toml:/config/node.toml:ro
    networks:
      testnet:
        ipv4_address: 172.20.0.10
    healthcheck:
      test: ["CMD", "qudag", "status"]
      interval: 30s
      timeout: 10s
      retries: 3

EOF

    # Add other nodes
    for i in $(seq 1 $((NODE_COUNT - 1))); do
        local ip_suffix=$((10 + i))
        cat >> "${PROJECT_ROOT}/docker-compose.testnet.yml" << EOF
  node-${i}:
    build:
      context: .
      dockerfile: Dockerfile
    container_name: ${TESTNET_NAME}-node-${i}
    hostname: node-${i}
    restart: unless-stopped
    depends_on:
      bootstrap-node:
        condition: service_healthy
    environment:
      - NODE_ID=node-${i}
      - RUST_LOG=info
    ports:
      - "$((4001 + i)):4001"
      - "$((8080 + i)):8080"
      - "$((9090 + i)):9090"
    volumes:
      - node-${i}-data:/data
      - ./testnet-configs/node-${i}.toml:/config/node.toml:ro
    networks:
      testnet:
        ipv4_address: 172.20.0.${ip_suffix}
    healthcheck:
      test: ["CMD", "qudag", "status"]
      interval: 30s
      timeout: 10s
      retries: 3

EOF
    done

    # Add monitoring services
    cat >> "${PROJECT_ROOT}/docker-compose.testnet.yml" << EOF
  # Prometheus
  prometheus:
    image: prom/prometheus:latest
    container_name: ${TESTNET_NAME}-prometheus
    restart: unless-stopped
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'
      - '--web.enable-lifecycle'
    ports:
      - "9093:9090"
    volumes:
      - ./monitoring/prometheus-testnet.yml:/etc/prometheus/prometheus.yml:ro
      - prometheus-data:/prometheus
    networks:
      - testnet

  # Grafana
  grafana:
    image: grafana/grafana:latest
    container_name: ${TESTNET_NAME}-grafana
    restart: unless-stopped
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=testnet
      - GF_USERS_ALLOW_SIGN_UP=false
    ports:
      - "3000:3000"
    volumes:
      - ./monitoring/grafana/provisioning:/etc/grafana/provisioning:ro
      - grafana-data:/var/lib/grafana
    networks:
      - testnet
    depends_on:
      - prometheus

networks:
  testnet:
    driver: bridge
    ipam:
      config:
        - subnet: ${NETWORK_SUBNET}

volumes:
  bootstrap-data:
  prometheus-data:
  grafana-data:
EOF

    # Add volume entries for each node
    for i in $(seq 1 $((NODE_COUNT - 1))); do
        echo "  node-${i}-data:" >> "${PROJECT_ROOT}/docker-compose.testnet.yml"
    done
    
    log_success "Generated docker-compose configuration"
}

# Generate Prometheus configuration
generate_prometheus_config() {
    log_info "Generating Prometheus configuration..."
    
    mkdir -p "${PROJECT_ROOT}/monitoring"
    
    cat > "${PROJECT_ROOT}/monitoring/prometheus-testnet.yml" << EOF
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'qudag-nodes'
    static_configs:
      - targets:
          - 'bootstrap-node:9090'
EOF

    # Add other nodes to Prometheus targets
    for i in $(seq 1 $((NODE_COUNT - 1))); do
        echo "          - 'node-${i}:9090'" >> "${PROJECT_ROOT}/monitoring/prometheus-testnet.yml"
    done
    
    log_success "Generated Prometheus configuration"
}

# Build Docker images
build_images() {
    log_info "Building Docker images..."
    
    cd "${PROJECT_ROOT}"
    
    # Build main QuDAG image
    docker build -t ${TESTNET_NAME}:latest .
    
    # Build Alpine variant for smaller footprint
    docker build -t ${TESTNET_NAME}:alpine -f Dockerfile.alpine .
    
    log_success "Docker images built successfully"
}

# Deploy testnet
deploy_testnet() {
    log_info "Deploying testnet with ${NODE_COUNT} nodes..."
    
    cd "${PROJECT_ROOT}"
    
    # Start services
    docker-compose -f docker-compose.testnet.yml up -d
    
    # Wait for services to be healthy
    log_info "Waiting for nodes to become healthy..."
    local max_attempts=30
    local attempt=0
    
    while [ $attempt -lt $max_attempts ]; do
        local healthy_count=$(docker-compose -f docker-compose.testnet.yml ps | grep -c "healthy" || true)
        
        if [ $healthy_count -eq $NODE_COUNT ]; then
            log_success "All nodes are healthy!"
            break
        fi
        
        log_info "Healthy nodes: ${healthy_count}/${NODE_COUNT}"
        sleep 10
        ((attempt++))
    done
    
    if [ $attempt -eq $max_attempts ]; then
        log_error "Timeout waiting for nodes to become healthy"
        return 1
    fi
    
    # Display node information
    display_node_info
}

# Display node information
display_node_info() {
    log_info "Testnet deployment summary:"
    echo
    echo "Nodes:"
    
    # Bootstrap node
    local bootstrap_id=$(docker exec ${TESTNET_NAME}-bootstrap qudag peer id 2>/dev/null || echo "Not available")
    echo "  Bootstrap Node:"
    echo "    Container: ${TESTNET_NAME}-bootstrap"
    echo "    P2P Port: 4001"
    echo "    RPC Port: 8080"
    echo "    Metrics Port: 9090"
    echo "    Peer ID: ${bootstrap_id}"
    echo
    
    # Other nodes
    for i in $(seq 1 $((NODE_COUNT - 1))); do
        local node_id=$(docker exec ${TESTNET_NAME}-node-${i} qudag peer id 2>/dev/null || echo "Not available")
        echo "  Node ${i}:"
        echo "    Container: ${TESTNET_NAME}-node-${i}"
        echo "    P2P Port: $((4001 + i))"
        echo "    RPC Port: $((8080 + i))"
        echo "    Metrics Port: $((9090 + i))"
        echo "    Peer ID: ${node_id}"
        echo
    done
    
    echo "Monitoring:"
    echo "  Prometheus: http://localhost:9093"
    echo "  Grafana: http://localhost:3000 (admin/testnet)"
    echo
}

# Stop testnet
stop_testnet() {
    log_info "Stopping testnet..."
    
    cd "${PROJECT_ROOT}"
    docker-compose -f docker-compose.testnet.yml down
    
    log_success "Testnet stopped"
}

# Clean up testnet
cleanup_testnet() {
    log_info "Cleaning up testnet..."
    
    cd "${PROJECT_ROOT}"
    
    # Stop and remove containers, networks, volumes
    docker-compose -f docker-compose.testnet.yml down -v
    
    # Remove generated files
    rm -rf testnet-configs
    rm -f docker-compose.testnet.yml
    rm -f monitoring/prometheus-testnet.yml
    
    log_success "Testnet cleaned up"
}

# Main execution
main() {
    case "${1:-deploy}" in
        deploy)
            check_prerequisites
            generate_node_configs
            generate_docker_compose
            generate_prometheus_config
            build_images
            deploy_testnet
            ;;
        stop)
            stop_testnet
            ;;
        clean)
            cleanup_testnet
            ;;
        restart)
            stop_testnet
            deploy_testnet
            ;;
        info)
            display_node_info
            ;;
        *)
            echo "Usage: $0 {deploy|stop|clean|restart|info}"
            echo
            echo "Commands:"
            echo "  deploy   - Deploy testnet (default)"
            echo "  stop     - Stop testnet"
            echo "  clean    - Clean up testnet and remove all data"
            echo "  restart  - Restart testnet"
            echo "  info     - Display testnet information"
            echo
            echo "Environment variables:"
            echo "  NODE_COUNT - Number of nodes to deploy (default: 5)"
            echo "  TESTNET_NAME - Name prefix for containers (default: qudag-testnet)"
            exit 1
            ;;
    esac
}

# Run main function
main "$@"