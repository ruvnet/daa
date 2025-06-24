# QuDAG Makefile
# Provides convenient targets for building and installing QuDAG

.PHONY: all build install install-debug install-system uninstall clean help

# Default target
all: build

# Build the project
build:
	@echo "Building QuDAG in release mode..."
	@cargo build --release --workspace

# Build in debug mode
build-debug:
	@echo "Building QuDAG in debug mode..."
	@cargo build --workspace

# Install to user directory (~/.local)
install: build
	@echo "Installing QuDAG CLI to ~/.local..."
	@./install.sh

# Install debug build
install-debug: build-debug
	@echo "Installing QuDAG CLI (debug build) to ~/.local..."
	@./install.sh --debug

# Install system-wide (requires sudo)
install-system: build
	@echo "Installing QuDAG CLI system-wide..."
	@sudo ./install.sh --system

# Install to CARGO_HOME/bin
install-cargo: build
	@echo "Installing QuDAG CLI to CARGO_HOME/bin..."
	@./install.sh --cargo-home

# Uninstall
uninstall:
	@echo "Uninstalling QuDAG CLI..."
	@./uninstall.sh

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	@cargo clean

# Run tests
test:
	@echo "Running tests..."
	@cargo test --workspace

# Run benchmarks
bench:
	@echo "Running benchmarks..."
	@cargo bench

# Check code quality
check:
	@echo "Running code quality checks..."
	@cargo fmt --check
	@cargo clippy -- -D warnings

# Format code
fmt:
	@echo "Formatting code..."
	@cargo fmt

# Show help
help:
	@echo "QuDAG Makefile targets:"
	@echo ""
	@echo "  make              - Build the project in release mode"
	@echo "  make build        - Build the project in release mode"
	@echo "  make build-debug  - Build the project in debug mode"
	@echo "  make install      - Install to ~/.local"
	@echo "  make install-debug - Install debug build to ~/.local"
	@echo "  make install-system - Install system-wide (requires sudo)"
	@echo "  make install-cargo - Install to CARGO_HOME/bin"
	@echo "  make uninstall    - Remove installation"
	@echo "  make clean        - Clean build artifacts"
	@echo "  make test         - Run tests"
	@echo "  make bench        - Run benchmarks"
	@echo "  make check        - Run code quality checks"
	@echo "  make fmt          - Format code"
	@echo "  make help         - Show this help message"
	@echo ""
	@echo "For more options, run: ./install.sh --help"