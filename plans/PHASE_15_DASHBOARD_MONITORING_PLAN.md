# Phase 15: Dashboard and Monitoring - Implementation Plan

**Document Version**: 1.0
**Created**: 2025-10-31
**Status**: Planning Phase
**Estimated Duration**: 10-12 weeks
**Target**: Q1 2025

---

## Executive Summary

Phase 15 will deliver a comprehensive, enterprise-grade monitoring and dashboard solution for LLM Shield, providing real-time visibility into API performance, security events, scanner effectiveness, and operational metrics. The solution will be commercially viable, scalable, and secure, meeting enterprise requirements for multi-tenancy, compliance, and observability.

### Key Objectives

1. **Real-time Visibility**: Live monitoring of API health, scanner performance, and security events
2. **Historical Analysis**: Time-series data for trend analysis and capacity planning
3. **Security Monitoring**: Threat detection, anomaly detection, and security event tracking
4. **Cost Management**: Usage tracking, cost allocation, and optimization insights
5. **Enterprise Features**: Multi-tenancy, RBAC, SSO, audit logging, and compliance reporting
6. **Alerting**: Intelligent alerting with multiple notification channels
7. **Self-Service Analytics**: Customizable dashboards and reports for different user roles

### Success Metrics

- Dashboard load time: < 2 seconds
- Real-time metric lag: < 5 seconds
- Query performance: < 1 second for standard queries
- Uptime: 99.9% availability
- Support 10,000+ concurrent dashboard users
- Handle 1M+ metrics per minute

---

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Technology Stack](#technology-stack)
3. [Features Breakdown](#features-breakdown)
4. [Implementation Phases](#implementation-phases)
5. [Security Architecture](#security-architecture)
6. [Data Model](#data-model)
7. [User Experience](#user-experience)
8. [Deployment Strategy](#deployment-strategy)
9. [Cost Analysis](#cost-analysis)
10. [Timeline](#timeline)
11. [Success Criteria](#success-criteria)
12. [Risk Assessment](#risk-assessment)

---

## Architecture Overview

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         User Layer                               │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐       │
│  │ Web UI   │  │ Mobile   │  │ CLI      │  │ API      │       │
│  │ (React)  │  │ (PWA)    │  │ Client   │  │ Client   │       │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘       │
└─────────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Dashboard API (Rust/Axum)                     │
│  ┌───────────────┐  ┌───────────────┐  ┌──────────────┐       │
│  │ Query Engine  │  │ Aggregation   │  │ Alert        │       │
│  │ (GraphQL)     │  │ Service       │  │ Manager      │       │
│  └───────────────┘  └───────────────┘  └──────────────┘       │
└─────────────────────────────────────────────────────────────────┘
                            │
        ┌───────────────────┼───────────────────┐
        ▼                   ▼                   ▼
┌──────────────┐  ┌──────────────┐  ┌──────────────┐
│ Time-Series  │  │ Relational   │  │ Cache        │
│ DB           │  │ DB           │  │ (Redis)      │
│ (TimescaleDB)│  │ (PostgreSQL) │  │              │
└──────────────┘  └──────────────┘  └──────────────┘
        ▲                   ▲                   ▲
        │                   │                   │
┌─────────────────────────────────────────────────────────────────┐
│                    Data Ingestion Layer                          │
│  ┌───────────────┐  ┌───────────────┐  ┌──────────────┐       │
│  │ Metrics       │  │ Logs          │  │ Traces       │       │
│  │ Collector     │  │ Aggregator    │  │ Collector    │       │
│  │ (Prometheus)  │  │ (Vector)      │  │ (Jaeger)     │       │
│  └───────────────┘  └───────────────┘  └──────────────┘       │
└─────────────────────────────────────────────────────────────────┘
        ▲                   ▲                   ▲
        │                   │                   │
┌─────────────────────────────────────────────────────────────────┐
│                    LLM Shield API Layer                          │
│  ┌───────────────┐  ┌───────────────┐  ┌──────────────┐       │
│  │ Metrics       │  │ Structured    │  │ Distributed  │       │
│  │ Export        │  │ Logging       │  │ Tracing      │       │
│  └───────────────┘  └───────────────┘  └──────────────┘       │
└─────────────────────────────────────────────────────────────────┘
```

### Component Architecture

#### 1. Frontend Layer (React + TypeScript)

**Purpose**: User-facing dashboard application

**Components**:
- **Dashboard UI**: Real-time metrics visualization
- **Query Builder**: Interactive query interface
- **Alert Manager**: Alert configuration and management
- **User Management**: RBAC and team management
- **Report Builder**: Custom report generation

**Technology**:
- React 18 with TypeScript
- Recharts/Apache ECharts for visualizations
- TanStack Query for data fetching
- Zustand for state management
- WebSocket/SSE for real-time updates
- Tailwind CSS for styling

#### 2. Backend API (Rust + Axum)

**Purpose**: Dashboard backend and query engine

**Components**:
- **GraphQL API**: Flexible query interface
- **Authentication**: OAuth2/OIDC, API keys
- **Authorization**: RBAC/ABAC enforcement
- **Query Engine**: Optimized metric queries
- **Aggregation Service**: Real-time metric aggregation
- **Alert Manager**: Alert evaluation and notification
- **Report Generator**: Scheduled report generation

**Crate Structure**:
```
crates/
└── llm-shield-dashboard/
    ├── Cargo.toml
    ├── README.md
    └── src/
        ├── lib.rs
        ├── api/          # GraphQL/REST API
        ├── auth/         # Authentication/Authorization
        ├── query/        # Query engine
        ├── aggregation/  # Metric aggregation
        ├── alerts/       # Alert management
        ├── reports/      # Report generation
        └── websocket/    # Real-time updates
```

#### 3. Data Storage Layer

**Time-Series Database: TimescaleDB**

Why TimescaleDB:
- PostgreSQL-based (familiar, robust)
- Excellent query performance (10-100x faster than PostgreSQL for time-series)
- Native SQL support (no new query language)
- Automatic partitioning and retention policies
- Continuous aggregates for pre-computation
- Cost-effective (open-source, self-hosted)

Alternative: InfluxDB (if pure time-series focus)

**Relational Database: PostgreSQL**

Purpose: User data, configurations, alert rules, reports

**Caching Layer: Redis**

Purpose: Session storage, query caching, rate limiting

#### 4. Data Ingestion Layer

**Metrics Collection: Prometheus + Custom Exporter**

- Pull-based metric collection
- Service discovery
- Custom LLM Shield metrics exporter

**Log Aggregation: Vector (Rust-based)**

- High-performance log collection
- Transformation and routing
- Multiple destination support (TimescaleDB, S3, Elasticsearch)

**Distributed Tracing: Jaeger**

- OpenTelemetry compatible
- Request tracing across services
- Performance profiling

---

## Technology Stack

### Frontend

| Component | Technology | Justification |
|-----------|-----------|---------------|
| **Framework** | React 18 + TypeScript | Industry standard, rich ecosystem, excellent TypeScript support |
| **Build Tool** | Vite | Fast builds, HMR, optimized production bundles |
| **State Management** | Zustand | Lightweight, TypeScript-friendly, simple API |
| **Data Fetching** | TanStack Query | Automatic caching, refetching, optimistic updates |
| **Charting** | Apache ECharts | Feature-rich, performant, 100+ chart types |
| **Tables** | TanStack Table | Headless, flexible, virtualizing for large datasets |
| **Forms** | React Hook Form | Performant, minimal re-renders, great DX |
| **Styling** | Tailwind CSS | Utility-first, consistent design, fast development |
| **Real-time** | WebSocket (Socket.io) | Bidirectional, reliable, fallback support |
| **Authentication** | Auth0/Keycloak SDK | Enterprise SSO, OAuth2/OIDC |
| **Testing** | Vitest + Testing Library | Fast, compatible with Vite, comprehensive |

### Backend

| Component | Technology | Justification |
|-----------|-----------|---------------|
| **Framework** | Axum 0.7 | Fast, type-safe, ergonomic Rust web framework |
| **API** | GraphQL (async-graphql) | Flexible queries, type-safe, self-documenting |
| **Authentication** | OAuth2/OIDC | Industry standard, enterprise SSO support |
| **Authorization** | Casbin | Flexible RBAC/ABAC, policy-based |
| **Database** | TimescaleDB + PostgreSQL | Time-series + relational, single stack |
| **ORM** | SQLx | Compile-time checked queries, async, performant |
| **Cache** | Redis 7 | Fast, reliable, rich data structures |
| **Message Queue** | Redis Streams | Simple, integrated with Redis, sufficient throughput |
| **Tracing** | OpenTelemetry | Standard, cloud-native, vendor-neutral |
| **Metrics** | Prometheus | De facto standard, rich ecosystem |

### Infrastructure

| Component | Technology | Justification |
|-----------|-----------|---------------|
| **Container** | Docker | Standard containerization |
| **Orchestration** | Kubernetes | Enterprise-grade, cloud-agnostic |
| **Service Mesh** | Istio (optional) | Advanced networking, security, observability |
| **CI/CD** | GitHub Actions | Integrated, flexible, free for OSS |
| **Monitoring** | Prometheus + Grafana | Best-in-class, open-source |
| **Alerting** | Alertmanager | Native Prometheus integration |
| **Logging** | Vector + Loki | Rust-based, performant, Grafana integration |

---

## Features Breakdown

### Phase 15.1: Core Dashboard Infrastructure (Weeks 1-3)

#### 1.1 Backend API Foundation

**Deliverables**:
- ✅ Crate structure setup (`llm-shield-dashboard`)
- ✅ Axum server with GraphQL endpoint
- ✅ TimescaleDB schema design and migrations
- ✅ Basic authentication (API keys)
- ✅ Health check endpoints
- ✅ Error handling and logging

**GraphQL Schema (Initial)**:
```graphql
type Query {
  # Metrics
  getMetrics(
    timeRange: TimeRange!
    filters: MetricFilters
    aggregation: AggregationType
  ): [MetricDataPoint!]!

  # Scanner performance
  getScannerStats(
    scanner: String!
    timeRange: TimeRange!
  ): ScannerStats!

  # API usage
  getApiUsage(
    timeRange: TimeRange!
    groupBy: GroupBy!
  ): [UsageDataPoint!]!

  # Security events
  getSecurityEvents(
    timeRange: TimeRange!
    severity: [Severity!]
    limit: Int = 100
  ): [SecurityEvent!]!
}

type Mutation {
  # Alerts
  createAlert(input: AlertInput!): Alert!
  updateAlert(id: ID!, input: AlertInput!): Alert!
  deleteAlert(id: ID!): Boolean!

  # Dashboards
  saveDashboard(input: DashboardInput!): Dashboard!
}

type Subscription {
  # Real-time metrics
  metricsStream(filters: MetricFilters): MetricDataPoint!

  # Security events
  securityEventsStream: SecurityEvent!

  # Alerts
  alertsStream: Alert!
}
```

**Database Schema**:
```sql
-- Time-series metrics
CREATE TABLE metrics (
  time TIMESTAMPTZ NOT NULL,
  metric_name TEXT NOT NULL,
  metric_value DOUBLE PRECISION NOT NULL,
  labels JSONB,
  tenant_id UUID NOT NULL
);

SELECT create_hypertable('metrics', 'time');

-- Continuous aggregates for performance
CREATE MATERIALIZED VIEW metrics_1min
WITH (timescaledb.continuous) AS
SELECT
  time_bucket('1 minute', time) AS bucket,
  metric_name,
  labels->>'scanner' AS scanner,
  AVG(metric_value) AS avg_value,
  MAX(metric_value) AS max_value,
  MIN(metric_value) AS min_value,
  COUNT(*) AS count,
  tenant_id
FROM metrics
GROUP BY bucket, metric_name, scanner, tenant_id;

-- Scanner statistics
CREATE TABLE scanner_stats (
  time TIMESTAMPTZ NOT NULL,
  scanner_name TEXT NOT NULL,
  requests_total BIGINT NOT NULL,
  requests_valid BIGINT NOT NULL,
  requests_invalid BIGINT NOT NULL,
  avg_latency_ms DOUBLE PRECISION NOT NULL,
  p95_latency_ms DOUBLE PRECISION NOT NULL,
  p99_latency_ms DOUBLE PRECISION NOT NULL,
  tenant_id UUID NOT NULL
);

SELECT create_hypertable('scanner_stats', 'time');

-- Security events
CREATE TABLE security_events (
  time TIMESTAMPTZ NOT NULL,
  event_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  event_type TEXT NOT NULL,
  severity TEXT NOT NULL,
  description TEXT NOT NULL,
  metadata JSONB,
  tenant_id UUID NOT NULL,
  user_id UUID
);

SELECT create_hypertable('security_events', 'time');

-- Alert rules
CREATE TABLE alert_rules (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  name TEXT NOT NULL,
  description TEXT,
  query TEXT NOT NULL,
  threshold DOUBLE PRECISION NOT NULL,
  operator TEXT NOT NULL CHECK (operator IN ('>', '<', '>=', '<=', '=', '!=')),
  duration_seconds INTEGER NOT NULL,
  severity TEXT NOT NULL,
  enabled BOOLEAN DEFAULT true,
  notification_channels TEXT[],
  tenant_id UUID NOT NULL,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Dashboards
CREATE TABLE dashboards (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  name TEXT NOT NULL,
  description TEXT,
  config JSONB NOT NULL,
  is_default BOOLEAN DEFAULT false,
  tenant_id UUID NOT NULL,
  created_by UUID NOT NULL,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Retention policies
SELECT add_retention_policy('metrics', INTERVAL '90 days');
SELECT add_retention_policy('scanner_stats', INTERVAL '1 year');
SELECT add_retention_policy('security_events', INTERVAL '2 years');
```

#### 1.2 Frontend Foundation

**Deliverables**:
- ✅ React project setup with Vite
- ✅ TypeScript configuration
- ✅ GraphQL client setup (Apollo Client)
- ✅ Authentication flow (login/logout)
- ✅ Basic layout (navbar, sidebar, main content)
- ✅ Routing setup (React Router)

**Project Structure**:
```
dashboard/
├── package.json
├── tsconfig.json
├── vite.config.ts
├── index.html
└── src/
    ├── main.tsx
    ├── App.tsx
    ├── components/
    │   ├── common/      # Reusable components
    │   ├── charts/      # Chart components
    │   ├── layout/      # Layout components
    │   └── forms/       # Form components
    ├── pages/
    │   ├── Dashboard.tsx
    │   ├── Metrics.tsx
    │   ├── Scanners.tsx
    │   ├── Security.tsx
    │   ├── Alerts.tsx
    │   ├── Settings.tsx
    │   └── Login.tsx
    ├── lib/
    │   ├── api.ts       # API client
    │   ├── auth.ts      # Auth utilities
    │   └── utils.ts     # Helper functions
    ├── hooks/           # Custom React hooks
    ├── store/           # State management
    ├── types/           # TypeScript types
    └── styles/          # Global styles
```

#### 1.3 Data Ingestion Pipeline

**Deliverables**:
- ✅ Prometheus exporter for LLM Shield metrics
- ✅ Vector configuration for log aggregation
- ✅ TimescaleDB ingestion service
- ✅ Metric buffering and batching
- ✅ Data validation and sanitization

**Metrics to Collect**:
```rust
// API Metrics
- llm_shield_api_requests_total{method, path, status}
- llm_shield_api_request_duration_seconds{method, path}
- llm_shield_api_active_requests{method}

// Scanner Metrics
- llm_shield_scanner_scans_total{scanner, result}
- llm_shield_scanner_duration_seconds{scanner}
- llm_shield_scanner_errors_total{scanner, error_type}

// Security Metrics
- llm_shield_security_events_total{event_type, severity}
- llm_shield_threats_detected_total{scanner, threat_type}
- llm_shield_false_positives_total{scanner}

// Performance Metrics
- llm_shield_cache_hits_total{cache_type}
- llm_shield_cache_misses_total{cache_type}
- llm_shield_database_query_duration_seconds{query_type}

// Business Metrics
- llm_shield_api_calls_by_tenant{tenant_id}
- llm_shield_tokens_processed_total{tenant_id}
- llm_shield_costs_estimated{tenant_id, resource_type}
```

### Phase 15.2: Core Dashboards (Weeks 4-6)

#### 2.1 Overview Dashboard

**Purpose**: High-level system health and key metrics

**Widgets**:
1. **System Health**
   - API uptime percentage (24h, 7d, 30d)
   - Request rate (current, average, peak)
   - Error rate (last hour, trend)
   - Active users (current count)

2. **Scanner Performance**
   - Top 5 most used scanners (pie chart)
   - Scanner success rate (bar chart)
   - Average scan latency (line chart)
   - Invalid requests by scanner (table)

3. **Security Overview**
   - Security events (last 24h, severity breakdown)
   - Threats detected (count, trend)
   - Top threat types (bar chart)
   - False positive rate (gauge)

4. **Resource Usage**
   - CPU usage (gauge + sparkline)
   - Memory usage (gauge + sparkline)
   - Database connections (gauge)
   - Cache hit rate (percentage)

**Implementation**:
```tsx
// Overview Dashboard Component
const OverviewDashboard: React.FC = () => {
  const { data: healthData } = useQuery(GET_SYSTEM_HEALTH, {
    pollInterval: 5000, // Refresh every 5 seconds
  });

  const { data: scannerData } = useQuery(GET_SCANNER_STATS, {
    variables: {
      timeRange: { start: '24h', end: 'now' },
    },
  });

  const { data: securityData } = useQuery(GET_SECURITY_EVENTS, {
    variables: {
      timeRange: { start: '24h', end: 'now' },
      limit: 100,
    },
  });

  return (
    <DashboardLayout>
      <Grid container spacing={3}>
        <Grid item xs={12} md={6} lg={3}>
          <MetricCard
            title="API Uptime"
            value={healthData?.uptime || 0}
            format="percentage"
            trend={healthData?.uptimeTrend}
          />
        </Grid>
        <Grid item xs={12} md={6} lg={3}>
          <MetricCard
            title="Request Rate"
            value={healthData?.requestRate || 0}
            format="number"
            suffix="req/s"
            trend={healthData?.requestRateTrend}
          />
        </Grid>
        <Grid item xs={12} md={6} lg={3}>
          <MetricCard
            title="Error Rate"
            value={healthData?.errorRate || 0}
            format="percentage"
            trend={healthData?.errorRateTrend}
            alert={healthData?.errorRate > 5}
          />
        </Grid>
        <Grid item xs={12} md={6} lg={3}>
          <MetricCard
            title="Active Users"
            value={healthData?.activeUsers || 0}
            format="number"
          />
        </Grid>

        <Grid item xs={12} lg={6}>
          <ScannerPerformanceChart data={scannerData} />
        </Grid>
        <Grid item xs={12} lg={6}>
          <SecurityEventsChart data={securityData} />
        </Grid>

        <Grid item xs={12}>
          <RecentSecurityEventsTable data={securityData} />
        </Grid>
      </Grid>
    </DashboardLayout>
  );
};
```

#### 2.2 Scanner Analytics Dashboard

**Purpose**: Detailed scanner performance and effectiveness analysis

**Sections**:
1. **Scanner Comparison**
   - Success rate comparison (all scanners)
   - Latency comparison (box plot)
   - Throughput comparison (bar chart)
   - Error rate comparison (stacked bar)

2. **Scanner Deep Dive** (per scanner)
   - Request volume (time series)
   - Success/failure breakdown (pie chart)
   - Latency distribution (histogram)
   - Error types (treemap)
   - Configuration impact analysis

3. **Scanner Effectiveness**
   - True positive rate
   - False positive rate
   - False negative rate (if feedback available)
   - Precision and recall metrics

4. **Cost Analysis**
   - Compute cost per scanner
   - Cost per successful scan
   - Model inference costs
   - Optimization recommendations

#### 2.3 Security Dashboard

**Purpose**: Security event monitoring and threat analysis

**Sections**:
1. **Threat Overview**
   - Threat severity distribution (pie chart)
   - Threats over time (area chart)
   - Top threat types (bar chart)
   - Threat sources (geographic map)

2. **Event Timeline**
   - Security events timeline (Gantt-style)
   - Event details panel (drill-down)
   - Related events clustering
   - Event correlation analysis

3. **Scanner Security Analysis**
   - Blocked requests by scanner (bar chart)
   - Bypass attempts detection
   - Evasion technique analysis
   - Scanner effectiveness rating

4. **Compliance Dashboard**
   - PII exposure incidents
   - Data leakage events
   - Compliance violations
   - Audit trail

#### 2.4 API Usage Dashboard

**Purpose**: API usage analytics and customer insights

**Sections**:
1. **Usage Metrics**
   - Total API calls (time series)
   - Unique users (time series)
   - Peak usage times (heatmap)
   - Geographic distribution (map)

2. **Endpoint Analysis**
   - Most used endpoints (bar chart)
   - Slowest endpoints (table)
   - Error-prone endpoints (table)
   - Endpoint usage patterns

3. **Tenant Analytics** (Multi-tenant)
   - Usage by tenant (bar chart)
   - Cost by tenant (pie chart)
   - Quota usage (gauge per tenant)
   - Growth trends

4. **Performance Insights**
   - Response time distribution (histogram)
   - Cache hit rate (gauge)
   - Rate limit hits (counter)
   - Throttled requests (time series)

### Phase 15.3: Real-time Features (Weeks 7-8)

#### 3.1 Real-time Metric Updates

**Technology**: WebSocket with Socket.io

**Implementation**:
```rust
// Backend WebSocket handler
use axum::extract::ws::{WebSocket, WebSocketUpgrade};

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();

    // Subscribe to metric updates
    let mut metric_rx = state.metrics_broadcaster.subscribe();

    // Send updates to client
    tokio::spawn(async move {
        while let Ok(metric) = metric_rx.recv().await {
            let msg = serde_json::to_string(&metric).unwrap();
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    // Handle client messages
    while let Some(Ok(msg)) = receiver.next().await {
        match msg {
            Message::Text(text) => {
                // Handle subscription updates
                handle_subscription(&text, &state).await;
            }
            Message::Close(_) => break,
            _ => {}
        }
    }
}
```

```tsx
// Frontend WebSocket client
const useRealtimeMetrics = (metricNames: string[]) => {
  const [metrics, setMetrics] = useState<Metric[]>([]);
  const socket = useSocket();

  useEffect(() => {
    // Subscribe to metrics
    socket.emit('subscribe', { metrics: metricNames });

    // Listen for updates
    socket.on('metric', (metric: Metric) => {
      setMetrics(prev => [...prev, metric].slice(-100)); // Keep last 100
    });

    return () => {
      socket.emit('unsubscribe', { metrics: metricNames });
      socket.off('metric');
    };
  }, [metricNames, socket]);

  return metrics;
};
```

#### 3.2 Live Security Event Stream

**Purpose**: Real-time security event monitoring

**Features**:
- Live event feed (scrolling list)
- Severity-based filtering
- Event details modal
- Quick actions (acknowledge, investigate, block)
- Event correlation indicators

#### 3.3 Live Alert Notifications

**Purpose**: Instant alert delivery to dashboard users

**Features**:
- Toast notifications for new alerts
- Alert sound (configurable)
- Alert acknowledgment
- Alert history
- Snooze functionality

### Phase 15.4: Advanced Features (Weeks 9-10)

#### 4.1 Alert Management System

**Alert Rule Engine**:
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct AlertRule {
    pub id: Uuid,
    pub name: String,
    pub query: String,  // PromQL-style query
    pub threshold: f64,
    pub operator: Operator,
    pub duration: Duration,
    pub severity: Severity,
    pub notification_channels: Vec<NotificationChannel>,
}

#[derive(Debug)]
pub enum Operator {
    GreaterThan,
    LessThan,
    EqualTo,
    NotEqualTo,
}

#[derive(Debug)]
pub enum Severity {
    Info,
    Warning,
    Error,
    Critical,
}

#[derive(Debug)]
pub enum NotificationChannel {
    Email(String),
    Slack(String),
    PagerDuty(String),
    Webhook(String),
}

// Alert evaluation engine
pub struct AlertEvaluator {
    rules: Arc<RwLock<Vec<AlertRule>>>,
    db: TimescaleDBClient,
    notifier: Notifier,
}

impl AlertEvaluator {
    pub async fn evaluate_all(&self) -> Result<()> {
        let rules = self.rules.read().await;

        for rule in rules.iter().filter(|r| r.enabled) {
            self.evaluate_rule(rule).await?;
        }

        Ok(())
    }

    async fn evaluate_rule(&self, rule: &AlertRule) -> Result<()> {
        let value = self.db.query_metric(&rule.query).await?;

        let triggered = match rule.operator {
            Operator::GreaterThan => value > rule.threshold,
            Operator::LessThan => value < rule.threshold,
            // ... other operators
        };

        if triggered {
            self.notifier.send_alert(rule).await?;
        }

        Ok(())
    }
}
```

**Alert UI**:
- Alert rule builder (visual editor)
- Alert history
- Alert analytics (most triggered, response times)
- Alert routing rules
- Escalation policies

#### 4.2 Custom Dashboards

**Features**:
- Drag-and-drop dashboard builder
- Widget library (20+ widget types)
- Custom widget creation
- Dashboard templates
- Dashboard sharing (team, organization)
- Dashboard versioning
- Dashboard export/import (JSON)

**Widget Types**:
1. **Metric Widgets**
   - Single stat (with sparkline)
   - Gauge
   - Counter
   - Progress bar

2. **Chart Widgets**
   - Line chart
   - Area chart
   - Bar chart
   - Pie/Donut chart
   - Scatter plot
   - Heatmap
   - Box plot

3. **Data Widgets**
   - Table
   - List
   - Timeline
   - Treemap

4. **Text Widgets**
   - Markdown
   - HTML
   - Alert list

#### 4.3 Report Generation

**Features**:
- Scheduled reports (daily, weekly, monthly)
- Custom report builder
- Report templates
- Multiple export formats (PDF, CSV, Excel, JSON)
- Email delivery
- S3/Cloud storage upload

**Report Types**:
1. **Executive Summary**
   - Key metrics overview
   - Trends and insights
   - Recommendations

2. **Security Report**
   - Security events summary
   - Threat analysis
   - Compliance status
   - Incident details

3. **Performance Report**
   - API performance metrics
   - Scanner performance
   - SLA compliance
   - Capacity planning

4. **Usage Report**
   - API usage by tenant
   - Cost breakdown
   - Top users/endpoints
   - Growth metrics

#### 4.4 Query Builder

**Purpose**: No-code metric query interface

**Features**:
- Visual query builder
- Query templates
- Query history
- Query sharing
- Export to CSV/JSON
- API endpoint generation

### Phase 15.5: Enterprise Features (Weeks 11-12)

#### 5.1 Multi-tenancy

**Architecture**:
- Tenant isolation (data and resources)
- Tenant-specific configurations
- Tenant usage limits
- Tenant-specific dashboards
- Cross-tenant aggregation (admin)

**Implementation**:
```rust
// Tenant context middleware
pub struct TenantContext {
    pub tenant_id: Uuid,
    pub permissions: Vec<Permission>,
}

pub async fn tenant_middleware(
    req: Request<Body>,
    next: Next<Body>,
) -> Result<Response> {
    let token = extract_token(&req)?;
    let tenant_id = validate_token_and_get_tenant(&token).await?;

    req.extensions_mut().insert(TenantContext {
        tenant_id,
        permissions: get_tenant_permissions(tenant_id).await?,
    });

    Ok(next.run(req).await)
}

// Row-level security in queries
async fn get_metrics(
    tenant_id: Uuid,
    filters: MetricFilters,
) -> Result<Vec<Metric>> {
    sqlx::query_as!(
        Metric,
        r#"
        SELECT * FROM metrics
        WHERE tenant_id = $1
        AND time >= $2
        AND time <= $3
        "#,
        tenant_id,
        filters.start_time,
        filters.end_time
    )
    .fetch_all(&db)
    .await
}
```

#### 5.2 Role-Based Access Control (RBAC)

**Roles**:
1. **Super Admin**
   - Full system access
   - Cross-tenant access
   - System configuration

2. **Tenant Admin**
   - Tenant management
   - User management
   - Dashboard management
   - Alert management

3. **Developer**
   - View dashboards
   - Create personal dashboards
   - Query metrics
   - No administrative access

4. **Viewer**
   - View dashboards only
   - No configuration access

**Permissions**:
```rust
#[derive(Debug, Serialize, Deserialize)]
pub enum Permission {
    // Dashboard permissions
    DashboardView,
    DashboardCreate,
    DashboardEdit,
    DashboardDelete,

    // Alert permissions
    AlertView,
    AlertCreate,
    AlertEdit,
    AlertDelete,

    // User permissions
    UserView,
    UserCreate,
    UserEdit,
    UserDelete,

    // System permissions
    SystemConfig,
    TenantManage,
}

// Permission checking
pub fn check_permission(
    user: &User,
    permission: Permission,
) -> Result<()> {
    if user.permissions.contains(&permission) {
        Ok(())
    } else {
        Err(Error::Unauthorized)
    }
}
```

#### 5.3 SSO Integration

**Supported Protocols**:
- OAuth 2.0
- OpenID Connect (OIDC)
- SAML 2.0

**Providers**:
- Auth0
- Okta
- Azure AD
- Google Workspace
- Generic OIDC

**Implementation**:
```rust
// OAuth2 configuration
pub struct OAuth2Config {
    pub client_id: String,
    pub client_secret: String,
    pub auth_url: String,
    pub token_url: String,
    pub redirect_uri: String,
}

// SSO login handler
pub async fn sso_login(
    Query(params): Query<SSOParams>,
    State(state): State<AppState>,
) -> Result<Redirect> {
    let auth_url = state.oauth2_client
        .authorize_url(|| CsrfToken::new(params.state))
        .add_scope(Scope::new("openid".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .add_scope(Scope::new("email".to_string()))
        .url();

    Ok(Redirect::to(auth_url.0.as_str()))
}

// OAuth2 callback handler
pub async fn sso_callback(
    Query(params): Query<CallbackParams>,
    State(state): State<AppState>,
) -> Result<Json<TokenResponse>> {
    let token = state.oauth2_client
        .exchange_code(AuthorizationCode::new(params.code))
        .await?;

    let user_info = fetch_user_info(&token).await?;
    let jwt = create_jwt_token(&user_info, &state.config).await?;

    Ok(Json(TokenResponse { token: jwt }))
}
```

#### 5.4 Audit Logging

**Purpose**: Track all user actions for compliance

**Logged Actions**:
- User authentication (login/logout)
- Dashboard access
- Configuration changes
- Alert rule modifications
- User management actions
- Data exports
- API key generation

**Schema**:
```sql
CREATE TABLE audit_log (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  user_id UUID NOT NULL,
  tenant_id UUID NOT NULL,
  action TEXT NOT NULL,
  resource_type TEXT NOT NULL,
  resource_id TEXT,
  changes JSONB,
  ip_address INET,
  user_agent TEXT,
  result TEXT NOT NULL CHECK (result IN ('success', 'failure')),
  error_message TEXT
);

CREATE INDEX idx_audit_log_timestamp ON audit_log(timestamp DESC);
CREATE INDEX idx_audit_log_user ON audit_log(user_id, timestamp DESC);
CREATE INDEX idx_audit_log_tenant ON audit_log(tenant_id, timestamp DESC);
```

**Implementation**:
```rust
pub async fn log_audit_event(
    db: &PgPool,
    event: AuditEvent,
) -> Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO audit_log (
            user_id, tenant_id, action, resource_type,
            resource_id, changes, ip_address, user_agent, result
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "#,
        event.user_id,
        event.tenant_id,
        event.action,
        event.resource_type,
        event.resource_id,
        event.changes,
        event.ip_address,
        event.user_agent,
        event.result.to_string()
    )
    .execute(db)
    .await?;

    Ok(())
}
```

---

## Security Architecture

### 1. Authentication

**Multi-factor Authentication**:
- TOTP (Time-based One-Time Password)
- SMS (optional)
- Backup codes

**Session Management**:
- JWT tokens (short-lived, 15 minutes)
- Refresh tokens (long-lived, 7 days, httpOnly cookies)
- Token rotation on refresh
- Secure session storage (Redis)

### 2. Authorization

**Implementation**: Casbin (RBAC + ABAC)

**Policy Model**:
```ini
[request_definition]
r = sub, obj, act

[policy_definition]
p = sub, obj, act

[role_definition]
g = _, _

[policy_effect]
e = some(where (p.eft == allow))

[matchers]
m = g(r.sub, p.sub) && r.obj == p.obj && r.act == p.act
```

**Example Policies**:
```csv
p, role:admin, dashboard:*, write
p, role:developer, dashboard:*, read
p, role:developer, dashboard:owned, write
p, role:viewer, dashboard:*, read

g, user:alice, role:admin
g, user:bob, role:developer
```

### 3. Data Protection

**Encryption at Rest**:
- Database: PostgreSQL encryption (pgcrypto)
- Sensitive fields: Column-level encryption
- Backups: Encrypted with KMS

**Encryption in Transit**:
- TLS 1.3 for all connections
- Certificate pinning for API
- HSTS headers

**Data Minimization**:
- PII redaction in logs
- Automatic data anonymization
- Configurable retention policies

### 4. Input Validation

**All inputs validated**:
- GraphQL schema validation
- SQL injection prevention (parameterized queries)
- XSS prevention (Content Security Policy)
- CSRF protection (tokens)

### 5. Rate Limiting

**Tiered rate limiting**:
- Anonymous: 10 requests/minute
- Authenticated: 100 requests/minute
- Premium: 1000 requests/minute

**Implementation**:
```rust
use tower_governor::{Governor, GovernorConfigBuilder};

let governor_conf = Box::new(
    GovernorConfigBuilder::default()
        .per_second(10)
        .burst_size(20)
        .finish()
        .unwrap(),
);

let app = Router::new()
    .route("/graphql", post(graphql_handler))
    .layer(governor_conf);
```

### 6. Security Monitoring

**Real-time monitoring**:
- Failed login attempts
- Suspicious query patterns
- Unusual data access
- API abuse detection
- DDoS mitigation

---

## Cost Analysis

### Infrastructure Costs (Monthly)

#### Self-Hosted (AWS)

| Component | Resource | Cost |
|-----------|---------|------|
| **Compute** | 2x t3.large (dashboard API) | $120 |
| **Database** | RDS PostgreSQL db.t3.large | $140 |
| **Cache** | ElastiCache Redis m5.large | $90 |
| **Load Balancer** | ALB | $20 |
| **Data Transfer** | 500 GB/month | $45 |
| **Backup/Storage** | S3 + Snapshots | $15 |
| **Monitoring** | CloudWatch | $10 |
| **Total** | | **$440/month** |

#### Cloud-Hosted (Managed Services)

| Component | Service | Cost |
|-----------|---------|------|
| **Compute** | AWS Fargate (2 tasks) | $150 |
| **Database** | TimescaleDB Cloud (4GB) | $200 |
| **Cache** | Upstash Redis | $40 |
| **Storage** | S3 | $10 |
| **CDN** | CloudFront | $15 |
| **Total** | | **$415/month** |

### Development Costs

| Phase | Duration | Engineering Cost (2 developers @ $150/hr) |
|-------|----------|------------------------------------------|
| Phase 15.1 | 3 weeks | $36,000 |
| Phase 15.2 | 3 weeks | $36,000 |
| Phase 15.3 | 2 weeks | $24,000 |
| Phase 15.4 | 2 weeks | $24,000 |
| Phase 15.5 | 2 weeks | $24,000 |
| **Total** | **12 weeks** | **$144,000** |

### Ongoing Costs

- **Maintenance**: 20 hours/month = $3,000/month
- **Support**: 10 hours/month = $1,500/month
- **Infrastructure**: $440/month

**Total Monthly**: ~$5,000/month

### Revenue Potential

**Pricing Tiers** (per tenant/month):

| Tier | Users | Dashboards | Retention | Price |
|------|-------|-----------|-----------|-------|
| **Starter** | 5 | 5 | 30 days | $99 |
| **Professional** | 20 | Unlimited | 90 days | $299 |
| **Enterprise** | Unlimited | Unlimited | 1 year | $999 |

**Break-even Analysis**:
- Monthly costs: $5,000
- Break-even: 5 Enterprise customers or 17 Starter customers
- Target: 50 customers (mix of tiers)
- Projected revenue: $20,000/month
- Projected profit: $15,000/month

---

## Timeline

### Phase 15.1: Core Infrastructure (Weeks 1-3)
- Week 1: Backend API foundation, database schema
- Week 2: Frontend setup, authentication
- Week 3: Data ingestion pipeline

### Phase 15.2: Core Dashboards (Weeks 4-6)
- Week 4: Overview dashboard
- Week 5: Scanner analytics dashboard
- Week 6: Security & API usage dashboards

### Phase 15.3: Real-time Features (Weeks 7-8)
- Week 7: WebSocket implementation, real-time metrics
- Week 8: Live security events, live alerts

### Phase 15.4: Advanced Features (Weeks 9-10)
- Week 9: Alert management, custom dashboards
- Week 10: Report generation, query builder

### Phase 15.5: Enterprise Features (Weeks 11-12)
- Week 11: Multi-tenancy, RBAC
- Week 12: SSO integration, audit logging, final testing

### Post-Launch (Weeks 13-16)
- Week 13: Beta testing with select customers
- Week 14: Bug fixes and refinements
- Week 15: Documentation and training materials
- Week 16: General availability launch

---

## Success Criteria

### Technical Metrics

- ✅ Dashboard load time < 2 seconds
- ✅ Real-time metric lag < 5 seconds
- ✅ Query performance < 1 second (95th percentile)
- ✅ 99.9% uptime
- ✅ Support 10,000+ concurrent users
- ✅ Handle 1M+ metrics per minute
- ✅ < 5% error rate under load

### User Metrics

- ✅ 10+ beta customers signed up
- ✅ 80%+ user satisfaction score
- ✅ 50%+ daily active users
- ✅ Average session duration > 5 minutes
- ✅ < 10% churn rate

### Business Metrics

- ✅ Break-even within 6 months
- ✅ 50+ paying customers within 1 year
- ✅ $20,000+ MRR within 1 year
- ✅ < $5,000 monthly infrastructure costs

---

## Risk Assessment

### Technical Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| **Performance issues at scale** | Medium | High | Load testing, caching, database optimization |
| **Real-time latency** | Medium | Medium | WebSocket optimization, CDN, edge caching |
| **Database bottlenecks** | Low | High | TimescaleDB continuous aggregates, read replicas |
| **Security vulnerabilities** | Low | Critical | Security audits, penetration testing, bug bounty |

### Business Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| **Low customer adoption** | Medium | High | Early beta program, customer development |
| **Competition** | High | Medium | Focus on LLM-specific features, fast iteration |
| **High infrastructure costs** | Low | Medium | Cost monitoring, auto-scaling, reserved instances |
| **Customer churn** | Medium | High | Excellent support, continuous improvement |

### Operational Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| **Key person dependency** | Low | High | Documentation, knowledge sharing, pair programming |
| **Scope creep** | High | Medium | Strict phase boundaries, MVP focus |
| **Technical debt** | Medium | Medium | Code reviews, refactoring sprints, quality gates |

---

## Next Steps

### Immediate Actions (Week 1)

1. **Architecture Review**
   - Review this plan with engineering team
   - Validate technology choices
   - Identify risks and dependencies

2. **Team Formation**
   - Assign 2 backend engineers (Rust)
   - Assign 1 frontend engineer (React)
   - Assign 1 DevOps engineer (part-time)

3. **Environment Setup**
   - Provision development infrastructure
   - Setup CI/CD pipelines
   - Configure monitoring and logging

4. **Customer Research**
   - Interview 5-10 potential customers
   - Validate dashboard requirements
   - Prioritize features

### Month 1 Goals

- Complete Phase 15.1 (Core Infrastructure)
- Deploy to staging environment
- Begin Phase 15.2 (Core Dashboards)
- Recruit beta testers

### Month 2-3 Goals

- Complete Phases 15.2-15.4
- Beta launch with 5-10 customers
- Gather feedback and iterate
- Complete Phase 15.5 (Enterprise Features)

### Month 4+ Goals

- General availability launch
- Customer acquisition campaign
- Break-even achievement
- Feature expansion based on feedback

---

## Appendix

### A. Technology Alternatives Considered

| Component | Chosen | Alternatives Considered | Reason for Choice |
|-----------|--------|------------------------|-------------------|
| **Time-Series DB** | TimescaleDB | InfluxDB, Prometheus | SQL familiarity, PostgreSQL ecosystem |
| **Frontend** | React | Vue, Svelte, Angular | Ecosystem, talent availability, maturity |
| **Charts** | Apache ECharts | Recharts, Chart.js, Plotly | Feature richness, performance |
| **Backend** | Axum | Actix, Rocket, Warp | Performance, ecosystem, type safety |
| **API** | GraphQL | REST, gRPC | Flexibility, type safety, developer experience |
| **Cache** | Redis | Memcached, DragonflyDB | Feature richness, ecosystem |

### B. Reference Architecture

See [Architecture Diagrams](./diagrams/) folder for detailed architecture diagrams (to be created).

### C. Database Schema

See [Database Schema](./schema/) folder for complete DDL scripts (to be created).

### D. API Documentation

See [API Docs](./api-docs/) folder for OpenAPI/GraphQL schema documentation (to be created).

---

## Conclusion

Phase 15 will transform LLM Shield into a comprehensive, enterprise-grade security platform with world-class observability. The dashboard and monitoring solution will provide:

- **Complete Visibility**: Real-time and historical insights into system performance
- **Proactive Monitoring**: Intelligent alerting and anomaly detection
- **Security Assurance**: Comprehensive security event tracking and threat analysis
- **Business Intelligence**: Usage analytics, cost management, and growth insights
- **Enterprise Ready**: Multi-tenancy, RBAC, SSO, and audit logging

With a clear implementation plan, proven technology stack, and focus on customer needs, Phase 15 will position LLM Shield as a market-leading solution in the LLM security space.

**Status**: Ready for Implementation
**Confidence**: High
**Recommendation**: Proceed with Phase 15.1 immediately

---

**Document Status**: Draft v1.0
**Next Review**: Before Phase 15.1 kickoff
**Maintained By**: LLM Shield Team
**Contact**: engineering@llmshield.dev
