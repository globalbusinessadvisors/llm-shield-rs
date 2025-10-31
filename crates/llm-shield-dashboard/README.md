# LLM Shield Dashboard

Enterprise-grade monitoring dashboard and analytics platform for LLM Shield.

## Features

- **Real-time Monitoring**: Track LLM Shield performance and security metrics in real-time
- **GraphQL API**: Flexible, type-safe API for querying metrics and managing resources
- **Multi-tenant Architecture**: Complete tenant isolation with row-level security
- **Time-series Database**: Powered by TimescaleDB for efficient metric storage and querying
- **Authentication**: Dual authentication support (JWT tokens and API keys)
- **Role-Based Access Control**: Four-tier RBAC (SuperAdmin, TenantAdmin, Developer, Viewer)
- **Health Checks**: Kubernetes-ready health, readiness, and liveness endpoints
- **Security Events**: Track and monitor security events with severity levels
- **Alert System**: Configurable alert rules with threshold-based triggering
- **Audit Logging**: Complete audit trail of all administrative actions

## Architecture

```
┌─────────────────┐
│   React SPA     │
└────────┬────────┘
         │
         │ GraphQL
         ▼
┌─────────────────┐
│  Axum Server    │
│  ┌───────────┐  │
│  │  GraphQL  │  │
│  │   API     │  │
│  └───────────┘  │
│  ┌───────────┐  │
│  │   Auth    │  │
│  │Middleware │  │
│  └───────────┘  │
└────────┬────────┘
         │
         │ sqlx
         ▼
┌─────────────────┐
│  TimescaleDB    │
│  PostgreSQL     │
└─────────────────┘
```

## Quick Start

### Prerequisites

- Rust 1.75+
- PostgreSQL 14+ with TimescaleDB 2.13+ extension
- Docker (optional, for development)

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
llm-shield-dashboard = "0.1.0"
```

### Database Setup

1. Install TimescaleDB extension:

```sql
CREATE EXTENSION IF NOT EXISTS timescaledb;
```

2. The dashboard will automatically run migrations on startup.

### Basic Usage

```rust
use llm_shield_dashboard::{
    config::{DashboardConfig, ServerConfig, DatabaseConfig, AuthConfig, CorsConfig, LoggingConfig},
    DashboardServer,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create configuration
    let config = DashboardConfig {
        server: ServerConfig {
            host: "0.0.0.0".to_string(),
            port: 8080,
            timeout_secs: 30,
            max_body_size: 10 * 1024 * 1024,
        },
        database: DatabaseConfig {
            url: "postgres://user:password@localhost:5432/llm_shield".to_string(),
            max_connections: 20,
            min_connections: 5,
            connection_timeout_secs: 30,
        },
        auth: AuthConfig {
            jwt_secret: "your-super-secret-jwt-key-min-32-chars".to_string(),
            jwt_expiration_secs: 900,
            refresh_token_expiration_secs: 604800,
            enable_api_keys: true,
            api_key_header: "X-API-Key".to_string(),
        },
        cors: CorsConfig {
            allowed_origins: vec!["http://localhost:3000".to_string()],
            allowed_methods: vec!["GET".to_string(), "POST".to_string()],
            allowed_headers: vec!["Content-Type".to_string(), "Authorization".to_string()],
            allow_credentials: true,
        },
        logging: LoggingConfig {
            level: "info".to_string(),
            format: "json".to_string(),
        },
    };

    // Create and start server
    let server = DashboardServer::new(config).await?;

    // Run migrations
    server.migrate().await?;

    // Start serving
    server.serve().await?;

    Ok(())
}
```

### Environment Variables

Configuration can be provided via environment variables with the `DASHBOARD__` prefix:

```bash
# Server configuration
export DASHBOARD__SERVER__HOST="0.0.0.0"
export DASHBOARD__SERVER__PORT="8080"

# Database configuration
export DASHBOARD__DATABASE__URL="postgres://user:password@localhost:5432/llm_shield"
export DASHBOARD__DATABASE__MAX_CONNECTIONS="20"

# Authentication configuration
export DASHBOARD__AUTH__JWT_SECRET="your-super-secret-jwt-key-min-32-chars"
export DASHBOARD__AUTH__JWT_EXPIRATION_SECS="900"
export DASHBOARD__AUTH__ENABLE_API_KEYS="true"

# CORS configuration
export DASHBOARD__CORS__ALLOWED_ORIGINS="http://localhost:3000,http://localhost:8080"

# Logging configuration
export RUST_LOG="info"
export DASHBOARD__LOGGING__FORMAT="json"
```

## API Endpoints

### Health Checks

- `GET /health` - Overall health status with database connection info
- `GET /health/ready` - Readiness check for Kubernetes
- `GET /health/live` - Liveness check for Kubernetes

### GraphQL

- `POST /graphql` - GraphQL API endpoint (requires authentication)
- `GET /graphql/playground` - Interactive GraphQL playground (requires authentication)

## Authentication

### JWT Tokens

Generate a JWT token for authentication:

```rust
use llm_shield_dashboard::auth::generate_token;
use uuid::Uuid;

let token = generate_token(
    user_id,
    tenant_id,
    "developer",
    "your-jwt-secret",
    900, // 15 minutes
)?;

// Use in requests:
// Authorization: Bearer <token>
```

### API Keys

API keys provide long-lived authentication for programmatic access:

```rust
use llm_shield_dashboard::auth::{generate_api_key, hash_api_key};

// Generate a new API key
let api_key = generate_api_key();
// Format: "llms_" + 32 random characters

// Hash before storing
let key_hash = hash_api_key(&api_key)?;

// Store key_hash in database, provide api_key to user once

// Use in requests:
// X-API-Key: llms_abc123...
```

## GraphQL Schema

### Queries

```graphql
type Query {
  # System queries
  version: String!
  health: Boolean!

  # Tenant queries
  tenant(id: UUID!): Tenant

  # User queries
  user(id: UUID!): User
}

type Tenant {
  id: UUID!
  name: String!
  display_name: String!
  settings: JSON!
  created_at: DateTime!
  updated_at: DateTime!
}

type User {
  id: UUID!
  tenant_id: UUID!
  email: String!
  role: UserRole!
  enabled: Boolean!
  created_at: DateTime!
  updated_at: DateTime!
}

enum UserRole {
  SUPER_ADMIN
  TENANT_ADMIN
  DEVELOPER
  VIEWER
}
```

### Example Queries

```graphql
# Get dashboard version
query {
  version
}

# Get tenant by ID
query {
  tenant(id: "123e4567-e89b-12d3-a456-426614174000") {
    id
    name
    display_name
  }
}

# Get user by ID
query {
  user(id: "123e4567-e89b-12d3-a456-426614174001") {
    id
    email
    role
    enabled
  }
}
```

## Database Schema

### Core Tables

- **tenants** - Multi-tenant isolation
- **users** - User accounts with RBAC
- **api_keys** - API key authentication

### Time-Series Tables (Hypertables)

- **metrics** - General metrics (90-day retention)
- **scanner_stats** - Scanner performance (1-year retention)
- **security_events** - Security event log (2-year retention)

### Management Tables

- **alert_rules** - Alert configuration
- **dashboards** - Dashboard definitions
- **audit_log** - Audit trail

### Continuous Aggregates

- **metrics_1min** - 1-minute metric rollups

## Security

### Authentication & Authorization

- **JWT Tokens**: Short-lived (15 minutes), signed with HS256
- **API Keys**: Long-lived, hashed with Argon2id
- **Password Hashing**: Argon2id algorithm
- **RBAC**: Four-tier role system with permission checks

### Multi-tenancy

- **Row-Level Security**: Tenant isolation enforced at database level
- **Tenant Context**: All queries scoped to authenticated user's tenant
- **API Key Scoping**: API keys tied to specific tenant

### Security Best Practices

1. **JWT Secret**: Use a strong secret (minimum 32 characters)
2. **HTTPS**: Always use HTTPS in production
3. **CORS**: Configure allowed origins restrictively
4. **API Keys**: Rotate regularly and use per-service keys
5. **Audit Logging**: Monitor audit logs for suspicious activity

## Performance

### Database Optimization

- **Hypertables**: Automatic partitioning for time-series data
- **Continuous Aggregates**: Pre-computed rollups
- **Retention Policies**: Automatic data lifecycle management
- **Indexes**: Optimized for common query patterns

### Connection Pooling

```rust
DatabaseConfig {
    max_connections: 20,     // Max pool size
    min_connections: 5,      // Min idle connections
    connection_timeout_secs: 30,
}
```

### Caching

- Connection pool statistics cached
- GraphQL schema cached at startup

## Monitoring

### Metrics

The dashboard exposes metrics through the `/health` endpoint:

```json
{
  "status": "healthy",
  "version": "0.1.0",
  "database": {
    "connected": true,
    "connections": 5,
    "idle_connections": 3
  }
}
```

### Logging

Structured logging with JSON or text format:

```bash
# JSON logging
export DASHBOARD__LOGGING__FORMAT="json"

# Text logging
export DASHBOARD__LOGGING__FORMAT="text"

# Log level
export RUST_LOG="info"
```

## Testing

### Unit Tests

```bash
cargo test
```

### Integration Tests

Requires PostgreSQL with TimescaleDB:

```bash
export TEST_DATABASE_URL="postgres://postgres:postgres@localhost:5432/llm_shield_test"
cargo test --test integration_tests -- --ignored
```

### Load Testing

Use tools like k6 or vegeta:

```javascript
// k6 example
import http from 'k6/http';

export default function() {
  const token = 'your-jwt-token';
  const query = '{ version }';

  http.post('http://localhost:8080/graphql', JSON.stringify({
    query: query
  }), {
    headers: {
      'Content-Type': 'application/json',
      'Authorization': `Bearer ${token}`
    }
  });
}
```

## Examples

See the `examples/` directory:

- `basic_server.rs` - Basic server setup

Run examples:

```bash
cargo run --example basic_server
```

## Deployment

### Docker

```dockerfile
FROM rust:1.75 AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libssl3 ca-certificates
COPY --from=builder /app/target/release/llm-shield-dashboard /usr/local/bin/
CMD ["llm-shield-dashboard"]
```

### Kubernetes

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: llm-shield-dashboard
spec:
  replicas: 3
  template:
    spec:
      containers:
      - name: dashboard
        image: llm-shield-dashboard:latest
        ports:
        - containerPort: 8080
        env:
        - name: DASHBOARD__DATABASE__URL
          valueFrom:
            secretKeyRef:
              name: database-secret
              key: url
        - name: DASHBOARD__AUTH__JWT_SECRET
          valueFrom:
            secretKeyRef:
              name: auth-secret
              key: jwt-secret
        livenessProbe:
          httpGet:
            path: /health/live
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 30
        readinessProbe:
          httpGet:
            path: /health/ready
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 10
```

## Roadmap

### Phase 1 (Week 1) - ✅ COMPLETED
- [x] Core infrastructure
- [x] Database schema with TimescaleDB
- [x] Authentication (JWT + API keys)
- [x] GraphQL API foundation
- [x] Health checks

### Phase 2 (Weeks 2-3)
- [ ] Additional GraphQL queries and mutations
- [ ] Real-time subscriptions
- [ ] Advanced metrics aggregation
- [ ] Alert rule processing

### Phase 3 (Weeks 4-6)
- [ ] React frontend dashboard
- [ ] WebSocket support for real-time updates
- [ ] Advanced visualizations
- [ ] User management UI

### Phase 4 (Weeks 7-9)
- [ ] SSO integration (SAML, OAuth)
- [ ] Advanced analytics
- [ ] Custom dashboard builder
- [ ] Report generation

### Phase 5 (Weeks 10-12)
- [ ] Multi-region support
- [ ] High availability setup
- [ ] Performance optimization
- [ ] Production hardening

## Contributing

Contributions are welcome! Please see the main repository for contribution guidelines.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../../LICENSE-APACHE))
- MIT license ([LICENSE-MIT](../../LICENSE-MIT))

at your option.
