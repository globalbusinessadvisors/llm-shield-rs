# Secrets Directory

This directory contains sensitive configuration files for LLM Shield.

## ⚠️ Security Warning

**NEVER commit actual secrets to version control!**

This directory should be added to `.gitignore` in production environments. The files here are examples for local development only.

## API Keys

### Local Development

For local development, you can use plain-text API keys in `api_keys.txt`:

```bash
# Generate a secure API key
openssl rand -base64 32

# Or using /dev/urandom
head -c 32 /dev/urandom | base64
```

Add the generated keys to `api_keys.txt` (one per line).

### Production Deployment

In production, **NEVER** store secrets in plain text files. Use proper secret management:

#### Kubernetes Secrets

```bash
# Create a Kubernetes secret
kubectl create secret generic llm-shield-api-keys \
  --from-literal=api-key-1="your-secure-key-here"

# Reference in deployment
env:
  - name: LLM_SHIELD_API_KEY
    valueFrom:
      secretKeyRef:
        name: llm-shield-api-keys
        key: api-key-1
```

#### AWS Secrets Manager

```bash
# Store secret
aws secretsmanager create-secret \
  --name llm-shield/api-keys \
  --secret-string '["key1", "key2"]'

# Reference using IAM role and IRSA
```

#### HashiCorp Vault

```bash
# Store in Vault
vault kv put secret/llm-shield/api-keys keys=@api_keys.json

# Use Vault Agent or CSI driver to inject
```

#### Azure Key Vault

```bash
# Store secret
az keyvault secret set \
  --vault-name llm-shield-vault \
  --name api-keys \
  --value "your-key"
```

#### GCP Secret Manager

```bash
# Create secret
gcloud secrets create llm-shield-api-keys \
  --data-file=api_keys.txt

# Grant access to service account
gcloud secrets add-iam-policy-binding llm-shield-api-keys \
  --member="serviceAccount:llm-shield@project.iam.gserviceaccount.com" \
  --role="roles/secretmanager.secretAccessor"
```

## API Key Management Best Practices

1. **Rotate keys regularly** (every 90 days)
2. **Use strong, random keys** (at least 32 bytes)
3. **Never log API keys** (they're automatically redacted by LLM Shield)
4. **Monitor key usage** (track in Prometheus/Grafana)
5. **Revoke compromised keys immediately**
6. **Use separate keys per client** (for auditing and rate limiting)
7. **Hash keys at rest** (LLM Shield uses argon2id)

## Key Rotation Procedure

```bash
# 1. Generate new key
NEW_KEY=$(openssl rand -base64 32)

# 2. Add to secrets (keep old key temporarily)
echo "$NEW_KEY" >> api_keys.txt

# 3. Update clients to use new key

# 4. Wait for grace period (e.g., 7 days)

# 5. Remove old key from api_keys.txt

# 6. Restart service
docker-compose restart llm-shield
```

## Environment-Specific Keys

Use different API keys for different environments:

- **Development**: Simple keys for testing (e.g., `dev-key-123`)
- **Staging**: Production-like keys (e.g., `stg-key-xyz`)
- **Production**: Highly secure, rotated keys from secret manager

## Troubleshooting

### Authentication Failed

```bash
# Check if API keys are loaded
docker-compose exec llm-shield cat /run/secrets/api_keys

# Test authentication
curl -H "Authorization: Bearer your-api-key" http://localhost:8080/health
```

### Key Not Found

```bash
# Verify secret is mounted
docker-compose exec llm-shield ls -la /run/secrets/

# Check logs
docker-compose logs llm-shield | grep -i auth
```

## Further Reading

- [LLM Shield Security Documentation](../../docs/security.md)
- [OWASP API Security Top 10](https://owasp.org/www-project-api-security/)
- [NIST Secret Management Guidelines](https://csrc.nist.gov/publications/detail/sp/800-57-part-1/rev-5/final)
