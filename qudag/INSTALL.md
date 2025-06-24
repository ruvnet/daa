# QuDAG CLI Installation Guide

This guide provides detailed instructions for installing the QuDAG CLI tool on your system.

## Prerequisites

- Rust 1.70 or newer (install from [rustup.rs](https://rustup.rs/))
- Git
- A UNIX-like operating system (Linux, macOS, BSD)

## Quick Installation

The easiest way to install QuDAG CLI is using the provided installation script:

```bash
./install.sh
```

This will:
1. Build the CLI in release mode
2. Install it to `~/.local/bin/qudag`
3. Update your PATH if needed
4. Install shell completions (if supported)

## Installation Methods

### Method 1: Installation Script

The installation script provides the most flexibility and handles all setup automatically.

```bash
# Install to user directory (~/.local)
./install.sh

# Install system-wide (requires sudo)
sudo ./install.sh --system

# Install to custom directory
./install.sh --prefix /opt/qudag

# Install debug build (for development)
./install.sh --debug

# Force reinstall (overwrite existing)
./install.sh --force

# View all options
./install.sh --help
```

### Method 2: Using Make

The Makefile provides convenient targets for common installation scenarios:

```bash
# Install to ~/.local
make install

# Install system-wide
make install-system

# Install debug build
make install-debug

# Install to CARGO_HOME/bin
make install-cargo
```

### Method 3: Manual Installation

For complete control over the installation process:

```bash
# Build in release mode
cargo build --release -p qudag-cli

# Copy binary to desired location
cp target/release/qudag-cli ~/.local/bin/qudag

# Make sure ~/.local/bin is in your PATH
export PATH="$HOME/.local/bin:$PATH"

# Add to your shell RC file for persistence
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
```

### Method 4: Cargo Install (Coming Soon)

Once published to crates.io:

```bash
cargo install qudag-cli
```

## Post-Installation

After installation, verify everything is working:

```bash
# Check version
qudag --version

# View help
qudag --help

# If PATH wasn't updated automatically, reload your shell
source ~/.bashrc  # or ~/.zshrc for Zsh users
```

## Shell Completions

The installation script automatically installs shell completions for:
- Bash
- Zsh
- Fish

If completions weren't installed automatically, you can generate them manually:

```bash
# For Bash
qudag completions bash > ~/.local/share/bash-completion/completions/qudag

# For Zsh
qudag completions zsh > ~/.zfunc/_qudag

# For Fish
qudag completions fish > ~/.config/fish/completions/qudag.fish
```

## Updating

To update to the latest version:

```bash
# Pull latest changes
git pull

# Reinstall
./install.sh --force
```

## Uninstalling

To remove QuDAG CLI from your system:

```bash
# Using the uninstall script
./uninstall.sh

# Or using Make
make uninstall

# Manual removal
rm ~/.local/bin/qudag
```

You may also want to remove the PATH entry from your shell configuration file.

## Troubleshooting

### Command not found

If you get "command not found" after installation:

1. Check if the binary exists:
   ```bash
   ls -la ~/.local/bin/qudag
   ```

2. Check if the directory is in your PATH:
   ```bash
   echo $PATH | grep -q "$HOME/.local/bin" && echo "IN PATH" || echo "NOT IN PATH"
   ```

3. If not in PATH, add it:
   ```bash
   export PATH="$HOME/.local/bin:$PATH"
   source ~/.bashrc
   ```

### Permission denied

If you get permission errors:

1. Make sure the binary is executable:
   ```bash
   chmod +x ~/.local/bin/qudag
   ```

2. For system-wide installation, use sudo:
   ```bash
   sudo ./install.sh --system
   ```

### Build failures

If the build fails:

1. Update Rust:
   ```bash
   rustup update
   ```

2. Clean and rebuild:
   ```bash
   cargo clean
   ./install.sh
   ```

3. Check for missing dependencies:
   ```bash
   cargo check -p qudag-cli
   ```

## Platform-Specific Notes

### macOS

On macOS, you may need to install Xcode Command Line Tools:

```bash
xcode-select --install
```

### Linux

Most Linux distributions work out of the box. For minimal distributions, you may need:

```bash
# Debian/Ubuntu
sudo apt-get install build-essential pkg-config

# Fedora/RHEL
sudo dnf install gcc pkg-config

# Arch
sudo pacman -S base-devel pkg-config
```

### Windows (WSL)

QuDAG CLI is designed for UNIX-like systems. On Windows, use WSL2:

1. Install WSL2
2. Install a Linux distribution (Ubuntu recommended)
3. Follow the Linux installation instructions

## Getting Help

If you encounter issues:

1. Check the [troubleshooting guide](USAGE_GUIDE.md#troubleshooting)
2. Run the test script: `./test_install.sh`
3. File an issue on [GitHub](https://github.com/ruvnet/QuDAG/issues)

## Next Steps

After installation, see the [Usage Guide](USAGE_GUIDE.md) to:
- Start your first QuDAG node
- Learn about CLI commands
- Explore advanced features