#!/bin/bash
# Deploy LLM Shield API to Azure
#
# Prerequisites:
# - Azure CLI configured
# - Docker installed
# - Azure Container Registry created
# - Azure Container Apps OR AKS cluster created

set -e

# Configuration
AZURE_SUBSCRIPTION="${AZURE_SUBSCRIPTION:-$(az account show --query id -o tsv)}"
AZURE_RESOURCE_GROUP="${AZURE_RESOURCE_GROUP:-llm-shield-rg}"
AZURE_LOCATION="${AZURE_LOCATION:-eastus}"
ACR_NAME="${ACR_NAME:-llmshieldacr}"
SERVICE_NAME="llm-shield-api"
IMAGE_TAG="${IMAGE_TAG:-latest}"
DEPLOY_TARGET="${DEPLOY_TARGET:-container-apps}"  # Options: container-apps, aks

echo "🚀 Deploying LLM Shield API to Azure"
echo "   Subscription: $AZURE_SUBSCRIPTION"
echo "   Resource Group: $AZURE_RESOURCE_GROUP"
echo "   Location: $AZURE_LOCATION"
echo "   Target: $DEPLOY_TARGET"
echo ""

# Set subscription
az account set --subscription $AZURE_SUBSCRIPTION

# Step 1: Build Docker image
echo "📦 Building Docker image..."
docker build \
  --build-arg FEATURES=cloud-azure \
  -t $SERVICE_NAME:$IMAGE_TAG \
  -f examples/cloud/Dockerfile \
  ../..

# Step 2: Push to ACR
echo "📤 Pushing to ACR..."
az acr login --name $ACR_NAME
docker tag $SERVICE_NAME:$IMAGE_TAG $ACR_NAME.azurecr.io/$SERVICE_NAME:$IMAGE_TAG
docker push $ACR_NAME.azurecr.io/$SERVICE_NAME:$IMAGE_TAG

if [ "$DEPLOY_TARGET" == "container-apps" ]; then
  # Deploy to Azure Container Apps
  echo "📦 Deploying to Azure Container Apps..."

  CONTAINER_APP_ENV="${CONTAINER_APP_ENV:-llm-shield-env}"

  # Create/update container app
  az containerapp up \
    --name $SERVICE_NAME \
    --resource-group $AZURE_RESOURCE_GROUP \
    --location $AZURE_LOCATION \
    --environment $CONTAINER_APP_ENV \
    --image $ACR_NAME.azurecr.io/$SERVICE_NAME:$IMAGE_TAG \
    --target-port 8080 \
    --ingress external \
    --cpu 2 \
    --memory 4Gi \
    --min-replicas 1 \
    --max-replicas 10 \
    --registry-server $ACR_NAME.azurecr.io \
    --env-vars \
      RUST_LOG=info \
      AZURE_SUBSCRIPTION_ID=$AZURE_SUBSCRIPTION

  # Enable system-assigned managed identity
  az containerapp identity assign \
    --name $SERVICE_NAME \
    --resource-group $AZURE_RESOURCE_GROUP \
    --system-assigned

  # Get managed identity principal ID
  PRINCIPAL_ID=$(az containerapp show \
    --name $SERVICE_NAME \
    --resource-group $AZURE_RESOURCE_GROUP \
    --query identity.principalId -o tsv)

  echo "🔐 Assigning RBAC roles to managed identity..."

  # Assign Key Vault role
  az role assignment create \
    --assignee $PRINCIPAL_ID \
    --role "LLM Shield Full Access" \
    --scope /subscriptions/$AZURE_SUBSCRIPTION/resourceGroups/$AZURE_RESOURCE_GROUP

  # Get service FQDN
  SERVICE_FQDN=$(az containerapp show \
    --name $SERVICE_NAME \
    --resource-group $AZURE_RESOURCE_GROUP \
    --query properties.configuration.ingress.fqdn -o tsv)

  echo "✅ Deployment complete!"
  echo ""
  echo "Service URL: https://$SERVICE_FQDN"
  echo "Health check: https://$SERVICE_FQDN/health"

elif [ "$DEPLOY_TARGET" == "aks" ]; then
  # Deploy to AKS
  echo "☸️  Deploying to AKS..."

  AKS_CLUSTER="${AKS_CLUSTER:-llm-shield-aks}"

  # Get AKS credentials
  az aks get-credentials \
    --resource-group $AZURE_RESOURCE_GROUP \
    --name $AKS_CLUSTER \
    --overwrite-existing

  # Apply Kubernetes manifests
  kubectl apply -f - <<EOF
apiVersion: apps/v1
kind: Deployment
metadata:
  name: $SERVICE_NAME
  labels:
    app: llm-shield
    azure.workload.identity/use: "true"
spec:
  replicas: 3
  selector:
    matchLabels:
      app: llm-shield
  template:
    metadata:
      labels:
        app: llm-shield
        azure.workload.identity/use: "true"
    spec:
      serviceAccountName: llm-shield-api
      containers:
      - name: api
        image: $ACR_NAME.azurecr.io/$SERVICE_NAME:$IMAGE_TAG
        ports:
        - containerPort: 8080
          name: http
        - containerPort: 9090
          name: metrics
        env:
        - name: RUST_LOG
          value: "info"
        - name: AZURE_SUBSCRIPTION_ID
          value: "$AZURE_SUBSCRIPTION"
        resources:
          requests:
            memory: "2Gi"
            cpu: "1000m"
          limits:
            memory: "4Gi"
            cpu: "2000m"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 5
---
apiVersion: v1
kind: Service
metadata:
  name: $SERVICE_NAME
  labels:
    app: llm-shield
  annotations:
    service.beta.kubernetes.io/azure-load-balancer-internal: "false"
spec:
  type: LoadBalancer
  ports:
  - port: 80
    targetPort: 8080
    name: http
  - port: 9090
    targetPort: 9090
    name: metrics
  selector:
    app: llm-shield
---
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: $SERVICE_NAME
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: $SERVICE_NAME
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
EOF

  # Wait for rollout
  echo "⏳ Waiting for rollout to complete..."
  kubectl rollout status deployment/$SERVICE_NAME

  # Get service IP
  SERVICE_IP=$(kubectl get service $SERVICE_NAME -o jsonpath='{.status.loadBalancer.ingress[0].ip}')

  echo "✅ Deployment complete!"
  echo ""
  echo "Service IP: $SERVICE_IP"
  echo "Service URL: http://$SERVICE_IP"
  echo "Metrics: http://$SERVICE_IP:9090/metrics"
fi
