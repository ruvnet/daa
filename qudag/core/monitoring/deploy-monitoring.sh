#!/bin/bash

# Deploy QuDAG Monitoring Stack

set -e

echo "Deploying QuDAG Monitoring Stack..."

# Check if Docker is installed
if ! command -v docker &> /dev/null; then
    echo "Docker is not installed. Please install Docker first."
    exit 1
fi

# Check if Docker Compose is installed
if ! command -v docker-compose &> /dev/null; then
    echo "Docker Compose is not installed. Please install Docker Compose first."
    exit 1
fi

# Copy dashboard to provisioning directory
echo "Setting up Grafana dashboards..."
cp dashboard_config.json grafana-provisioning/dashboards/qudag-performance.json

# Start the monitoring stack
echo "Starting monitoring services..."
docker-compose up -d

# Wait for services to be ready
echo "Waiting for services to be ready..."
sleep 10

# Check service health
echo "Checking service health..."
docker-compose ps

echo ""
echo "Monitoring stack deployed successfully!"
echo ""
echo "Access the services at:"
echo "  - Grafana: http://localhost:3001 (admin/admin)"
echo "  - Prometheus: http://localhost:9091"
echo "  - Alertmanager: http://localhost:9093"
echo ""
echo "QuDAG metrics endpoint should be exposed at: http://localhost:9090/metrics"
echo ""
echo "To stop the monitoring stack, run: docker-compose down"
echo "To view logs, run: docker-compose logs -f"