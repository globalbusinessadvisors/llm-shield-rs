#!/bin/bash
# Deploy LLM Shield API to AWS
#
# Prerequisites:
# - AWS CLI configured
# - Docker installed
# - ECR repository created
# - ECS cluster created

set -e

# Configuration
AWS_REGION="${AWS_REGION:-us-east-1}"
AWS_ACCOUNT_ID=$(aws sts get-caller-identity --query Account --output text)
ECR_REPO="llm-shield-api"
ECS_CLUSTER="llm-shield-cluster"
ECS_SERVICE="llm-shield-api"
TASK_FAMILY="llm-shield-api"
IMAGE_TAG="${IMAGE_TAG:-latest}"

echo "🚀 Deploying LLM Shield API to AWS"
echo "   Region: $AWS_REGION"
echo "   Account: $AWS_ACCOUNT_ID"
echo ""

# Step 1: Build Docker image
echo "📦 Building Docker image..."
docker build \
  --build-arg FEATURES=cloud-aws \
  -t llm-shield-api:$IMAGE_TAG \
  -f examples/cloud/Dockerfile \
  ../..

# Step 2: Tag and push to ECR
echo "📤 Pushing to ECR..."
aws ecr get-login-password --region $AWS_REGION | \
  docker login --username AWS --password-stdin $AWS_ACCOUNT_ID.dkr.ecr.$AWS_REGION.amazonaws.com

docker tag llm-shield-api:$IMAGE_TAG \
  $AWS_ACCOUNT_ID.dkr.ecr.$AWS_REGION.amazonaws.com/$ECR_REPO:$IMAGE_TAG

docker push $AWS_ACCOUNT_ID.dkr.ecr.$AWS_REGION.amazonaws.com/$ECR_REPO:$IMAGE_TAG

# Step 3: Update ECS task definition
echo "📝 Updating ECS task definition..."
TASK_DEF=$(cat <<EOF
{
  "family": "$TASK_FAMILY",
  "networkMode": "awsvpc",
  "requiresCompatibilities": ["FARGATE"],
  "cpu": "1024",
  "memory": "2048",
  "executionRoleArn": "arn:aws:iam::$AWS_ACCOUNT_ID:role/ecsTaskExecutionRole",
  "taskRoleArn": "arn:aws:iam::$AWS_ACCOUNT_ID:role/LLMShieldAPIRole",
  "containerDefinitions": [
    {
      "name": "llm-shield-api",
      "image": "$AWS_ACCOUNT_ID.dkr.ecr.$AWS_REGION.amazonaws.com/$ECR_REPO:$IMAGE_TAG",
      "portMappings": [
        {
          "containerPort": 8080,
          "protocol": "tcp"
        },
        {
          "containerPort": 9090,
          "protocol": "tcp"
        }
      ],
      "environment": [
        {
          "name": "AWS_REGION",
          "value": "$AWS_REGION"
        },
        {
          "name": "RUST_LOG",
          "value": "info"
        }
      ],
      "logConfiguration": {
        "logDriver": "awslogs",
        "options": {
          "awslogs-group": "/aws/llm-shield/api",
          "awslogs-region": "$AWS_REGION",
          "awslogs-stream-prefix": "ecs"
        }
      },
      "healthCheck": {
        "command": ["CMD-SHELL", "curl -f http://localhost:8080/health || exit 1"],
        "interval": 30,
        "timeout": 5,
        "retries": 3,
        "startPeriod": 60
      }
    }
  ]
}
EOF
)

aws ecs register-task-definition \
  --region $AWS_REGION \
  --cli-input-json "$TASK_DEF"

# Step 4: Update ECS service
echo "🔄 Updating ECS service..."
aws ecs update-service \
  --region $AWS_REGION \
  --cluster $ECS_CLUSTER \
  --service $ECS_SERVICE \
  --task-definition $TASK_FAMILY \
  --force-new-deployment

# Step 5: Wait for deployment
echo "⏳ Waiting for deployment to complete..."
aws ecs wait services-stable \
  --region $AWS_REGION \
  --cluster $ECS_CLUSTER \
  --services $ECS_SERVICE

echo "✅ Deployment complete!"
echo ""
echo "Service URL: https://api.llmshield.example.com"
echo "Metrics: http://$(aws ecs describe-tasks --region $AWS_REGION --cluster $ECS_CLUSTER --tasks $(aws ecs list-tasks --region $AWS_REGION --cluster $ECS_CLUSTER --service-name $ECS_SERVICE --query 'taskArns[0]' --output text) --query 'tasks[0].containers[0].networkInterfaces[0].privateIpv4Address' --output text):9090/metrics"
