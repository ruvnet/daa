#!/bin/bash
# Build script for QuDAG Docker images

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${GREEN}[BUILD]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

# Check if we're in the right directory
if [ ! -f "../Cargo.toml" ]; then
    print_error "This script must be run from the qudag-testnet directory"
    exit 1
fi

# Parse command line arguments
BUILD_CACHE=""
while [[ $# -gt 0 ]]; do
    case $1 in
        --no-cache)
            BUILD_CACHE="--no-cache"
            print_warning "Building without cache"
            shift
            ;;
        --help)
            echo "Usage: $0 [options]"
            echo "Options:"
            echo "  --no-cache    Build without using Docker cache"
            echo "  --help        Show this help message"
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Build the Docker image from the workspace root
print_status "Building QuDAG Docker image..."
cd ..
docker build $BUILD_CACHE -f qudag-testnet/Dockerfile -t qudag:latest .

if [ $? -eq 0 ]; then
    print_status "Build completed successfully!"
    echo ""
    echo "To run the testnet with regular nodes:"
    echo "  cd qudag-testnet && docker-compose up"
    echo ""
    echo "To run the testnet with exchange server:"
    echo "  cd qudag-testnet && docker-compose -f docker-compose-with-exchange.yml up"
    echo ""
    echo "To run a single node:"
    echo "  docker run -e NODE_TYPE=node qudag:latest"
    echo ""
    echo "To run an exchange server:"
    echo "  docker run -e NODE_TYPE=exchange -p 8085:8085 qudag:latest"
else
    print_error "Build failed!"
    exit 1
fi