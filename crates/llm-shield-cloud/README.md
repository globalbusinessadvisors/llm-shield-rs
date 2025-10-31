# llm-shield-cloud

Cloud abstraction layer for LLM Shield providing unified traits for AWS, GCP, and Azure integrations.

## Overview

This crate provides trait-based abstractions for cloud services, enabling LLM Shield to leverage cloud-native features while maintaining portability across providers.

## Features

- **Secret Management**: Unified `CloudSecretManager` trait for AWS Secrets Manager, GCP Secret Manager, and Azure Key Vault
- **Object Storage**: `CloudStorage` trait for AWS S3, GCP Cloud Storage, and Azure Blob Storage
- **Observability**: `CloudMetrics`, `CloudLogger`, and `CloudTracer` traits for cloud-native monitoring
- **Configuration**: Type-safe configuration structures for all cloud providers
- **Caching**: Built-in secret caching with TTL support
- **Error Handling**: Unified error types across all cloud operations

## Architecture

```
┌────────────────────────────────────┐
│   LLM Shield Application           │
└────────────────────────────────────┘
               │
               ▼
┌────────────────────────────────────┐
│   llm-shield-cloud (traits)        │
│   - CloudSecretManager             │
│   - CloudStorage                   │
│   - CloudMetrics/Logger/Tracer     │
└────────────────────────────────────┘
      │             │             │
      ▼             ▼             ▼
┌──────────┐  ┌──────────┐  ┌──────────┐
│   AWS    │  │   GCP    │  │  Azure   │
│ Provider │  │ Provider │  │ Provider │
└──────────┘  └──────────┘  └──────────┘
```

## Usage

### Basic Example

```rust
use llm_shield_cloud::{CloudSecretManager, SecretValue, Result};

async fn load_api_keys(
    secret_manager: &dyn CloudSecretManager
) -> Result<Vec<String>> {
    // Fetch API keys from cloud secret manager
    let secret = secret_manager.get_secret("llm-shield/api-keys").await?;

    // Parse the secret value
    let api_keys: Vec<String> = serde_json::from_str(secret.as_string())?;

    Ok(api_keys)
}
```

### Secret Caching

```rust
use llm_shield_cloud::SecretCache;
use std::time::Duration;

let cache = SecretCache::new(300); // 5 minute TTL

// Set a secret in cache
cache.set("my-key".to_string(), secret_value).await;

// Get from cache (returns None if expired)
if let Some(value) = cache.get("my-key").await {
    println!("Cache hit!");
}
```

### Storage Operations

```rust
use llm_shield_cloud::{CloudStorage, PutObjectOptions};

async fn upload_model(storage: &dyn CloudStorage) -> Result<()> {
    let model_data = tokio::fs::read("model.onnx").await?;

    let options = PutObjectOptions {
        content_type: Some("application/octet-stream".to_string()),
        storage_class: Some("STANDARD".to_string()),
        ..Default::default()
    };

    storage.put_object_with_options(
        "models/toxicity.onnx",
        &model_data,
        &options
    ).await?;

    Ok(())
}
```

## Configuration

Cloud integrations are configured via `CloudConfig`:

```yaml
cloud:
  provider: aws  # or gcp, azure, none

  aws:
    region: us-east-1
    secrets_manager:
      enabled: true
      cache_ttl_seconds: 300
    s3:
      bucket: llm-shield-models
      models_prefix: models/
      results_prefix: scan-results/
    cloudwatch:
      enabled: true
      namespace: LLMShield
      log_group: /llm-shield/api
```

## Providers

Concrete implementations are provided by separate crates:

- **`llm-shield-cloud-aws`**: AWS integrations (Secrets Manager, S3, CloudWatch, X-Ray)
- **`llm-shield-cloud-gcp`**: GCP integrations (Secret Manager, Cloud Storage, Cloud Logging, Cloud Trace)
- **`llm-shield-cloud-azure`**: Azure integrations (Key Vault, Blob Storage, Azure Monitor, App Insights)

Enable provider-specific features in your `Cargo.toml`:

```toml
[dependencies]
llm-shield-cloud = "0.1"
llm-shield-cloud-aws = { version = "0.1", optional = true }

[features]
cloud-aws = ["llm-shield-cloud-aws"]
```

## Error Handling

All cloud operations return `Result<T, CloudError>`:

```rust
use llm_shield_cloud::{CloudError, Result};

match secret_manager.get_secret("my-secret").await {
    Ok(value) => println!("Secret: {}", value.as_string()),
    Err(CloudError::SecretNotFound(name)) => {
        eprintln!("Secret not found: {}", name);
    }
    Err(e) => {
        eprintln!("Failed to fetch secret: {}", e);
    }
}
```

## Testing

Run tests:

```bash
cargo test -p llm-shield-cloud
```

Run tests with output:

```bash
cargo test -p llm-shield-cloud -- --nocapture
```

## Performance

- **Caching**: Built-in secret caching reduces API calls by >90%
- **Async**: All operations are fully async with tokio
- **Zero-cost abstractions**: Trait-based design adds <5% overhead

## Security

- Zero plain-text secrets in code or configuration
- Automatic credential rotation support
- Constant-time comparison for sensitive data
- Comprehensive audit logging

## License

MIT OR Apache-2.0
