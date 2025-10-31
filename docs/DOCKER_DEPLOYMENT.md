# Docker Deployment Guide

Complete guide for deploying LLM Shield using Docker and Docker Compose.

## Table of Contents

1. [Quick Start](#quick-start)
2. [Prerequisites](#prerequisites)
3. [Building the Image](#building-the-image)
4. [Running with Docker](#running-with-docker)
5. [Running with Docker Compose](#running-with-docker-compose)
6. [Configuration](#configuration)
7. [Monitoring Stack](#monitoring-stack)
8. [Security](#security)
9. [Troubleshooting](#troubleshooting)
10. [Production Deployment](#production-deployment)

---

## Quick Start

Get LLM Shield running in under 5 minutes:

```bash
# 1. Clone the repository
git clone https://github.com/llm-shield/llm-shield-rs.git
cd llm-shield-rs

# 2. Generate API keys (development only)
echo "dev-api-key-$(openssl rand -hex 16)" > secrets/api_keys.txt

# 3. Start all services
docker-compose up -d

# 4. Check health
curl http://localhost:8080/health

# 5. Access services
# API:        http://localhost:8080
# Swagger:    http://localhost:8080/swagger-ui
# Prometheus: http://localhost:9091
# Grafana:    http://localhost:3000 (admin/admin)
```

---

## Prerequisites

### Required

- **Docker**: 20.10 or later
- **Docker Compose**: 2.0 or later (V2 syntax)
- **System Resources**:
  - CPU: 2 cores minimum, 4 cores recommended
  - RAM: 2GB minimum, 4GB recommended
  - Disk: 5GB free space

### Optional

- **Rust toolchain**: For local development (not needed for Docker-only deployment)
- **make**: For convenience commands

### Installation

#### Linux

```bash
# Docker
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh

# Docker Compose (included in Docker Desktop)
sudo apt-get install docker-compose-plugin
```

#### macOS

```bash
# Install Docker Desktop
brew install --cask docker
```

#### Windows

Download and install [Docker Desktop for Windows](https://docs.docker.com/desktop/install/windows-install/)

---

## Building the Image

### Standard Build

```bash
# Build with default settings
docker build -t llm-shield-api:latest .

# Check image size
docker images llm-shield-api:latest
# Expected: ~50MB
```

### Build Arguments

```bash
# Use different Rust version
docker build \
  --build-arg RUST_VERSION=1.76 \
  -t llm-shield-api:1.76 \
  .

# Build with specific target
docker build \
  --build-arg CARGO_BUILD_TARGET=x86_64-unknown-linux-musl \
  -t llm-shield-api:musl \
  .
```

### Multi-Platform Build

```bash
# Build for multiple architectures
docker buildx build \
  --platform linux/amd64,linux/arm64 \
  -t llm-shield-api:latest \
  --push \
  .
```

### Build Optimization

```bash
# Use BuildKit for faster builds
DOCKER_BUILDKIT=1 docker build \
  --cache-from llm-shield-api:latest \
  -t llm-shield-api:latest \
  .

# Build with layer caching
docker build \
  --cache-from ghcr.io/llm-shield/llm-shield-api:cache \
  --build-arg BUILDKIT_INLINE_CACHE=1 \
  -t llm-shield-api:latest \
  .
```

---

## Running with Docker

### Basic Run

```bash
# Run with default settings
docker run -d \
  -p 8080:8080 \
  --name llm-shield \
  llm-shield-api:latest
```

### With Environment Variables

```bash
docker run -d \
  -p 8080:8080 \
  -p 9090:9090 \
  -e RUST_LOG=debug \
  -e LLM_SHIELD_AUTH_ENABLED=false \
  --name llm-shield \
  llm-shield-api:latest
```

### With Volume Mounts

```bash
docker run -d \
  -p 8080:8080 \
  -v $(pwd)/config:/etc/llm-shield:ro \
  -v $(pwd)/secrets/api_keys.txt:/run/secrets/api_keys:ro \
  -v $(pwd)/models:/opt/llm-shield/models:ro \
  --name llm-shield \
  llm-shield-api:latest
```

### With Resource Limits

```bash
docker run -d \
  -p 8080:8080 \
  --memory="1g" \
  --cpus="2.0" \
  --restart=unless-stopped \
  --name llm-shield \
  llm-shield-api:latest
```

---

## Running with Docker Compose

### Start Services

```bash
# Start all services in background
docker-compose up -d

# Start with build
docker-compose up -d --build

# View logs
docker-compose logs -f llm-shield

# View all logs
docker-compose logs -f
```

### Stop Services

```bash
# Stop services
docker-compose stop

# Stop and remove containers
docker-compose down

# Stop and remove volumes (CAUTION: deletes data)
docker-compose down -v
```

### Individual Service Management

```bash
# Restart API only
docker-compose restart llm-shield

# View API logs
docker-compose logs -f llm-shield

# Execute command in container
docker-compose exec llm-shield sh
```

### Service Health

```bash
# Check all service status
docker-compose ps

# Check API health
curl http://localhost:8080/health

# Check Prometheus targets
curl http://localhost:9091/api/v1/targets
```

---

## Configuration

### Environment Variables

LLM Shield supports configuration via environment variables:

| Variable | Default | Description |
|----------|---------|-------------|
| `RUST_LOG` | `info` | Log level (trace, debug, info, warn, error) |
| `LLM_SHIELD_HOST` | `0.0.0.0` | Server bind address |
| `LLM_SHIELD_PORT` | `8080` | API port |
| `LLM_SHIELD_METRICS_PORT` | `9090` | Metrics port |
| `LLM_SHIELD_AUTH_ENABLED` | `true` | Enable API key authentication |
| `LLM_SHIELD_API_KEYS_FILE` | `/run/secrets/api_keys` | API keys file path |
| `LLM_SHIELD_RATE_LIMIT_REQUESTS` | `100` | Rate limit (requests per window) |
| `LLM_SHIELD_RATE_LIMIT_WINDOW_SECS` | `60` | Rate limit window (seconds) |
| `LLM_SHIELD_CORS_ENABLED` | `true` | Enable CORS |
| `LLM_SHIELD_MAX_BODY_SIZE` | `1048576` | Max request body size (bytes) |

### Configuration File

Edit `config/llm-shield.yml` for detailed configuration:

```yaml
server:
  host: "0.0.0.0"
  port: 8080
  timeout: 30

auth:
  enabled: true
  api_keys:
    file: "/run/secrets/api_keys"

rate_limit:
  enabled: true
  requests_per_window: 100
  window_seconds: 60

# See config/llm-shield.yml for full options
```

### API Keys

For local development:

```bash
# Generate secure API key
openssl rand -base64 32 > secrets/api_keys.txt

# Or multiple keys
cat > secrets/api_keys.txt <<EOF
dev-client-1-$(openssl rand -hex 16)
dev-client-2-$(openssl rand -hex 16)
EOF
```

For production, use secret management (see [Security](#security)).

---

## Monitoring Stack

### Accessing Services

| Service | URL | Credentials | Purpose |
|---------|-----|-------------|---------|
| LLM Shield API | http://localhost:8080 | API key | REST API |
| Swagger UI | http://localhost:8080/swagger-ui | - | API documentation |
| Prometheus | http://localhost:9091 | - | Metrics collection |
| Grafana | http://localhost:3000 | admin/admin | Dashboards |
| Loki | http://localhost:3100 | - | Log aggregation |

### Prometheus Metrics

Key metrics to monitor:

```promql
# Request rate
rate(http_requests_total[5m])

# Error rate
rate(http_requests_total{status=~"5.."}[5m]) / rate(http_requests_total[5m])

# Latency (p95)
histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m]))

# Memory usage
process_resident_memory_bytes / 1024 / 1024

# Scanner performance
histogram_quantile(0.95, rate(scanner_duration_seconds_bucket[5m]))
```

### Grafana Dashboards

Pre-configured dashboards are available at:

```
monitoring/grafana/dashboards/llm-shield-overview.json
```

Import additional dashboards:
1. Access Grafana: http://localhost:3000
2. Navigate to Dashboards → Import
3. Upload JSON file or paste JSON

### Alerting

Alert rules are defined in `monitoring/prometheus/alerts.yml`:

- **APIDown**: API is unreachable for > 1 minute
- **APIHighErrorRate**: Error rate > 5% for 5 minutes
- **HighMemoryUsage**: Memory usage > 800MB
- **ScannerHighFailureRate**: Scanner failures > 10%

### Logs

View logs with Promtail + Loki:

```bash
# Docker Compose logs
docker-compose logs -f llm-shield

# Query Loki directly
curl -G -s http://localhost:3100/loki/api/v1/query \
  --data-urlencode 'query={job="llm-shield-api"}' \
  | jq .

# View in Grafana
# Navigate to Explore → Select Loki → Query: {job="llm-shield-api"}
```

---

## Security

### Container Security

The Docker image follows security best practices:

1. **Distroless base image**: Minimal attack surface, no shell
2. **Non-root user**: Runs as UID 65532 (nonroot)
3. **Read-only root filesystem**: Enabled via `securityContext` in K8s
4. **Dropped capabilities**: No privileged operations
5. **No secrets in image**: Secrets mounted at runtime

### Vulnerability Scanning

Scan the image for vulnerabilities:

```bash
# Using Trivy
docker run --rm -v /var/run/docker.sock:/var/run/docker.sock \
  aquasec/trivy:latest image llm-shield-api:latest

# Using Snyk
snyk container test llm-shield-api:latest

# Using Docker Scout
docker scout cves llm-shield-api:latest
```

### Secret Management

**DO NOT** commit secrets to Git!

#### Development

Use local files (already in `.gitignore`):

```bash
echo "my-dev-key-$(openssl rand -hex 16)" > secrets/api_keys.txt
```

#### Production

Use proper secret management:

**Docker Secrets** (Swarm):
```bash
echo "production-key" | docker secret create llm_shield_api_keys -
docker service create \
  --secret llm_shield_api_keys \
  llm-shield-api:latest
```

**Environment variables** (encrypted):
```bash
# Using encrypted .env file
ansible-vault encrypt .env
docker-compose --env-file <(ansible-vault view .env) up -d
```

**External secret managers**:
- AWS Secrets Manager + ECS
- HashiCorp Vault
- Azure Key Vault
- GCP Secret Manager

See `secrets/README.md` for detailed instructions.

### Network Security

```yaml
# Restrict network access in docker-compose.yml
networks:
  llm-shield-net:
    driver: bridge
    internal: true  # No external access

services:
  llm-shield:
    networks:
      - llm-shield-net
    # Only expose necessary ports
    ports:
      - "127.0.0.1:8080:8080"  # Bind to localhost only
```

### TLS/HTTPS

For production, use TLS termination:

**Option 1: Nginx reverse proxy**
```yaml
services:
  nginx:
    image: nginx:alpine
    ports:
      - "443:443"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf:ro
      - ./certs:/etc/nginx/certs:ro
```

**Option 2: Traefik**
```yaml
services:
  traefik:
    image: traefik:v2.10
    command:
      - --providers.docker=true
      - --entrypoints.websecure.address=:443
      - --certificatesresolvers.letsencrypt.acme.email=admin@example.com
```

---

## Troubleshooting

### Common Issues

#### Container Won't Start

```bash
# Check logs
docker-compose logs llm-shield

# Common causes:
# 1. Port already in use
sudo lsof -i :8080
sudo lsof -i :9090

# 2. Missing secrets file
ls -la secrets/api_keys.txt

# 3. Invalid configuration
docker-compose config  # Validate syntax
```

#### High Memory Usage

```bash
# Check memory usage
docker stats llm-shield

# Limit memory
docker update --memory="1g" llm-shield

# Or in docker-compose.yml:
services:
  llm-shield:
    deploy:
      resources:
        limits:
          memory: 1G
```

#### Slow Performance

```bash
# Check CPU usage
docker stats llm-shield

# Increase CPU limit
docker update --cpus="4.0" llm-shield

# Check scanner performance
curl http://localhost:9090/metrics | grep scanner_duration
```

#### Authentication Errors

```bash
# Verify API key is loaded
docker-compose exec llm-shield cat /run/secrets/api_keys

# Test authentication
curl -H "Authorization: Bearer dev-api-key-12345" \
  http://localhost:8080/health

# Check auth metrics
curl http://localhost:9090/metrics | grep auth
```

### Debug Mode

```bash
# Enable debug logging
docker-compose stop llm-shield
docker-compose run -e RUST_LOG=debug llm-shield

# Or edit docker-compose.yml:
environment:
  - RUST_LOG=debug,llm_shield=trace
```

### Health Checks

```bash
# API health
curl http://localhost:8080/health
# Expected: {"status":"healthy"}

# Detailed health (if enabled)
curl http://localhost:8080/health?detailed=true

# Prometheus scrape health
curl http://localhost:9091/-/healthy

# Grafana health
curl http://localhost:3000/api/health
```

---

## Production Deployment

### Production Checklist

- [ ] Use production-grade secret management (not plain text files)
- [ ] Enable TLS/HTTPS with valid certificates
- [ ] Configure proper rate limiting per client
- [ ] Set up log retention and rotation
- [ ] Enable alerting (Prometheus AlertManager)
- [ ] Configure backups for Prometheus/Grafana data
- [ ] Set resource limits (CPU, memory)
- [ ] Enable security scanning in CI/CD
- [ ] Use specific image tags (not `latest`)
- [ ] Configure health checks and restart policies
- [ ] Set up monitoring and observability
- [ ] Review and harden network policies

### Production Configuration

```yaml
# docker-compose.prod.yml
version: '3.9'

services:
  llm-shield:
    image: ghcr.io/llm-shield/llm-shield-api:v1.0.0  # Specific version
    restart: always
    read_only: true  # Read-only filesystem
    cap_drop:
      - ALL  # Drop all capabilities
    security_opt:
      - no-new-privileges:true
    deploy:
      resources:
        limits:
          cpus: '2.0'
          memory: 1G
        reservations:
          cpus: '0.5'
          memory: 256M
    environment:
      - RUST_LOG=info  # Production log level
      - LLM_SHIELD_AUTH_ENABLED=true
    secrets:
      - api_keys
    healthcheck:
      test: ["CMD-SHELL", "wget --no-verbose --tries=1 --spider http://localhost:8080/health || exit 1"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s
```

### Deployment Commands

```bash
# Deploy with production config
docker-compose -f docker-compose.yml -f docker-compose.prod.yml up -d

# Zero-downtime deployment
docker-compose -f docker-compose.prod.yml pull
docker-compose -f docker-compose.prod.yml up -d --no-deps --build llm-shield

# Rollback
docker-compose -f docker-compose.prod.yml up -d llm-shield:v1.0.0
```

### Monitoring in Production

```yaml
# Add cAdvisor for container metrics
services:
  cadvisor:
    image: gcr.io/cadvisor/cadvisor:latest
    volumes:
      - /:/rootfs:ro
      - /var/run:/var/run:ro
      - /sys:/sys:ro
      - /var/lib/docker:/var/lib/docker:ro
    ports:
      - 8081:8080
```

### Backup and Recovery

```bash
# Backup Prometheus data
docker-compose exec prometheus tar czf - /prometheus \
  > prometheus-backup-$(date +%Y%m%d).tar.gz

# Backup Grafana dashboards
docker-compose exec grafana tar czf - /var/lib/grafana \
  > grafana-backup-$(date +%Y%m%d).tar.gz

# Restore
docker-compose stop prometheus
docker-compose exec prometheus tar xzf - < prometheus-backup.tar.gz
docker-compose start prometheus
```

---

## Next Steps

- **Kubernetes Deployment**: See [KUBERNETES_DEPLOYMENT.md](./KUBERNETES_DEPLOYMENT.md) (Coming in Week 2-3)
- **Cloud Deployment**: See [TERRAFORM_DEPLOYMENT.md](./TERRAFORM_DEPLOYMENT.md) (Coming in Week 4)
- **API Documentation**: Visit http://localhost:8080/swagger-ui
- **Examples**: See `examples/` directory for integration samples

## Support

- **Documentation**: https://docs.llm-shield.com
- **Issues**: https://github.com/llm-shield/llm-shield-rs/issues
- **Discussions**: https://github.com/llm-shield/llm-shield-rs/discussions
- **Security**: security@llm-shield.com
