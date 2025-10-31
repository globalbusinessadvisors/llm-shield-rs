# LLM Shield - Docker Quick Start

Get LLM Shield running with Docker in under 5 minutes.

## Prerequisites

- Docker 20.10+
- Docker Compose 2.0+
- 2GB RAM, 2 CPU cores

## Quick Start

```bash
# 1. Generate API key (development only)
echo "dev-key-$(openssl rand -hex 16)" > secrets/api_keys.txt

# 2. Start all services
docker-compose up -d

# 3. Wait for services to be healthy (~30 seconds)
docker-compose ps

# 4. Test API
curl http://localhost:8080/health

# Expected response:
# {"status":"healthy"}
```

## Access Services

| Service | URL | Credentials |
|---------|-----|-------------|
| **LLM Shield API** | http://localhost:8080 | API key from secrets/api_keys.txt |
| **Swagger UI** | http://localhost:8080/swagger-ui | - |
| **Prometheus** | http://localhost:9091 | - |
| **Grafana** | http://localhost:3000 | admin / admin |

## Test the API

```bash
# Get your API key
API_KEY=$(head -n 1 secrets/api_keys.txt | tail -n 1)

# Test health endpoint
curl http://localhost:8080/health

# Test scan endpoint (example)
curl -X POST http://localhost:8080/api/v1/scan \
  -H "Authorization: Bearer $API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "text": "This is a test message",
    "scanners": ["ban_substrings"]
  }'
```

## View Logs

```bash
# All services
docker-compose logs -f

# API only
docker-compose logs -f llm-shield

# Last 100 lines
docker-compose logs --tail=100 llm-shield
```

## Stop Services

```bash
# Stop services (keeps data)
docker-compose stop

# Stop and remove containers
docker-compose down

# Stop and remove everything (including volumes)
docker-compose down -v
```

## Monitoring

### Prometheus

1. Open http://localhost:9091
2. Try queries:
   ```promql
   # Request rate
   rate(http_requests_total[5m])

   # Error rate
   rate(http_requests_total{status=~"5.."}[5m]) / rate(http_requests_total[5m])

   # Latency (95th percentile)
   histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m]))
   ```

### Grafana

1. Open http://localhost:3000
2. Login: admin / admin
3. Navigate to Dashboards → Browse
4. Open "LLM Shield - Overview"

## Troubleshooting

### Port Already in Use

```bash
# Check what's using the port
sudo lsof -i :8080
sudo lsof -i :9091
sudo lsof -i :3000

# Stop the conflicting service or change ports in docker-compose.yml
```

### Container Won't Start

```bash
# Check logs
docker-compose logs llm-shield

# Check if API keys file exists
ls -la secrets/api_keys.txt

# Recreate containers
docker-compose down
docker-compose up -d
```

### Authentication Errors

```bash
# Verify API key is loaded
docker-compose exec llm-shield cat /run/secrets/api_keys

# Use the correct API key format
# Format: plain text, one key per line
```

## Production Deployment

**DO NOT use this configuration in production as-is!**

For production deployment:
- Use proper secret management (AWS Secrets Manager, Vault, etc.)
- Enable TLS/HTTPS
- Configure proper rate limiting
- Set up alerting
- Use specific image tags (not `latest`)
- Review security settings

See [docs/DOCKER_DEPLOYMENT.md](docs/DOCKER_DEPLOYMENT.md) for complete production guide.

## Next Steps

- **Full Documentation**: [docs/DOCKER_DEPLOYMENT.md](docs/DOCKER_DEPLOYMENT.md)
- **API Documentation**: http://localhost:8080/swagger-ui
- **Configuration**: [config/llm-shield.yml](config/llm-shield.yml)
- **Examples**: [examples/](examples/) directory

## Support

- **Issues**: https://github.com/llm-shield/llm-shield-rs/issues
- **Documentation**: https://docs.llm-shield.com
- **Security**: security@llm-shield.com
