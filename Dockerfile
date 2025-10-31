# Multi-stage Dockerfile for LLM Shield API
# Optimizes for size (<50MB), security (distroless), and build speed (layer caching)
#
# Build: docker build -t llm-shield-api:latest .
# Run:   docker run -p 8080:8080 llm-shield-api:latest

# ==============================================================================
# Stage 1: Build Environment
# ==============================================================================
FROM rust:1.75-slim-bookworm AS builder

WORKDIR /build

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace configuration
COPY Cargo.toml Cargo.lock ./

# Copy all crate manifests for dependency caching
COPY crates/llm-shield-core/Cargo.toml crates/llm-shield-core/
COPY crates/llm-shield-models/Cargo.toml crates/llm-shield-models/
COPY crates/llm-shield-scanners/Cargo.toml crates/llm-shield-scanners/
COPY crates/llm-shield-api/Cargo.toml crates/llm-shield-api/
COPY crates/llm-shield-wasm/Cargo.toml crates/llm-shield-wasm/
COPY crates/llm-shield-cli/Cargo.toml crates/llm-shield-cli/
COPY crates/llm-shield-py/Cargo.toml crates/llm-shield-py/

# Create dummy source files to build dependencies (cached layer)
RUN mkdir -p crates/llm-shield-core/src && \
    mkdir -p crates/llm-shield-models/src && \
    mkdir -p crates/llm-shield-scanners/src && \
    mkdir -p crates/llm-shield-api/src && \
    mkdir -p crates/llm-shield-wasm/src && \
    mkdir -p crates/llm-shield-cli/src && \
    mkdir -p crates/llm-shield-py/src && \
    echo "fn main() {}" > crates/llm-shield-api/src/main.rs && \
    echo "pub fn placeholder() {}" > crates/llm-shield-core/src/lib.rs && \
    echo "pub fn placeholder() {}" > crates/llm-shield-models/src/lib.rs && \
    echo "pub fn placeholder() {}" > crates/llm-shield-scanners/src/lib.rs && \
    echo "pub fn placeholder() {}" > crates/llm-shield-wasm/src/lib.rs && \
    echo "fn main() {}" > crates/llm-shield-cli/src/main.rs && \
    echo "pub fn placeholder() {}" > crates/llm-shield-py/src/lib.rs

# Build dependencies only (this layer will be cached)
RUN cargo build --release --bin llm-shield-api && \
    rm -rf target/release/deps/llm_shield*

# Copy actual source code
COPY crates ./crates

# Build the application
RUN cargo build --release --bin llm-shield-api

# Strip debug symbols to reduce binary size
RUN strip target/release/llm-shield-api

# ==============================================================================
# Stage 2: Runtime Environment (Distroless for Security)
# ==============================================================================
FROM gcr.io/distroless/cc-debian12

# Metadata
LABEL maintainer="LLM Shield Team"
LABEL description="LLM Shield API - Production-ready LLM security scanning"
LABEL version="1.0"

# Copy binary from builder
COPY --from=builder /build/target/release/llm-shield-api /usr/local/bin/llm-shield-api

# Copy ML models if they exist (optional)
# COPY --from=builder /build/models /opt/llm-shield/models

# Non-root user (distroless default: nonroot:nonroot UID/GID 65532)
USER nonroot:nonroot

# Expose API port (8080) and metrics port (9090)
EXPOSE 8080 9090

# Health check (requires curl in distroless - use custom health check externally)
# HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
#     CMD ["/usr/local/bin/llm-shield-api", "health"]

# Run the application
ENTRYPOINT ["/usr/local/bin/llm-shield-api"]
CMD ["--host", "0.0.0.0", "--port", "8080"]

# ==============================================================================
# Build Information
# ==============================================================================
# Expected image size: ~50MB (vs ~500MB with full Rust image)
# Security features:
#   - Distroless base (minimal attack surface, no shell)
#   - Non-root user
#   - Read-only root filesystem (set via K8s securityContext)
#   - Dropped capabilities (set via K8s securityContext)
#
# Build time optimization:
#   - Dependency caching: Only rebuilds when Cargo.toml changes
#   - Multi-stage: Discards build dependencies (~450MB savings)
#   - Binary stripping: Removes debug symbols (~20% size reduction)
