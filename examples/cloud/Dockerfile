# Multi-stage Dockerfile for LLM Shield API with cloud integrations
#
# Build with:
#   docker build --build-arg FEATURES=cloud-aws -t llm-shield-api .
#   docker build --build-arg FEATURES=cloud-gcp -t llm-shield-api .
#   docker build --build-arg FEATURES=cloud-azure -t llm-shield-api .
#   docker build --build-arg FEATURES=cloud-all -t llm-shield-api .

# Builder stage
FROM rust:1.75-slim as builder

# Install dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /build

# Copy workspace manifest
COPY Cargo.toml Cargo.lock ./

# Copy all crates
COPY crates/ crates/

# Build argument for cloud features
ARG FEATURES=cloud-all
ENV FEATURES=${FEATURES}

# Build the API with specified features
RUN cargo build --release \
    --package llm-shield-api \
    --features ${FEATURES} \
    && strip target/release/llm-shield-api

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 llmshield

# Set working directory
WORKDIR /app

# Copy binary from builder
COPY --from=builder /build/target/release/llm-shield-api /usr/local/bin/

# Copy configuration files
COPY examples/cloud/config-*.toml /app/config/

# Create models directory
RUN mkdir -p /app/models && chown -R llmshield:llmshield /app

# Switch to non-root user
USER llmshield

# Expose ports
EXPOSE 8080 9090

# Health check
HEALTHCHECK --interval=30s --timeout=5s --start-period=60s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Run the API
ENTRYPOINT ["llm-shield-api"]
CMD ["--config", "/app/config/config.toml"]
