# IAM Policies for LLM Shield AWS Integration

This directory contains IAM policy templates for granting LLM Shield the necessary permissions to access AWS services.

## Policy Files

### Individual Service Policies

1. **`secrets-manager-policy.json`** - AWS Secrets Manager permissions
   - Read secrets (GetSecretValue, DescribeSecret, ListSecrets)
   - Write secrets (CreateSecret, UpdateSecret, PutSecretValue)
   - Delete secrets (DeleteSecret, RestoreSecret)
   - Rotate secrets (RotateSecret, GetRandomPassword)

2. **`s3-policy.json`** - AWS S3 storage permissions
   - Bucket operations (ListBucket, GetBucketLocation)
   - Object read (GetObject, GetObjectVersion, GetObjectMetadata)
   - Object write (PutObject, DeleteObject)
   - Multipart uploads (CreateMultipartUpload, UploadPart, CompleteMultipartUpload)
   - KMS encryption for S3 objects

3. **`cloudwatch-policy.json`** - AWS CloudWatch observability permissions
   - Metrics (PutMetricData, GetMetricData, ListMetrics)
   - Logs (CreateLogGroup, CreateLogStream, PutLogEvents, GetLogEvents)
   - Alarms (PutMetricAlarm, DescribeAlarms, DeleteAlarms)
   - Dashboards (PutDashboard, GetDashboard, DeleteDashboards)

### Combined Policy

4. **`llm-shield-full-policy.json`** - All permissions combined
   - Use this for development and testing
   - For production, use individual service policies for least-privilege access

## Usage

### Option 1: Create IAM User (Development)

For development and testing, create an IAM user with programmatic access:

```bash
# Create IAM user
aws iam create-user --user-name llm-shield-dev

# Attach policy
aws iam put-user-policy \
  --user-name llm-shield-dev \
  --policy-name LLMShieldFullAccess \
  --policy-document file://llm-shield-full-policy.json

# Create access key
aws iam create-access-key --user-name llm-shield-dev
```

Set environment variables:

```bash
export AWS_ACCESS_KEY_ID=<access-key-id>
export AWS_SECRET_ACCESS_KEY=<secret-access-key>
export AWS_DEFAULT_REGION=us-east-1
```

### Option 2: IAM Role for EC2 (Production)

For EC2 instances, attach an IAM role with the appropriate policies:

```bash
# Create IAM role
aws iam create-role \
  --role-name LLMShieldEC2Role \
  --assume-role-policy-document '{
    "Version": "2012-10-17",
    "Statement": [{
      "Effect": "Allow",
      "Principal": {"Service": "ec2.amazonaws.com"},
      "Action": "sts:AssumeRole"
    }]
  }'

# Attach policies
aws iam put-role-policy \
  --role-name LLMShieldEC2Role \
  --policy-name LLMShieldFullAccess \
  --policy-document file://llm-shield-full-policy.json

# Create instance profile
aws iam create-instance-profile \
  --instance-profile-name LLMShieldEC2Profile

# Add role to instance profile
aws iam add-role-to-instance-profile \
  --instance-profile-name LLMShieldEC2Profile \
  --role-name LLMShieldEC2Role
```

Launch EC2 instance with this profile:

```bash
aws ec2 run-instances \
  --image-id ami-xxxxx \
  --instance-type t3.medium \
  --iam-instance-profile Name=LLMShieldEC2Profile \
  ...
```

### Option 3: IAM Role for ECS (Containers)

For ECS tasks:

```bash
# Create IAM role for ECS task
aws iam create-role \
  --role-name LLMShieldECSTaskRole \
  --assume-role-policy-document '{
    "Version": "2012-10-17",
    "Statement": [{
      "Effect": "Allow",
      "Principal": {"Service": "ecs-tasks.amazonaws.com"},
      "Action": "sts:AssumeRole"
    }]
  }'

# Attach policies
aws iam put-role-policy \
  --role-name LLMShieldECSTaskRole \
  --policy-name LLMShieldFullAccess \
  --policy-document file://llm-shield-full-policy.json
```

Reference in ECS task definition:

```json
{
  "family": "llm-shield-api",
  "taskRoleArn": "arn:aws:iam::ACCOUNT_ID:role/LLMShieldECSTaskRole",
  "containerDefinitions": [...]
}
```

### Option 4: IRSA for EKS (Kubernetes)

For EKS pods using IAM Roles for Service Accounts (IRSA):

```bash
# Create OIDC provider (if not already done)
eksctl utils associate-iam-oidc-provider \
  --cluster llm-shield-cluster \
  --approve

# Create service account with IAM role
eksctl create iamserviceaccount \
  --name llm-shield-sa \
  --namespace default \
  --cluster llm-shield-cluster \
  --attach-policy-arn arn:aws:iam::ACCOUNT_ID:policy/LLMShieldFullAccess \
  --approve \
  --override-existing-serviceaccounts
```

Reference in Kubernetes deployment:

```yaml
apiVersion: v1
kind: Pod
metadata:
  name: llm-shield-api
spec:
  serviceAccountName: llm-shield-sa
  containers:
    - name: api
      image: llm-shield:latest
```

## Resource Naming Conventions

The policies follow these naming conventions:

### Secrets Manager
- **Prefix**: `llm-shield/`
- **Examples**:
  - `llm-shield/openai-api-key`
  - `llm-shield/database-password`
  - `llm-shield/jwt-secret`

### S3 Buckets
- **Pattern**: `llm-shield-*`
- **Examples**:
  - `llm-shield-models-prod`
  - `llm-shield-results-dev`
  - `llm-shield-configs`

### S3 Object Prefixes
- **Models**: `models/`
- **Scan Results**: `scan-results/`
- **Configs**: `configs/`

### CloudWatch
- **Namespaces**:
  - `LLMShield`
  - `LLMShield/API`
  - `LLMShield/Scanners`
- **Log Groups**: `/llm-shield/*`
  - `/llm-shield/api`
  - `/llm-shield/scanners`
  - `/llm-shield/workers`

## Security Best Practices

### 1. Least Privilege

For production, use individual service policies and only grant the permissions you need:

```bash
# Only Secrets Manager (read-only)
aws iam put-role-policy \
  --role-name LLMShieldRole \
  --policy-name SecretsManagerReadOnly \
  --policy-document file://secrets-manager-policy.json
```

### 2. Resource Restrictions

Further restrict resources using ARN patterns:

```json
{
  "Resource": [
    "arn:aws:secretsmanager:us-east-1:123456789012:secret:llm-shield/*"
  ]
}
```

### 3. Condition Keys

Use condition keys to restrict access:

```json
{
  "Condition": {
    "StringEquals": {
      "aws:RequestedRegion": ["us-east-1", "eu-west-1"]
    },
    "IpAddress": {
      "aws:SourceIp": ["10.0.0.0/8"]
    }
  }
}
```

### 4. MFA Requirements

Require MFA for sensitive operations:

```json
{
  "Condition": {
    "BoolIfExists": {
      "aws:MultiFactorAuthPresent": "true"
    }
  }
}
```

### 5. Encryption Requirements

Enforce encryption for S3 uploads:

```json
{
  "Condition": {
    "StringNotEquals": {
      "s3:x-amz-server-side-encryption": "AES256"
    }
  },
  "Effect": "Deny",
  "Action": "s3:PutObject"
}
```

## Testing Permissions

Test IAM permissions using the AWS CLI:

```bash
# Test Secrets Manager access
aws secretsmanager get-secret-value \
  --secret-id llm-shield/test-secret \
  --query SecretString \
  --output text

# Test S3 access
aws s3 ls s3://llm-shield-models/

# Test CloudWatch Logs access
aws logs describe-log-groups \
  --log-group-name-prefix /llm-shield/
```

## Troubleshooting

### Access Denied Errors

If you see `AccessDenied` errors:

1. **Check IAM role/user**: Verify the correct role is attached
   ```bash
   aws sts get-caller-identity
   ```

2. **Check policy**: Verify the policy is attached
   ```bash
   aws iam list-attached-role-policies --role-name LLMShieldRole
   aws iam list-role-policies --role-name LLMShieldRole
   ```

3. **Check resource names**: Ensure resources follow naming conventions
   - Secrets: `llm-shield/*`
   - Buckets: `llm-shield-*`
   - Log groups: `/llm-shield/*`

4. **Check regions**: Ensure you're operating in the correct region
   ```bash
   echo $AWS_DEFAULT_REGION
   ```

### Permission Denied on KMS

If encryption fails:

```bash
# Check KMS key policy
aws kms get-key-policy \
  --key-id <key-id> \
  --policy-name default \
  --output text
```

Ensure the KMS key policy grants access to the IAM role.

## Cost Estimates

Typical monthly costs with these permissions:

- **Secrets Manager**: ~$0.40 per secret + $0.05 per 10,000 API calls
- **S3**: ~$0.023 per GB stored + $0.005 per 1,000 PUT requests
- **CloudWatch Logs**: ~$0.50 per GB ingested + $0.03 per GB stored
- **CloudWatch Metrics**: First 10 custom metrics free, then $0.30 per metric

**Example** (10 secrets, 100GB S3, 50GB logs/month):
- Secrets Manager: ~$5
- S3: ~$2.50
- CloudWatch Logs: ~$26.50
- CloudWatch Metrics: ~$3
- **Total**: ~$37/month

## License

MIT OR Apache-2.0
