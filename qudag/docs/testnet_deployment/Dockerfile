# Build Stage 1: Chef for dependency caching
FROM lukemathwalker/cargo-chef:latest-rust-1.75 AS chef
WORKDIR /app

# Build Stage 2: Recipe planner
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Build Stage 3: Builder
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    cmake \
    clang \
    && rm -rf /var/lib/apt/lists/*

# Build dependencies - this is cached efficiently
RUN cargo chef cook --release --recipe-path recipe.json

# Copy source code
COPY . .

# Build the QuDAG node binary
RUN cargo build --release --bin qudag-node

# Runtime Stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1001 -s /bin/bash qudag

# Copy binary from builder
COPY --from=builder /app/target/release/qudag-node /usr/local/bin/qudag-node

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

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=40s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Entry point
ENTRYPOINT ["qudag-node"]
CMD ["--config", "/data/qudag/config.toml"]