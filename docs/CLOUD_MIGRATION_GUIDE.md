# Cloud Migration Guide

This guide covers migrating LLM Shield deployments between cloud providers, upgrading from local to cloud deployments, and best practices for zero-downtime migrations.

## Table of Contents

- [Overview](#overview)
- [Migration Scenarios](#migration-scenarios)
- [Pre-Migration Checklist](#pre-migration-checklist)
- [Migration Strategies](#migration-strategies)
- [Provider-Specific Migrations](#provider-specific-migrations)
- [Data Migration](#data-migration)
- [Rollback Procedures](#rollback-procedures)
- [Post-Migration Validation](#post-migration-validation)

## Overview

LLM Shield's cloud abstraction layer enables seamless migration between cloud providers with minimal code changes. The primary challenges involve:

1. **Data Migration**: Secrets, models, and historical metrics/logs
2. **Configuration Changes**: Provider-specific settings
3. **Authentication**: Credential setup
4. **Testing**: Validation before cutover
5. **Monitoring**: Ensuring equivalent observability

## Migration Scenarios

### Scenario 1: Local to Cloud

**Use Case**: Moving from local development to cloud production

**Complexity**: Low
**Estimated Time**: 2-4 hours
**Risk Level**: Low

### Scenario 2: Between Cloud Providers

**Use Cases**:
- Cost optimization
- Regional requirements
- Feature requirements
- Vendor lock-in avoidance

**Complexity**: Medium
**Estimated Time**: 1-2 days
**Risk Level**: Medium

### Scenario 3: Multi-Cloud Deployment

**Use Case**: Active-active or disaster recovery setup

**Complexity**: High
**Estimated Time**: 1-2 weeks
**Risk Level**: High

## Pre-Migration Checklist

### 1. Inventory Current Resources

```bash
# List all secrets
aws secretsmanager list-secrets --region us-east-1 > secrets-inventory.json

# List all storage objects
aws s3 ls s3://llm-shield-models --recursive > storage-inventory.txt

# Export current metrics configuration
aws cloudwatch describe-alarms > alarms-config.json

# Export log groups
aws logs describe-log-groups > log-groups.json
```

### 2. Assess Dependencies

- [ ] Identify all services using LLM Shield API
- [ ] Document current traffic patterns
- [ ] List all integrations (CI/CD, monitoring, etc.)
- [ ] Identify data retention requirements
- [ ] Review compliance requirements (GDPR, HIPAA, etc.)

### 3. Plan Downtime Window

**Zero-Downtime Migration (Recommended)**:
- Blue-green deployment
- 4-8 hour migration window
- Rollback capability

**Maintenance Window**:
- 2-4 hour window
- Lower risk
- Simpler rollback

### 4. Backup Current State

```bash
# Backup all secrets
./scripts/backup-secrets.sh aws us-east-1 backup/secrets/

# Backup all models
aws s3 sync s3://llm-shield-models backup/models/

# Export metrics (if possible)
# Note: Most cloud providers don't support metric export
# Consider using Prometheus for historical metrics

# Export logs
aws logs tail /aws/llm-shield/api --since 30d > backup/logs/api.log
```

## Migration Strategies

### Strategy 1: Blue-Green Deployment (Recommended)

**Advantages**:
- Zero downtime
- Easy rollback
- Full validation before cutover

**Disadvantages**:
- Higher cost during migration
- More complex setup

**Steps**:

1. **Deploy Green Environment** (new cloud provider)
   ```bash
   # Deploy to new provider
   export DEPLOY_ENV=green
   ./examples/cloud/deploy-gcp.sh  # Example: migrating to GCP
   ```

2. **Replicate Data**
   ```bash
   # Copy secrets
   ./scripts/migrate-secrets.sh aws gcp

   # Copy models
   ./scripts/migrate-storage.sh aws gcp

   # Configure logging/metrics
   ./scripts/setup-observability.sh gcp
   ```

3. **Run Parallel Traffic** (10%)
   ```yaml
   # Update load balancer to split traffic
   apiVersion: networking.k8s.io/v1
   kind: Ingress
   metadata:
     annotations:
       nginx.ingress.kubernetes.io/canary: "true"
       nginx.ingress.kubernetes.io/canary-weight: "10"
   ```

4. **Validate Green Environment**
   - Check logs for errors
   - Compare metrics (latency, error rate)
   - Run smoke tests
   - Monitor for 24-48 hours

5. **Gradual Traffic Shift**
   - 10% → 25% → 50% → 75% → 100%
   - Monitor at each step
   - Rollback if issues detected

6. **Decommission Blue Environment**
   - Keep for 7-14 days for rollback
   - Delete after validation period

### Strategy 2: Direct Cutover

**Advantages**:
- Simple
- Lower cost
- Faster migration

**Disadvantages**:
- Requires downtime
- Riskier rollback

**Steps**:

1. **Schedule Maintenance Window**
2. **Stop Blue Environment**
3. **Migrate Data**
4. **Deploy Green Environment**
5. **Update DNS/Load Balancer**
6. **Validate and Monitor**

### Strategy 3: Dual-Write Pattern

**Advantages**:
- No downtime
- Extended validation period
- Safest approach

**Disadvantages**:
- Most complex
- Highest cost
- Requires code changes

**Steps**:

1. **Modify Application** to write to both providers
   ```rust
   // Write to both AWS and GCP
   tokio::try_join!(
       aws_storage.put_object(key, data),
       gcp_storage.put_object(key, data)
   )?;
   ```

2. **Backfill Historical Data**
3. **Switch Reads** to new provider
4. **Remove Dual-Write Logic**

## Provider-Specific Migrations

### Migrating from AWS to GCP

#### 1. Secret Migration

```bash
#!/bin/bash
# migrate-secrets-aws-to-gcp.sh

set -e

AWS_REGION="us-east-1"
GCP_PROJECT="llm-shield-prod"

echo "Migrating secrets from AWS to GCP..."

# Get list of secrets
SECRETS=$(aws secretsmanager list-secrets \
  --region $AWS_REGION \
  --query 'SecretList[*].Name' \
  --output text)

for SECRET_NAME in $SECRETS; do
  echo "Migrating: $SECRET_NAME"

  # Get secret value from AWS
  SECRET_VALUE=$(aws secretsmanager get-secret-value \
    --region $AWS_REGION \
    --secret-id $SECRET_NAME \
    --query 'SecretString' \
    --output text)

  # Create secret in GCP
  echo -n "$SECRET_VALUE" | gcloud secrets create $SECRET_NAME \
    --project $GCP_PROJECT \
    --data-file=- \
    --replication-policy="automatic" || echo "Secret $SECRET_NAME already exists, updating..."

  # Update existing secret
  echo -n "$SECRET_VALUE" | gcloud secrets versions add $SECRET_NAME \
    --project $GCP_PROJECT \
    --data-file=-

  echo "✓ Migrated: $SECRET_NAME"
done

echo "✅ All secrets migrated successfully!"
```

#### 2. Storage Migration

```bash
#!/bin/bash
# migrate-storage-aws-to-gcp.sh

set -e

AWS_BUCKET="llm-shield-models"
AWS_REGION="us-east-1"
GCP_BUCKET="llm-shield-models"
GCP_PROJECT="llm-shield-prod"

echo "Migrating storage from AWS S3 to GCP Cloud Storage..."

# Install gsutil if not already installed
which gsutil || curl https://sdk.cloud.google.com | bash

# Create GCP bucket if it doesn't exist
gsutil mb -p $GCP_PROJECT gs://$GCP_BUCKET || echo "Bucket already exists"

# Copy all objects
aws s3 sync s3://$AWS_BUCKET temp-migration/
gsutil -m rsync -r temp-migration/ gs://$GCP_BUCKET/

# Verify
AWS_COUNT=$(aws s3 ls s3://$AWS_BUCKET --recursive | wc -l)
GCP_COUNT=$(gsutil ls -r gs://$GCP_BUCKET/** | wc -l)

echo "AWS object count: $AWS_COUNT"
echo "GCP object count: $GCP_COUNT"

if [ "$AWS_COUNT" -eq "$GCP_COUNT" ]; then
  echo "✅ Migration successful! Object counts match."
else
  echo "⚠️  Warning: Object counts don't match. Please verify."
fi

# Cleanup
rm -rf temp-migration/
```

#### 3. Configuration Changes

**Before (AWS):**
```toml
[cloud]
enabled = true
provider = "aws"

[cloud.aws]
region = "us-east-1"

[cloud.aws.secrets]
enabled = true

[cloud.aws.storage]
enabled = true
bucket = "llm-shield-models"

[cloud.aws.observability]
metrics_enabled = true
logs_enabled = true
namespace = "LLMShield/API"
log_group = "/aws/llm-shield/api"
log_stream = "production"
```

**After (GCP):**
```toml
[cloud]
enabled = true
provider = "gcp"

[cloud.gcp]
project_id = "llm-shield-prod"

[cloud.gcp.secrets]
enabled = true

[cloud.gcp.storage]
enabled = true
bucket = "llm-shield-models"

[cloud.gcp.observability]
metrics_enabled = true
logs_enabled = true
log_name = "llm-shield-api"
```

### Migrating from AWS to Azure

#### 1. Secret Migration

```bash
#!/bin/bash
# migrate-secrets-aws-to-azure.sh

set -e

AWS_REGION="us-east-1"
AZURE_VAULT_NAME="llm-shield-kv"

echo "Migrating secrets from AWS to Azure..."

# Get list of secrets
SECRETS=$(aws secretsmanager list-secrets \
  --region $AWS_REGION \
  --query 'SecretList[*].Name' \
  --output text)

for SECRET_NAME in $SECRETS; do
  echo "Migrating: $SECRET_NAME"

  # Get secret value from AWS
  SECRET_VALUE=$(aws secretsmanager get-secret-value \
    --region $AWS_REGION \
    --secret-id $SECRET_NAME \
    --query 'SecretString' \
    --output text)

  # Azure Key Vault doesn't allow certain characters in names
  # Replace underscores with hyphens
  AZURE_SECRET_NAME=$(echo $SECRET_NAME | tr '_' '-')

  # Create secret in Azure
  az keyvault secret set \
    --vault-name $AZURE_VAULT_NAME \
    --name $AZURE_SECRET_NAME \
    --value "$SECRET_VALUE"

  echo "✓ Migrated: $SECRET_NAME -> $AZURE_SECRET_NAME"
done

echo "✅ All secrets migrated successfully!"
```

#### 2. Storage Migration

```bash
#!/bin/bash
# migrate-storage-aws-to-azure.sh

set -e

AWS_BUCKET="llm-shield-models"
AZURE_STORAGE_ACCOUNT="llmshieldstorage"
AZURE_CONTAINER="models"

echo "Migrating storage from AWS S3 to Azure Blob Storage..."

# Install azcopy if not already installed
which azcopy || {
  wget -O azcopy.tar.gz https://aka.ms/downloadazcopy-v10-linux
  tar -xf azcopy.tar.gz --strip-components=1
  mv azcopy /usr/local/bin/
}

# Create container if it doesn't exist
az storage container create \
  --account-name $AZURE_STORAGE_ACCOUNT \
  --name $AZURE_CONTAINER \
  --auth-mode login

# Get SAS token for destination
END_DATE=$(date -u -d "7 days" '+%Y-%m-%dT%H:%MZ')
AZURE_SAS=$(az storage container generate-sas \
  --account-name $AZURE_STORAGE_ACCOUNT \
  --name $AZURE_CONTAINER \
  --permissions racwdl \
  --expiry $END_DATE \
  --auth-mode login \
  --as-user \
  --output tsv)

# Copy using azcopy with S3 credentials
export AWS_ACCESS_KEY_ID=$(aws configure get aws_access_key_id)
export AWS_SECRET_ACCESS_KEY=$(aws configure get aws_secret_access_key)

azcopy copy \
  "https://s3.amazonaws.com/$AWS_BUCKET/*" \
  "https://$AZURE_STORAGE_ACCOUNT.blob.core.windows.net/$AZURE_CONTAINER?$AZURE_SAS" \
  --recursive

echo "✅ Migration complete!"
```

### Migrating from GCP to Azure

#### 1. Secret Migration

```bash
#!/bin/bash
# migrate-secrets-gcp-to-azure.sh

set -e

GCP_PROJECT="llm-shield-prod"
AZURE_VAULT_NAME="llm-shield-kv"

echo "Migrating secrets from GCP to Azure..."

# Get list of secrets
SECRETS=$(gcloud secrets list --project $GCP_PROJECT --format="value(name)")

for SECRET_NAME in $SECRETS; do
  echo "Migrating: $SECRET_NAME"

  # Get secret value from GCP
  SECRET_VALUE=$(gcloud secrets versions access latest \
    --secret=$SECRET_NAME \
    --project=$GCP_PROJECT)

  # Azure Key Vault naming constraints
  AZURE_SECRET_NAME=$(echo $SECRET_NAME | tr '_' '-')

  # Create secret in Azure
  az keyvault secret set \
    --vault-name $AZURE_VAULT_NAME \
    --name $AZURE_SECRET_NAME \
    --value "$SECRET_VALUE"

  echo "✓ Migrated: $SECRET_NAME -> $AZURE_SECRET_NAME"
done

echo "✅ All secrets migrated successfully!"
```

#### 2. Storage Migration

```bash
#!/bin/bash
# migrate-storage-gcp-to-azure.sh

set -e

GCP_BUCKET="llm-shield-models"
GCP_PROJECT="llm-shield-prod"
AZURE_STORAGE_ACCOUNT="llmshieldstorage"
AZURE_CONTAINER="models"

echo "Migrating storage from GCP to Azure..."

# Create temp directory
mkdir -p temp-migration

# Download from GCP
gsutil -m rsync -r gs://$GCP_BUCKET/ temp-migration/

# Upload to Azure
az storage blob upload-batch \
  --account-name $AZURE_STORAGE_ACCOUNT \
  --destination $AZURE_CONTAINER \
  --source temp-migration/ \
  --auth-mode login

# Verify counts
GCP_COUNT=$(gsutil ls -r gs://$GCP_BUCKET/** | wc -l)
AZURE_COUNT=$(az storage blob list \
  --account-name $AZURE_STORAGE_ACCOUNT \
  --container-name $AZURE_CONTAINER \
  --auth-mode login \
  --query "length([*])" \
  --output tsv)

echo "GCP object count: $GCP_COUNT"
echo "Azure object count: $AZURE_COUNT"

if [ "$GCP_COUNT" -eq "$AZURE_COUNT" ]; then
  echo "✅ Migration successful!"
else
  echo "⚠️  Warning: Object counts don't match."
fi

# Cleanup
rm -rf temp-migration/
```

## Data Migration

### Secrets Migration Checklist

- [ ] Export all secrets from source provider
- [ ] Verify secret naming compatibility
- [ ] Handle secret rotation policies
- [ ] Update secret references in configuration
- [ ] Test secret access from new provider
- [ ] Update IAM/RBAC permissions
- [ ] Document secret mapping (if names changed)

### Storage Migration Checklist

- [ ] Calculate total data size
- [ ] Estimate transfer time
- [ ] Plan for bandwidth limits
- [ ] Verify storage class compatibility
- [ ] Test integrity after transfer (checksums)
- [ ] Update object ACLs/permissions
- [ ] Verify metadata preservation
- [ ] Test application access

### Metrics/Logs Migration

**Important**: Most cloud providers don't support importing historical metrics/logs.

**Options**:

1. **Start Fresh**: Accept loss of historical data
2. **Export to External System**: Use Prometheus/Grafana/ELK
3. **Dual-Write Period**: Write to both providers for transition

```rust
// Dual-write metrics during migration
tokio::spawn(async move {
    let _ = aws_metrics.export_metric(&metric).await;
});
tokio::spawn(async move {
    let _ = gcp_metrics.export_metric(&metric).await;
});
```

## Rollback Procedures

### Blue-Green Rollback

```bash
# 1. Stop new traffic to green environment
kubectl patch ingress llm-shield-api \
  -p '{"metadata":{"annotations":{"nginx.ingress.kubernetes.io/canary-weight":"0"}}}'

# 2. Route 100% to blue (old) environment
kubectl patch ingress llm-shield-api \
  -p '{"metadata":{"annotations":{"nginx.ingress.kubernetes.io/canary":"false"}}}'

# 3. Verify traffic restored
curl -I https://api.llmshield.example.com/health

# 4. Investigate issues in green environment
kubectl logs -l app=llm-shield-green --tail=1000
```

### Direct Cutover Rollback

```bash
# 1. Redeploy old version
kubectl rollout undo deployment/llm-shield-api

# 2. Restore old configuration
kubectl apply -f backup/configmap-aws.yaml

# 3. Verify deployment
kubectl rollout status deployment/llm-shield-api

# 4. Check health
curl https://api.llmshield.example.com/health
```

## Post-Migration Validation

### 1. Functional Testing

```bash
# Health check
curl https://api.llmshield.example.com/health

# API functionality
curl -X POST https://api.llmshield.example.com/v1/scan \
  -H "Authorization: Bearer $API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"text": "Test content", "scanners": ["secrets"]}'

# List scanners
curl https://api.llmshield.example.com/v1/scanners
```

### 2. Performance Testing

```bash
# Run load test
ab -n 10000 -c 100 \
  -H "Authorization: Bearer $API_KEY" \
  -p test-payload.json \
  https://api.llmshield.example.com/v1/scan

# Compare metrics
# - P50, P95, P99 latency
# - Error rate
# - Throughput (requests/sec)
```

### 3. Observability Validation

```bash
# Check metrics are being exported
curl https://api.llmshield.example.com:9090/metrics | grep llm_shield

# Verify logs are flowing
# AWS
aws logs tail /aws/llm-shield/api --follow

# GCP
gcloud logging read "resource.type=cloud_run_revision" --limit 50

# Azure
az monitor log-analytics query \
  --workspace $WORKSPACE_ID \
  --analytics-query "LLMShieldAPI_CL | take 50"
```

### 4. Data Integrity Checks

```bash
# Verify secret counts match
./scripts/verify-secrets.sh

# Verify storage object counts
./scripts/verify-storage.sh

# Sample and compare objects
./scripts/compare-objects.sh
```

## Troubleshooting

### Issue: Authentication Failures

```bash
# AWS
aws sts get-caller-identity

# GCP
gcloud auth application-default print-access-token

# Azure
az account get-access-token
```

### Issue: Slow Performance

```bash
# Check network latency
ping storage-endpoint.region.provider.com

# Verify regional deployment
# Should be in same region as cloud resources
```

### Issue: Missing Data

```bash
# Re-run migration scripts with verbose logging
./migrate-secrets.sh --verbose
./migrate-storage.sh --verbose --verify
```

## Best Practices

1. **Always Test in Staging First**
2. **Use Infrastructure as Code** (Terraform, CloudFormation)
3. **Automate Migration Scripts**
4. **Document Everything** (especially naming changes)
5. **Keep Blue Environment Running** for 7-14 days
6. **Monitor Costs** during dual-write periods
7. **Communicate with Stakeholders**
8. **Have Rollback Plan Ready**

## Cost Estimation

### Blue-Green Migration Costs

**Duration**: 7-14 days
**Additional Costs**:
- 2x compute resources: $300-600
- Data transfer: $50-200 (depending on data size)
- Total: $350-800

### Direct Cutover Migration Costs

**Duration**: 1-2 days
**Additional Costs**:
- Data transfer: $50-200
- Minimal compute overhead: $20-50
- Total: $70-250

## Support

For migration assistance:
- Migration consulting: consulting@llmshield.dev
- Emergency support: support@llmshield.dev (24/7)
- Community forum: https://forum.llmshield.dev
