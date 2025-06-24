# QuDAG CLI Documentation

Complete reference for the QuDAG Command Line Interface.

## Installation

```bash
# Install from source
cargo install --path tools/cli

# Or copy binary
cp target/debug/qudag-cli ~/.local/bin/qudag
```

## Command Reference

### Node Management

#### `qudag start`
Start a QuDAG node with P2P networking and consensus.

```bash
qudag start [OPTIONS]

Options:
  -p, --port <PORT>           Network port [default: 8000]
  -d, --data-dir <DIR>        Data directory [default: ./data]
  -l, --log-level <LEVEL>     Log level [default: info]
      --peer <ADDRESS>        Bootstrap peer (multiaddr format)
  -b, --background            Run in background (daemon mode)
  -h, --help                  Print help
```

**Examples:**
```bash
# Start with default settings
qudag start

# Start on custom port with debug logging
qudag start --port 8001 --log-level debug

# Start with bootstrap peers
qudag start --peer /ip4/192.168.1.100/tcp/8000 --peer /ip4/10.0.0.5/tcp/8000

# Start in background (daemon mode)
qudag start --background
```

### Dark Addressing

#### `qudag address register`
Register a .dark domain.

```bash
qudag address register <DOMAIN>

Arguments:
  <DOMAIN>    Domain name (e.g., mynode.dark)

Options:
      --ttl <SECONDS>    Time-to-live [default: 3600]
  -h, --help             Print help
```

#### `qudag address resolve`
Resolve a .dark domain to network addresses.

```bash
qudag address resolve <DOMAIN>

Arguments:
  <DOMAIN>    Domain to resolve (e.g., example.dark)

Options:
      --json     Output in JSON format
  -h, --help     Print help
```

For complete documentation, see the full CLI guide.