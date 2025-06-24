# Build Stage: Compile QuDAG node stub for quick deployment
FROM rust:1.75 AS builder
WORKDIR /app

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    cmake \
    clang \
    && rm -rf /var/lib/apt/lists/*

# Copy the HTTP-enabled stub source
COPY configs/qudag-node-http.rs ./src/main.rs

# Create a minimal Cargo.toml with serde_json dependency
RUN echo '[package]' > Cargo.toml && \
    echo 'name = "qudag-node"' >> Cargo.toml && \
    echo 'version = "1.0.0"' >> Cargo.toml && \
    echo 'edition = "2021"' >> Cargo.toml && \
    echo '' >> Cargo.toml && \
    echo '[dependencies]' >> Cargo.toml && \
    echo 'serde_json = "1.0"' >> Cargo.toml && \
    echo '' >> Cargo.toml && \
    echo '[[bin]]' >> Cargo.toml && \
    echo 'name = "qudag-node"' >> Cargo.toml && \
    echo 'path = "src/main.rs"' >> Cargo.toml

# Build the node
RUN cargo build --release

# Runtime Stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1001 -s /bin/bash qudag

# Copy binary from builder
COPY --from=builder /app/target/release/qudag-node /usr/local/bin/qudag-node

# Copy configuration files
COPY configs/*.toml /app/configs/

# Copy startup script
COPY startup.sh /usr/local/bin/startup.sh
RUN chmod +x /usr/local/bin/startup.sh

# Create data directories
RUN mkdir -p /data/qudag && chown -R qudag:qudag /data

# Switch to non-root user
USER qudag

# Set working directory
WORKDIR /data/qudag

# Environment variables for QuDAG configuration
ENV RUST_LOG=info
ENV QUDAG_DATA_DIR=/data/qudag
ENV QUDAG_P2P_PORT=4001
ENV QUDAG_RPC_PORT=8080
ENV QUDAG_METRICS_PORT=9090

# Expose ports
# P2P networking
EXPOSE 4001/tcp
EXPOSE 4001/udp
# RPC API
EXPOSE 8080/tcp
# Metrics
EXPOSE 9090/tcp

# Health check - works for both node types
HEALTHCHECK --interval=30s --timeout=10s --start-period=40s --retries=3 \
    CMD curl -f http://localhost:${QUDAG_RPC_PORT:-8080}/health || \
        curl -f http://localhost:${QUDAG_RPC_PORT:-8080}/api/health || \
        exit 1

# Entry point - uses startup script that selects binary based on NODE_TYPE
ENTRYPOINT ["/usr/local/bin/startup.sh"]
CMD ["--config", "/data/qudag/config.toml"]