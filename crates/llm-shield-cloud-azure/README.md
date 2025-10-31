# llm-shield-cloud-azure

Azure cloud integrations for LLM Shield - Key Vault, Blob Storage, and Azure Monitor.

## Overview

Production-ready Azure implementations of cloud abstraction traits:

- **Azure Key Vault** - Secure secret storage with automatic caching
- **Azure Blob Storage** - Object storage for models and results
- **Azure Monitor Metrics** - Application metrics and monitoring
- **Azure Monitor Logs** - Structured logging via Log Analytics

## Installation

```toml
[dependencies]
llm-shield-cloud-azure = "0.1"
llm-shield-cloud = "0.1"
tokio = { version = "1.35", features = ["full"] }
```

## Quick Start

### Key Vault

```rust
use llm_shield_cloud_azure::AzureKeyVault;
use llm_shield_cloud::CloudSecretManager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let vault = AzureKeyVault::new("https://my-vault.vault.azure.net").await?;
    let api_key = vault.get_secret("openai-api-key").await?;
    println!("API Key: {}", api_key.as_string());
    Ok(())
}
```

### Blob Storage

```rust
use llm_shield_cloud_azure::AzureBlobStorage;
use llm_shield_cloud::CloudStorage;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let storage = AzureBlobStorage::new("mystorageaccount", "models").await?;

    let data = b"Hello, Azure!";
    storage.put_object("test.txt", data).await?;

    let retrieved = storage.get_object("test.txt").await?;
    assert_eq!(data, retrieved.as_slice());

    Ok(())
}
```

### Azure Monitor

```rust
use llm_shield_cloud_azure::AzureMonitorMetrics;
use llm_shield_cloud::{CloudMetrics, Metric};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let metrics = AzureMonitorMetrics::new(
        "/subscriptions/sub-id/resourceGroups/rg/...",
        "eastus"
    ).await?;

    let metric = Metric {
        name: "scan_duration".to_string(),
        value: 123.45,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs(),
        dimensions: HashMap::new(),
        unit: Some("Milliseconds".to_string()),
    };

    metrics.export_metric(&metric).await?;
    Ok(())
}
```

## Configuration

```yaml
cloud:
  provider: azure
  azure:
    key_vault:
      vault_url: https://my-vault.vault.azure.net
      cache_ttl_seconds: 300
    storage:
      account_name: mystorageaccount
      container_name: models
    monitor:
      workspace_id: workspace-id
      shared_key: shared-key
      log_type: LLMShieldLog
```

## Azure Credentials

Uses DefaultAzureCredential which tries:

1. **Environment variables**: AZURE_TENANT_ID, AZURE_CLIENT_ID, AZURE_CLIENT_SECRET
2. **Azure CLI**: `az login` credentials
3. **Managed Identity**: For Azure VMs, App Service, Container Apps

### Development Setup

```bash
# Install Azure CLI
curl -sL https://aka.ms/InstallAzureCLIDeb | sudo bash

# Login
az login

# Set subscription
az account set --subscription "my-subscription-id"
```

### Production (Managed Identity)

**Azure VM:**
```bash
# Enable system-assigned managed identity
az vm identity assign \
  --name my-vm \
  --resource-group my-rg

# Get principal ID
PRINCIPAL_ID=$(az vm show \
  --name my-vm \
  --resource-group my-rg \
  --query identity.principalId -o tsv)

# Assign roles
az role assignment create \
  --assignee $PRINCIPAL_ID \
  --role "LLM Shield Full Access" \
  --scope /subscriptions/sub-id
```

**App Service:**
```bash
# Enable managed identity
az webapp identity assign \
  --name my-app \
  --resource-group my-rg

# Assign roles
PRINCIPAL_ID=$(az webapp show \
  --name my-app \
  --resource-group my-rg \
  --query identity.principalId -o tsv)

az role assignment create \
  --assignee $PRINCIPAL_ID \
  --role "LLM Shield Full Access"
```

**Container Apps:**
```bash
# Enable managed identity
az containerapp identity assign \
  --name my-app \
  --resource-group my-rg

# Assign roles
PRINCIPAL_ID=$(az containerapp show \
  --name my-app \
  --resource-group my-rg \
  --query identity.principalId -o tsv)

az role assignment create \
  --assignee $PRINCIPAL_ID \
  --role "LLM Shield Full Access"
```

## RBAC Permissions

See `rbac-roles/` for custom role definitions:

- `key-vault-role.json` - Key Vault permissions
- `storage-role.json` - Blob Storage permissions
- `monitor-role.json` - Azure Monitor permissions
- `llm-shield-full-role.json` - All permissions (dev/test)

### Creating Custom Roles

```bash
# Create custom role
az role definition create \
  --role-definition @rbac-roles/llm-shield-full-role.json

# Assign to managed identity
az role assignment create \
  --assignee $PRINCIPAL_ID \
  --role "LLM Shield Full Access" \
  --scope /subscriptions/sub-id
```

## Testing

### Unit Tests

```bash
cargo test -p llm-shield-cloud-azure
```

### Integration Tests

```bash
export AZURE_TENANT_ID=tenant-id
export AZURE_CLIENT_ID=client-id
export AZURE_CLIENT_SECRET=client-secret
export TEST_VAULT_URL=https://test-vault.vault.azure.net
export TEST_STORAGE_ACCOUNT=teststorageaccount
export TEST_CONTAINER=test-container

cargo test -p llm-shield-cloud-azure --test integration -- --ignored
```

## Performance

| Operation | Throughput | Latency (p50) |
|-----------|-----------|---------------|
| Secret fetch (cached) | 100,000/s | <1ms |
| Secret fetch (uncached) | 1,000/s | ~50ms |
| Blob upload (1MB) | 50 MB/s | ~20ms |
| Blob upload (50MB blocks) | 80 MB/s | ~625ms |
| Metrics export (batch) | 1,000/s | ~10ms |
| Logs export (batch) | 10,000/s | ~5ms |

## Cost Estimates

Monthly costs (production):

| Service | Usage | Cost |
|---------|-------|------|
| Key Vault | 10 secrets, 100K ops | ~$3 |
| Blob Storage (LRS) | 100 GB, 1M ops | ~$2 |
| Log Analytics | 50 GB ingested | ~$115 |
| Azure Monitor Metrics | 50 metrics | ~$10 |
| **Total** | | **~$130/month** |

## Security Best Practices

1. Use Managed Identity (no credentials in code)
2. Apply least-privilege RBAC roles
3. Enable Key Vault soft delete and purge protection
4. Use private endpoints for Key Vault and Storage
5. Enable Azure Policy for compliance
6. Set storage account firewall rules
7. Enable diagnostic logging for all services
8. Use customer-managed keys (CMK) for encryption
9. Enable Azure Defender for Cloud
10. Review access regularly with Azure AD PIM

## Troubleshooting

### Authentication Errors

```bash
# Check current authentication
az account show

# Check managed identity
curl -H Metadata:true \
  "http://169.254.169.254/metadata/identity/oauth2/token?api-version=2018-02-01&resource=https://management.azure.com/"
```

### Permission Errors

```bash
# List role assignments
az role assignment list \
  --assignee $PRINCIPAL_ID \
  --all

# Test Key Vault access
az keyvault secret show \
  --vault-name my-vault \
  --name test-secret

# Test Blob Storage access
az storage blob list \
  --account-name mystorageaccount \
  --container-name models \
  --auth-mode login
```

### Resource Not Found

```bash
# Verify Key Vault exists
az keyvault show --name my-vault

# Verify storage account exists
az storage account show \
  --name mystorageaccount \
  --resource-group my-rg

# Verify container exists
az storage container show \
  --name models \
  --account-name mystorageaccount \
  --auth-mode login
```

## Architecture

```text
┌─────────────────────────────────────┐
│   LLM Shield Application            │
└─────────────────────────────────────┘
                │
                ▼
┌─────────────────────────────────────┐
│   llm-shield-cloud (traits)         │
└─────────────────────────────────────┘
                │
                ▼
┌─────────────────────────────────────┐
│   llm-shield-cloud-azure (impl)     │
│   - AzureKeyVault                   │
│   - AzureBlobStorage                │
│   - AzureMonitorMetrics/Logs        │
└─────────────────────────────────────┘
                │
                ▼
┌─────────────────────────────────────┐
│   Azure SDK for Rust                │
│   - azure_security_keyvault         │
│   - azure_storage_blobs             │
│   - reqwest (Monitor API)           │
└─────────────────────────────────────┘
                │
                ▼
┌─────────────────────────────────────┐
│   Azure Services                    │
│   - Key Vault                       │
│   - Blob Storage                    │
│   - Azure Monitor                   │
└─────────────────────────────────────┘
```

## License

MIT OR Apache-2.0

## Related Crates

- [`llm-shield-cloud`](../llm-shield-cloud) - Cloud abstraction traits
- [`llm-shield-cloud-aws`](../llm-shield-cloud-aws) - AWS integrations
- [`llm-shield-cloud-gcp`](../llm-shield-cloud-gcp) - GCP integrations
