# Multi-Cloud Deployment Examples

This directory contains production-ready deployment configurations and scripts for deploying LLM Shield API to AWS, GCP, and Azure.

## Quick Start

### AWS Deployment

```bash
# Set environment variables
export AWS_REGION=us-east-1
export IMAGE_TAG=v1.0.0

# Run deployment script
chmod +x deploy-aws.sh
./deploy-aws.sh
```

### GCP Deployment

```bash
# Set environment variables
export GCP_PROJECT=llm-shield-prod
export GCP_REGION=us-central1
export DEPLOY_TARGET=cloud-run  # or 'gke'
export IMAGE_TAG=v1.0.0

# Run deployment script
chmod +x deploy-gcp.sh
./deploy-gcp.sh
```

### Azure Deployment

```bash
# Set environment variables
export AZURE_RESOURCE_GROUP=llm-shield-rg
export AZURE_LOCATION=eastus
export ACR_NAME=llmshieldacr
export DEPLOY_TARGET=container-apps  # or 'aks'
export IMAGE_TAG=v1.0.0

# Run deployment script
chmod +x deploy-azure.sh
./deploy-azure.sh
```

## Architecture

### AWS Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Internet Gateway                      │
└─────────────────────────────────────────────────────────┘
                            │
┌─────────────────────────────────────────────────────────┐
│              Application Load Balancer                   │
└─────────────────────────────────────────────────────────┘
                            │
┌─────────────────────────────────────────────────────────┐
│                   ECS Fargate Service                    │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐              │
│  │ Task 1   │  │ Task 2   │  │ Task 3   │              │
│  │ (2 vCPU) │  │ (2 vCPU) │  │ (2 vCPU) │              │
│  └──────────┘  └──────────┘  └──────────┘              │
└─────────────────────────────────────────────────────────┘
                            │
         ┌──────────────────┼──────────────────┐
         │                  │                  │
┌────────▼────────┐ ┌───────▼──────┐ ┌────────▼────────┐
│ Secrets Manager │ │   S3 Bucket  │ │   CloudWatch    │
│  - JWT Secret   │ │  - ML Models │ │  - Metrics/Logs │
└─────────────────┘ └──────────────┘ └─────────────────┘
```

**Components:**
- **ECS Fargate**: Serverless container orchestration
- **Application Load Balancer**: Traffic distribution with SSL termination
- **Secrets Manager**: Secure API keys and JWT secrets
- **S3**: ML model storage
- **CloudWatch**: Metrics and centralized logging
- **IAM Roles**: Least-privilege access control

**Cost Estimate**: ~$150-300/month (3 tasks, moderate traffic)

### GCP Architecture (Cloud Run)

```
┌─────────────────────────────────────────────────────────┐
│                  Cloud Load Balancer                     │
└─────────────────────────────────────────────────────────┘
                            │
┌─────────────────────────────────────────────────────────┐
│                    Cloud Run Service                     │
│  Auto-scales 1-10 instances based on traffic            │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐              │
│  │Instance 1│  │Instance 2│  │Instance 3│              │
│  │ (2 vCPU) │  │ (2 vCPU) │  │ (2 vCPU) │              │
│  └──────────┘  └──────────┘  └──────────┘              │
└─────────────────────────────────────────────────────────┘
                            │
         ┌──────────────────┼──────────────────┐
         │                  │                  │
┌────────▼────────┐ ┌───────▼──────┐ ┌────────▼────────┐
│ Secret Manager  │ │Cloud Storage │ │Cloud Monitoring │
│  - JWT Secret   │ │  - ML Models │ │  - Metrics/Logs │
└─────────────────┘ └──────────────┘ └─────────────────┘
```

**Components:**
- **Cloud Run**: Fully managed serverless platform
- **Cloud Load Balancer**: Global load balancing with SSL
- **Secret Manager**: Managed secret storage
- **Cloud Storage**: Object storage for models
- **Cloud Monitoring**: Unified observability
- **Workload Identity**: Secure service-to-service authentication

**Cost Estimate**: ~$100-200/month (pay-per-use, scales to zero)

### GCP Architecture (GKE)

```
┌─────────────────────────────────────────────────────────┐
│                  Cloud Load Balancer                     │
└─────────────────────────────────────────────────────────┘
                            │
┌─────────────────────────────────────────────────────────┐
│              GKE Cluster (3-10 nodes)                    │
│  ┌────────────────────────────────────────────┐         │
│  │           llm-shield-api Deployment        │         │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐ │         │
│  │  │  Pod 1   │  │  Pod 2   │  │  Pod 3   │ │         │
│  │  └──────────┘  └──────────┘  └──────────┘ │         │
│  │  HPA: 3-10 replicas based on CPU/memory   │         │
│  └────────────────────────────────────────────┘         │
└─────────────────────────────────────────────────────────┘
                            │
         ┌──────────────────┼──────────────────┐
         │                  │                  │
┌────────▼────────┐ ┌───────▼──────┐ ┌────────▼────────┐
│ Secret Manager  │ │Cloud Storage │ │Cloud Monitoring │
│  - JWT Secret   │ │  - ML Models │ │  - Metrics/Logs │
└─────────────────┘ └──────────────┘ └─────────────────┘
```

**Cost Estimate**: ~$200-400/month (3-node cluster)

### Azure Architecture (Container Apps)

```
┌─────────────────────────────────────────────────────────┐
│              Azure Application Gateway                   │
└─────────────────────────────────────────────────────────┘
                            │
┌─────────────────────────────────────────────────────────┐
│              Azure Container Apps Service                │
│  Auto-scales 1-10 instances based on HTTP traffic       │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐              │
│  │Instance 1│  │Instance 2│  │Instance 3│              │
│  │ (2 vCPU) │  │ (2 vCPU) │  │ (2 vCPU) │              │
│  └──────────┘  └──────────┘  └──────────┘              │
└─────────────────────────────────────────────────────────┘
                            │
         ┌──────────────────┼──────────────────┐
         │                  │                  │
┌────────▼────────┐ ┌───────▼──────┐ ┌────────▼────────┐
│   Key Vault     │ │ Blob Storage │ │ Azure Monitor   │
│  - JWT Secret   │ │  - ML Models │ │  - Metrics/Logs │
└─────────────────┘ └──────────────┘ └─────────────────┘
```

**Components:**
- **Azure Container Apps**: Serverless Kubernetes-based platform
- **Application Gateway**: Application-level load balancing
- **Key Vault**: Managed secret and key storage
- **Blob Storage**: Object storage for models
- **Azure Monitor**: Comprehensive monitoring solution
- **Managed Identity**: Azure AD-based authentication

**Cost Estimate**: ~$120-250/month (1-10 instances, moderate traffic)

### Azure Architecture (AKS)

```
┌─────────────────────────────────────────────────────────┐
│             Azure Load Balancer                          │
└─────────────────────────────────────────────────────────┘
                            │
┌─────────────────────────────────────────────────────────┐
│              AKS Cluster (3-10 nodes)                    │
│  ┌────────────────────────────────────────────┐         │
│  │           llm-shield-api Deployment        │         │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐ │         │
│  │  │  Pod 1   │  │  Pod 2   │  │  Pod 3   │ │         │
│  │  └──────────┘  └──────────┘  └──────────┘ │         │
│  │  HPA: 3-10 replicas based on CPU/memory   │         │
│  └────────────────────────────────────────────┘         │
└─────────────────────────────────────────────────────────┘
                            │
         ┌──────────────────┼──────────────────┐
         │                  │                  │
┌────────▼────────┐ ┌───────▼──────┐ ┌────────▼────────┐
│   Key Vault     │ │ Blob Storage │ │ Azure Monitor   │
│  - JWT Secret   │ │  - ML Models │ │  - Metrics/Logs │
└─────────────────┘ └──────────────┘ └─────────────────┘
```

**Cost Estimate**: ~$250-500/month (3-node cluster)

## Configuration

### Environment Variables

All deployments support configuration via environment variables:

**Common:**
```bash
RUST_LOG=info                    # Logging level
LLM_SHIELD_API__SERVER__PORT=8080
LLM_SHIELD_API__SERVER__HOST=0.0.0.0
```

**AWS:**
```bash
AWS_REGION=us-east-1
AWS_ACCESS_KEY_ID=...           # For local testing only
AWS_SECRET_ACCESS_KEY=...       # For local testing only
```

**GCP:**
```bash
GCP_PROJECT=llm-shield-prod
GOOGLE_APPLICATION_CREDENTIALS=/path/to/service-account.json  # For local testing
```

**Azure:**
```bash
AZURE_TENANT_ID=...             # For service principal auth
AZURE_CLIENT_ID=...
AZURE_CLIENT_SECRET=...
```

### Configuration Files

Each cloud provider has a dedicated TOML configuration file:

- `config-aws.toml` - AWS configuration
- `config-gcp.toml` - GCP configuration
- `config-azure.toml` - Azure configuration

Configuration precedence (highest to lowest):
1. Environment variables (prefixed with `LLM_SHIELD_API__`)
2. TOML configuration file
3. Built-in defaults

## Prerequisites

### AWS Prerequisites

```bash
# Install AWS CLI
curl "https://awscli.amazonaws.com/awscli-exe-linux-x86_64.zip" -o "awscliv2.zip"
unzip awscliv2.zip
sudo ./aws/install

# Configure credentials
aws configure

# Create ECR repository
aws ecr create-repository --repository-name llm-shield-api --region us-east-1

# Create ECS cluster
aws ecs create-cluster --cluster-name llm-shield-cluster --region us-east-1

# Create IAM role (see AWS documentation)
# - ecsTaskExecutionRole (for ECS)
# - LLMShieldAPIRole (for application permissions)
```

### GCP Prerequisites

```bash
# Install gcloud CLI
curl https://sdk.cloud.google.com | bash
exec -l $SHELL

# Initialize and authenticate
gcloud init
gcloud auth login

# Set project
gcloud config set project llm-shield-prod

# Enable required APIs
gcloud services enable \
  run.googleapis.com \
  container.googleapis.com \
  secretmanager.googleapis.com \
  storage.googleapis.com \
  monitoring.googleapis.com \
  logging.googleapis.com

# Create service account
gcloud iam service-accounts create llm-shield-api \
  --display-name="LLM Shield API Service Account"

# Grant roles
gcloud projects add-iam-policy-binding llm-shield-prod \
  --member="serviceAccount:llm-shield-api@llm-shield-prod.iam.gserviceaccount.com" \
  --role="roles/secretmanager.secretAccessor"

gcloud projects add-iam-policy-binding llm-shield-prod \
  --member="serviceAccount:llm-shield-api@llm-shield-prod.iam.gserviceaccount.com" \
  --role="roles/storage.objectViewer"

gcloud projects add-iam-policy-binding llm-shield-prod \
  --member="serviceAccount:llm-shield-api@llm-shield-prod.iam.gserviceaccount.com" \
  --role="roles/monitoring.metricWriter"
```

### Azure Prerequisites

```bash
# Install Azure CLI
curl -sL https://aka.ms/InstallAzureCLIDeb | sudo bash

# Login
az login

# Create resource group
az group create \
  --name llm-shield-rg \
  --location eastus

# Create container registry
az acr create \
  --resource-group llm-shield-rg \
  --name llmshieldacr \
  --sku Standard

# Create custom RBAC role
az role definition create \
  --role-definition @../../crates/llm-shield-cloud-azure/rbac-roles/llm-shield-full-role.json

# For Container Apps: Create environment
az containerapp env create \
  --name llm-shield-env \
  --resource-group llm-shield-rg \
  --location eastus

# For AKS: Create cluster
az aks create \
  --resource-group llm-shield-rg \
  --name llm-shield-aks \
  --node-count 3 \
  --node-vm-size Standard_D4s_v3 \
  --enable-managed-identity \
  --generate-ssh-keys
```

## Security Best Practices

### 1. Secrets Management

**Never hardcode secrets in configuration files or environment variables.**

✅ **Good:**
```toml
jwt_secret = "${AWS_SECRET:llm-shield/jwt-secret}"
```

❌ **Bad:**
```toml
jwt_secret = "my-secret-key-123"
```

### 2. Network Security

- Use private networking where possible (VPC, VNET)
- Enable TLS/SSL for all external endpoints
- Restrict ingress to known IP ranges
- Use security groups/firewall rules

### 3. Identity and Access Management

**AWS:**
- Use IAM roles (not access keys)
- Enable MFA for root account
- Apply least-privilege permissions

**GCP:**
- Use Workload Identity (not service account keys)
- Enable Cloud IAM Recommender
- Use organization policies

**Azure:**
- Use Managed Identity (not service principals with secrets)
- Enable Azure AD PIM
- Apply Azure Policy

### 4. Monitoring and Auditing

- Enable CloudTrail/Cloud Audit Logs/Activity Log
- Set up alerting for anomalous behavior
- Review access logs regularly
- Enable GuardDuty/Security Command Center/Defender

## Scaling Configuration

### Horizontal Pod Autoscaler (HPA)

All Kubernetes deployments (GKE, AKS) include HPA configuration:

```yaml
minReplicas: 3
maxReplicas: 10
metrics:
  - type: Resource
    resource:
      name: cpu
      target:
        type: Utilization
        averageUtilization: 70
  - type: Resource
    resource:
      name: memory
      target:
        type: Utilization
        averageUtilization: 80
```

### Serverless Autoscaling

**Cloud Run:**
```bash
--min-instances 1 --max-instances 10
```

**Azure Container Apps:**
```bash
--min-replicas 1 --max-replicas 10
```

## Monitoring

### Health Checks

All deployments expose health check endpoints:

```bash
# Liveness probe
curl http://localhost:8080/health

# Readiness probe (same endpoint)
curl http://localhost:8080/health
```

### Metrics

Prometheus metrics are exposed on port 9090:

```bash
curl http://localhost:9090/metrics
```

Key metrics:
- `http_requests_total` - Total HTTP requests
- `http_request_duration_seconds` - Request latency
- `scan_duration_seconds` - Scanner execution time
- `cache_hits_total` - Cache hit rate

### Logs

Structured JSON logs are sent to:
- AWS: CloudWatch Logs
- GCP: Cloud Logging
- Azure: Log Analytics

Query examples:

**AWS CloudWatch Insights:**
```
fields @timestamp, level, message
| filter level = "ERROR"
| sort @timestamp desc
```

**GCP:**
```
resource.type="cloud_run_revision"
severity="ERROR"
```

**Azure (KQL):**
```kusto
LLMShieldAPI_CL
| where Level == "ERROR"
| order by TimeGenerated desc
```

## Troubleshooting

### Common Issues

**1. Authentication Errors**

```bash
# AWS
aws sts get-caller-identity

# GCP
gcloud auth list
gcloud auth application-default print-access-token

# Azure
az account show
```

**2. Image Pull Errors**

```bash
# AWS
aws ecr get-login-password --region us-east-1

# GCP
gcloud auth configure-docker

# Azure
az acr login --name llmshieldacr
```

**3. Permission Errors**

Check IAM roles/RBAC assignments for the compute service account.

## Cost Optimization

1. **Use spot/preemptible instances** for non-production workloads
2. **Enable autoscaling** to scale down during low traffic
3. **Use reserved/committed use** discounts for production
4. **Optimize container images** (multi-stage builds, alpine base)
5. **Enable compression** for API responses
6. **Use caching** aggressively
7. **Monitor and rightsize** resource requests/limits

## Next Steps

- [ ] Set up CI/CD pipelines (GitHub Actions, Cloud Build, Azure Pipelines)
- [ ] Configure custom domains and SSL certificates
- [ ] Set up monitoring dashboards
- [ ] Implement backup and disaster recovery
- [ ] Configure WAF rules
- [ ] Set up staging environments
- [ ] Performance testing and optimization
- [ ] Security scanning and compliance

## Support

For issues and questions:
- GitHub Issues: https://github.com/llm-shield/llm-shield-rs/issues
- Documentation: https://docs.llmshield.dev
- Email: support@llmshield.dev
