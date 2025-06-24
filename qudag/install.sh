#!/usr/bin/env bash

# QuDAG CLI Installation Script
# This script builds and installs the QuDAG CLI tool

set -euo pipefail

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

# Default values
INSTALL_PREFIX="${HOME}/.local"
BINARY_NAME="qudag"
CARGO_BUILD_MODE="release"
FORCE_INSTALL=false
UNINSTALL=false

# Print colored output
print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_step() {
    echo -e "${PURPLE}[STEP]${NC} $1"
}

# Show usage
usage() {
    cat << EOF
QuDAG CLI Installation Script

Usage: ./install.sh [OPTIONS]

OPTIONS:
    -h, --help              Show this help message
    -p, --prefix PATH       Installation prefix (default: ~/.local)
    -d, --debug             Build in debug mode instead of release
    -f, --force             Force installation even if binary exists
    -u, --uninstall         Uninstall QuDAG CLI
    --system                Install system-wide to /usr/local (requires sudo)
    --cargo-home            Install to CARGO_HOME/bin (if set)

EXAMPLES:
    ./install.sh                    # Install to ~/.local
    ./install.sh --system           # Install system-wide
    ./install.sh --prefix /opt      # Install to custom location
    ./install.sh --uninstall        # Remove installation

EOF
}

# Check prerequisites
check_prerequisites() {
    print_step "Checking prerequisites..."
    
    # Check for Rust/Cargo
    if ! command -v cargo &> /dev/null; then
        print_error "Cargo is not installed. Please install Rust from https://rustup.rs/"
        exit 1
    fi
    
    # Check for git (for version info)
    if ! command -v git &> /dev/null; then
        print_warning "Git is not installed. Version information will be limited."
    fi
    
    # Check if we're in the correct directory
    if [ ! -f "Cargo.toml" ] || [ ! -d "tools/cli" ]; then
        print_error "This script must be run from the QuDAG project root directory"
        exit 1
    fi
    
    print_success "All prerequisites met"
}

# Get git version info
get_version_info() {
    if command -v git &> /dev/null && [ -d ".git" ]; then
        GIT_HASH=$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")
        GIT_BRANCH=$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "unknown")
        GIT_DIRTY=$(git diff --quiet || echo "-dirty")
        VERSION_INFO="${GIT_BRANCH}-${GIT_HASH}${GIT_DIRTY}"
    else
        VERSION_INFO="unknown"
    fi
}

# Build the CLI
build_cli() {
    print_step "Building QuDAG CLI in ${CARGO_BUILD_MODE} mode..."
    
    # Set build flags
    BUILD_FLAGS=""
    if [ "$CARGO_BUILD_MODE" = "release" ]; then
        BUILD_FLAGS="--release"
    fi
    
    # Build with version info
    RUSTFLAGS="-C target-cpu=native" \
    QUDAG_VERSION="${VERSION_INFO}" \
    cargo build -p qudag-cli ${BUILD_FLAGS}
    
    if [ $? -eq 0 ]; then
        print_success "Build completed successfully"
    else
        print_error "Build failed"
        exit 1
    fi
}

# Install the binary
install_binary() {
    local install_dir="${INSTALL_PREFIX}/bin"
    local source_binary="target/${CARGO_BUILD_MODE}/qudag"
    local target_binary="${install_dir}/${BINARY_NAME}"
    
    print_step "Installing QuDAG CLI to ${install_dir}..."
    
    # Create installation directory
    if [ ! -d "$install_dir" ]; then
        print_info "Creating installation directory: ${install_dir}"
        mkdir -p "$install_dir"
    fi
    
    # Check if binary already exists
    if [ -f "$target_binary" ] && [ "$FORCE_INSTALL" = false ]; then
        print_error "Binary already exists at ${target_binary}"
        print_info "Use --force to overwrite existing installation"
        exit 1
    fi
    
    # Copy binary
    if [ -f "$source_binary" ]; then
        cp "$source_binary" "$target_binary"
        chmod +x "$target_binary"
        print_success "Binary installed to ${target_binary}"
    else
        print_error "Built binary not found at ${source_binary}"
        exit 1
    fi
    
    # Create version file
    echo "${VERSION_INFO}" > "${install_dir}/.qudag-version"
}

# Check and update PATH
update_path() {
    local install_dir="${INSTALL_PREFIX}/bin"
    local shell_config=""
    local shell_name=$(basename "$SHELL")
    
    print_step "Checking PATH configuration..."
    
    # Check if directory is already in PATH
    if echo "$PATH" | grep -q "$install_dir"; then
        print_success "Installation directory is already in PATH"
        return
    fi
    
    # Determine shell configuration file
    case "$shell_name" in
        bash)
            if [ -f "$HOME/.bashrc" ]; then
                shell_config="$HOME/.bashrc"
            elif [ -f "$HOME/.bash_profile" ]; then
                shell_config="$HOME/.bash_profile"
            fi
            ;;
        zsh)
            shell_config="$HOME/.zshrc"
            ;;
        fish)
            shell_config="$HOME/.config/fish/config.fish"
            ;;
        *)
            print_warning "Unknown shell: $shell_name"
            ;;
    esac
    
    if [ -n "$shell_config" ]; then
        print_info "Adding ${install_dir} to PATH in ${shell_config}"
        
        # Add PATH export
        if [ "$shell_name" = "fish" ]; then
            echo "set -gx PATH ${install_dir} \$PATH" >> "$shell_config"
        else
            echo "export PATH=\"${install_dir}:\$PATH\"" >> "$shell_config"
        fi
        
        print_warning "PATH updated. Please run 'source ${shell_config}' or restart your shell"
    else
        print_warning "Could not determine shell configuration file"
        print_info "Please manually add ${install_dir} to your PATH"
    fi
}

# Create shell completions
install_completions() {
    local completion_dir="${INSTALL_PREFIX}/share/bash-completion/completions"
    
    print_step "Installing shell completions..."
    
    # Check if binary supports completions generation
    if "${INSTALL_PREFIX}/bin/${BINARY_NAME}" completions --help &> /dev/null; then
        mkdir -p "$completion_dir"
        
        # Generate completions for different shells
        for shell in bash zsh fish; do
            if "${INSTALL_PREFIX}/bin/${BINARY_NAME}" completions "$shell" > "${completion_dir}/${BINARY_NAME}.${shell}" 2>/dev/null; then
                print_success "Installed ${shell} completions"
            fi
        done
    else
        print_info "Shell completions not available in this build"
    fi
}

# Uninstall function
uninstall_qudag() {
    print_step "Uninstalling QuDAG CLI..."
    
    local binary_path="${INSTALL_PREFIX}/bin/${BINARY_NAME}"
    local version_file="${INSTALL_PREFIX}/bin/.qudag-version"
    local completion_dir="${INSTALL_PREFIX}/share/bash-completion/completions"
    
    # Remove binary
    if [ -f "$binary_path" ]; then
        rm -f "$binary_path"
        print_success "Removed binary: ${binary_path}"
    else
        print_warning "Binary not found: ${binary_path}"
    fi
    
    # Remove version file
    [ -f "$version_file" ] && rm -f "$version_file"
    
    # Remove completions
    for shell in bash zsh fish; do
        local completion_file="${completion_dir}/${BINARY_NAME}.${shell}"
        [ -f "$completion_file" ] && rm -f "$completion_file"
    done
    
    print_success "Uninstallation complete"
    print_info "You may want to remove the PATH entry from your shell configuration"
}

# Verify installation
verify_installation() {
    print_step "Verifying installation..."
    
    local binary_path="${INSTALL_PREFIX}/bin/${BINARY_NAME}"
    
    if [ -f "$binary_path" ] && [ -x "$binary_path" ]; then
        print_success "Binary is installed and executable"
        
        # Test execution
        if "$binary_path" --version &> /dev/null; then
            local version=$("$binary_path" --version 2>&1 || echo "unknown")
            print_success "QuDAG CLI version: ${version}"
        else
            print_warning "Could not determine version"
        fi
    else
        print_error "Installation verification failed"
        exit 1
    fi
}

# Print post-installation instructions
print_instructions() {
    echo
    print_success "QuDAG CLI installation completed!"
    echo
    echo "To get started:"
    echo "  1. Reload your shell configuration or run:"
    echo "     source ~/.bashrc  # or ~/.zshrc, etc."
    echo
    echo "  2. Verify installation:"
    echo "     ${BINARY_NAME} --version"
    echo
    echo "  3. View available commands:"
    echo "     ${BINARY_NAME} --help"
    echo
    echo "  4. Start a QuDAG node:"
    echo "     ${BINARY_NAME} start"
    echo
    echo "For more information, see the documentation at:"
    echo "  https://github.com/your-org/QuDAG"
    echo
}

# Parse command line arguments
parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help)
                usage
                exit 0
                ;;
            -p|--prefix)
                if [ -z "${2:-}" ]; then
                    print_error "Option --prefix requires an argument"
                    exit 1
                fi
                INSTALL_PREFIX="$2"
                shift 2
                ;;
            -d|--debug)
                CARGO_BUILD_MODE="debug"
                shift
                ;;
            -f|--force)
                FORCE_INSTALL=true
                shift
                ;;
            -u|--uninstall)
                UNINSTALL=true
                shift
                ;;
            --system)
                INSTALL_PREFIX="/usr/local"
                shift
                ;;
            --cargo-home)
                if [ -n "${CARGO_HOME:-}" ]; then
                    INSTALL_PREFIX="$CARGO_HOME"
                else
                    print_error "CARGO_HOME is not set"
                    exit 1
                fi
                shift
                ;;
            *)
                print_error "Unknown option: $1"
                usage
                exit 1
                ;;
        esac
    done
}

# Main installation flow
main() {
    print_info "QuDAG CLI Installation Script"
    echo
    
    # Parse arguments
    parse_args "$@"
    
    # Handle uninstall
    if [ "$UNINSTALL" = true ]; then
        uninstall_qudag
        exit 0
    fi
    
    # Check if we need sudo for system-wide installation
    if [ "$INSTALL_PREFIX" = "/usr/local" ] && [ "$EUID" -ne 0 ]; then
        print_error "System-wide installation requires sudo"
        print_info "Please run: sudo ./install.sh --system"
        exit 1
    fi
    
    # Run installation steps
    check_prerequisites
    get_version_info
    build_cli
    install_binary
    update_path
    install_completions
    verify_installation
    print_instructions
}

# Run main function
main "$@"