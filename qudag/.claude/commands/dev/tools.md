# /dev/tools

## Purpose
Development utilities for QuDAG including file watching, auto-rebuilding, development servers, debugging workflows, and performance profiling. Streamlines development workflow with hot reload and debugging capabilities.

## Parameters
- `<command>`: Tool command - watch|serve|debug|profile|clean|setup (required)
- `[target]`: Target to work with - rust|wasm|docs|all (default: all)
- `[mode]`: Development mode - dev|debug|release (default: dev)
- `[port]`: Server port for serve command (default: 8000)

## Prerequisites
- [ ] Development dependencies installed
- [ ] File watcher tools available (cargo-watch, nodemon)
- [ ] Debug tools installed (gdb, lldb, browser dev tools)
- [ ] Profiling tools available (perf, flamegraph, valgrind)
- [ ] Development server dependencies

## Execution Steps

### 1. Tool Setup and Validation
- Install development dependencies
  ```bash
  # Install Rust development tools
  cargo install cargo-watch cargo-expand cargo-flamegraph
  
  # Install Node.js development tools
  npm install -g nodemon browser-sync http-server
  
  # Install system debugging tools (Linux)
  sudo apt-get install gdb valgrind perf
  ```
- Verify tool availability
- Configure development environment

### 2. File Watching and Auto-Rebuild
- Step 2.1: Watch Rust source files
  ```bash
  cd /workspaces/QuDAG
  
  # Watch and rebuild on changes
  cargo watch -x "build --workspace" -x "test --workspace" \
    --watch src --watch core --ignore "target/*"
  ```
- Step 2.2: Watch WASM development
  ```bash
  cd /workspaces/QuDAG/qudag-wasm
  
  # Watch and rebuild WASM on changes
  cargo watch -s "./build.sh && echo 'WASM rebuilt'" \
    --watch src --ignore "pkg*" --ignore "target"
  ```
- Step 2.3: Watch documentation
  ```bash
  # Auto-rebuild documentation
  cargo watch -x "doc --workspace --no-deps" \
    --watch src --watch core --watch README.md
  ```

### 3. Development Server
- Step 3.1: HTTP server for WASM testing
  ```bash
  cd /workspaces/QuDAG/qudag-wasm
  
  # Serve WASM files with CORS headers
  python3 -m http.server 8000 --bind 0.0.0.0 &
  SERVER_PID=$!
  
  echo "WASM development server: http://localhost:8000"
  echo "Test page: http://localhost:8000/test-wasm-crypto.html"
  ```
- Step 3.2: Live reload development server
  ```bash
  # Browser-sync with live reload
  browser-sync start --server --files "pkg/**/*.js,pkg/**/*.wasm,*.html" \
    --port 3000 --no-open
  ```
- Step 3.3: Documentation server
  ```bash
  # Serve generated documentation
  cargo doc --workspace --no-deps --open
  
  # Or manual server
  cd target/doc
  python3 -m http.server 8080
  ```

### 4. Debugging Workflows
- Step 4.1: Rust debugging setup
  ```bash
  # Build with debug symbols
  cargo build --workspace
  
  # Debug with GDB
  gdb target/debug/qudag
  # (gdb) break main
  # (gdb) run --help
  ```
- Step 4.2: WASM debugging
  ```bash
  # Build WASM with debug info
  cd /workspaces/QuDAG/qudag-wasm
  wasm-pack build --dev --target web
  
  # Use browser dev tools for debugging
  echo "Open browser dev tools and check Sources tab for WASM debugging"
  ```
- Step 4.3: Network debugging
  ```bash
  # Debug network operations with detailed logging
  RUST_LOG=debug,qudag_network=trace cargo run -- node start
  ```

### 5. Performance Profiling
- Step 5.1: CPU profiling with perf
  ```bash
  # Profile CPU usage
  cargo build --release --workspace
  
  perf record --call-graph dwarf target/release/qudag benchmark
  perf report --stdio > perf-report.txt
  ```
- Step 5.2: Memory profiling with Valgrind
  ```bash
  # Check for memory leaks
  valgrind --tool=memcheck --leak-check=full \
    target/debug/qudag test-command
  ```
- Step 5.3: Flame graph generation
  ```bash
  # Generate flame graph
  cargo flamegraph --bin qudag -- benchmark
  
  # WASM performance profiling
  cd /workspaces/QuDAG/qudag-wasm
  node --prof test-performance.mjs
  ```

### 6. Code Analysis Tools
- Step 6.1: Cargo expand for macro debugging
  ```bash
  # Expand macros to debug compilation issues
  cargo expand --package qudag-crypto > expanded-crypto.rs
  ```
- Step 6.2: Dependency analysis
  ```bash
  # Analyze dependency tree
  cargo tree --workspace --duplicates
  cargo tree --workspace --format "{p} {f}"
  ```
- Step 6.3: Code coverage
  ```bash
  # Generate code coverage report
  cargo install cargo-tarpaulin
  cargo tarpaulin --workspace --out Html --output-dir coverage
  ```

### 7. Development Environment Cleanup
- Step 7.1: Clean build artifacts
  ```bash
  cd /workspaces/QuDAG
  
  # Clean Rust builds
  cargo clean
  
  # Clean WASM builds
  cd qudag-wasm && rm -rf pkg* target
  
  # Clean Node.js modules
  find . -name "node_modules" -type d -exec rm -rf {} +
  ```
- Step 7.2: Reset development environment
  ```bash
  # Reset to clean state
  git clean -fdx
  git reset --hard HEAD
  
  # Reinstall dependencies
  cargo fetch
  ```

## Success Criteria
- [ ] File watchers detect changes and rebuild successfully
- [ ] Development servers start and serve content correctly
- [ ] Debugging tools attach and provide useful information
- [ ] Performance profiling generates actionable insights
- [ ] Hot reload works for both Rust and WASM development
- [ ] All development tools integrate smoothly

## Error Handling
- **File watcher failures**: Check file permissions and inotify limits
- **Server startup failures**: Verify port availability and permissions
- **Debug tool failures**: Install missing system packages
- **Profiling errors**: Ensure profiling tools have necessary permissions
- **Hot reload issues**: Check file paths and watch patterns
- **Memory issues**: Increase system limits or use smaller datasets

## Output
- **Success**: Development environment ready with specified tools
- **Failure**: Error messages with specific tool setup issues
- **Reports**: 
  - Performance profiling results in `/workspaces/QuDAG/profiles/`
  - Code coverage reports in `/workspaces/QuDAG/coverage/`
  - Debug logs and analysis in `/workspaces/QuDAG/debug/`

## Example Usage
```bash
# Start file watcher for Rust development
/dev/tools watch rust dev

# Start development server for WASM
/dev/tools serve wasm dev 8000

# Profile application performance
/dev/tools profile rust release

# Clean development environment
/dev/tools clean all
```

### Example Output
```
Setting up QuDAG development tools...

✓ File watcher started for Rust workspace
  - Watching: src/, core/, qudag/
  - Auto-rebuild on save
  - Running tests after build

✓ WASM development server: http://localhost:8000
  - Serving: qudag-wasm/
  - Live reload enabled
  - CORS headers configured

✓ Documentation server: http://localhost:8080
  - Auto-rebuilding docs on changes
  - API documentation available

Development tools ready:
- Rust: cargo-watch monitoring workspace
- WASM: Live reload server on port 8000
- Docs: Auto-generated at localhost:8080
- Debug: GDB and browser tools configured
```

## Related Commands
- `/build/wasm`: Build WASM for development testing
- `/build/cargo`: Build Rust binaries for profiling
- `/test/wasm`: Test WASM in development environment

## Workflow Integration
This command is part of the Development Workflow and:
- Supports: All development activities with hot reload
- Integrates with: All build and test commands
- Enhances: Developer productivity and debugging capabilities

## Agent Coordination
- **Primary Agent**: Development Tools Agent
- **Supporting Agents**: 
  - Build Agent: Coordinates with auto-rebuild functionality
  - Test Agent: Integrates with continuous testing
  - Performance Agent: Analyzes profiling results

## Notes
- File watchers can be resource-intensive on large codebases
- Some profiling tools require system-level permissions
- Browser debugging works best with source maps enabled
- Memory profiling may significantly slow down execution
- Consider using development containers for consistent environments

---

## Advanced Development Setups

### Multi-Target Development Workflow
```bash
# Watch and build multiple targets simultaneously
start_multi_watch() {
  # Terminal 1: Rust development
  cargo watch -x "build --workspace" -x "test --workspace" &
  
  # Terminal 2: WASM development  
  cd qudag-wasm && cargo watch -s "./build.sh" &
  
  # Terminal 3: Documentation
  cargo watch -x "doc --workspace --no-deps" &
  
  # Terminal 4: Development server
  cd qudag-wasm && python3 -m http.server 8000 &
  
  echo "Multi-target development environment started"
}
```

### Custom Development Scripts
```bash
#!/bin/bash
# dev-setup.sh - Custom development environment setup

setup_rust_analyzer() {
  # Configure rust-analyzer for better IDE support
  cat > .vscode/settings.json << EOF
{
  "rust-analyzer.cargo.features": "all",
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.cargo.target": "wasm32-unknown-unknown"
}
EOF
}

setup_debugging() {
  # Setup debugging configuration
  cat > .vscode/launch.json << EOF
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug QuDAG",
      "cargo": {
        "args": ["build", "--bin=qudag"],
        "filter": {
          "name": "qudag",
          "kind": "bin"
        }
      },
      "args": ["--help"]
    }
  ]
}
EOF
}
```

### Performance Monitoring Dashboard
```bash
# Create performance monitoring dashboard
create_perf_dashboard() {
  cat > perf-monitor.html << 'EOF'
<!DOCTYPE html>
<html>
<head>
    <title>QuDAG Performance Monitor</title>
    <script src="https://cdn.plot.ly/plotly-latest.min.js"></script>
</head>
<body>
    <div id="cpu-usage" style="width:100%;height:300px;"></div>
    <div id="memory-usage" style="width:100%;height:300px;"></div>
    <script>
        // Real-time performance monitoring
        setInterval(updateMetrics, 1000);
        
        function updateMetrics() {
            fetch('/api/metrics')
                .then(response => response.json())
                .then(data => {
                    updateChart('cpu-usage', data.cpu);
                    updateChart('memory-usage', data.memory);
                });
        }
    </script>
</body>
</html>
EOF
}
```

### Automated Testing Pipeline
```bash
# Continuous integration development workflow
dev_ci_pipeline() {
  # Run on every file change
  cargo watch -x "build --workspace" \
    -x "test --workspace" \
    -x "clippy --workspace -- -D warnings" \
    -x "fmt --all -- --check" \
    --watch src --watch core
}
```

### Docker Development Environment
```dockerfile
# Dockerfile.dev - Development container
FROM rust:1.70

# Install development tools
RUN cargo install cargo-watch cargo-flamegraph wasm-pack
RUN apt-get update && apt-get install -y \
    gdb valgrind perf nodejs npm python3

# Setup working directory
WORKDIR /workspace
COPY . .

# Start development environment
CMD ["cargo", "watch", "-x", "build", "-x", "test"]
```

### Hot Reload Configuration  
```toml
# .cargo/config.toml - Development configuration
[build]
target-dir = "target"

[env]
RUST_LOG = { value = "debug", force = true }
RUST_BACKTRACE = { value = "1", force = true }

[target.wasm32-unknown-unknown]
runner = "wasm-server-runner"
```