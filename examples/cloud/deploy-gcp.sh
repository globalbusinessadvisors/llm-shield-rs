#!/bin/bash
# Deploy LLM Shield API to GCP
#
# Prerequisites:
# - gcloud CLI configured
# - Docker installed
# - GCR/Artifact Registry configured
# - GKE cluster created OR Cloud Run configured

set -e

# Configuration
GCP_PROJECT="${GCP_PROJECT:-llm-shield-prod}"
GCP_REGION="${GCP_REGION:-us-central1}"
SERVICE_NAME="llm-shield-api"
IMAGE_TAG="${IMAGE_TAG:-latest}"
DEPLOY_TARGET="${DEPLOY_TARGET:-cloud-run}"  # Options: cloud-run, gke

echo "🚀 Deploying LLM Shield API to GCP"
echo "   Project: $GCP_PROJECT"
echo "   Region: $GCP_REGION"
echo "   Target: $DEPLOY_TARGET"
echo ""

# Step 1: Build Docker image
echo "📦 Building Docker image..."
docker build \
  --build-arg FEATURES=cloud-gcp \
  -t gcr.io/$GCP_PROJECT/$SERVICE_NAME:$IMAGE_TAG \
  -f examples/cloud/Dockerfile \
  ../..

# Step 2: Push to Container Registry
echo "📤 Pushing to GCR..."
gcloud auth configure-docker
docker push gcr.io/$GCP_PROJECT/$SERVICE_NAME:$IMAGE_TAG

if [ "$DEPLOY_TARGET" == "cloud-run" ]; then
  # Deploy to Cloud Run
  echo "☁️  Deploying to Cloud Run..."

  gcloud run deploy $SERVICE_NAME \
    --image gcr.io/$GCP_PROJECT/$SERVICE_NAME:$IMAGE_TAG \
    --platform managed \
    --region $GCP_REGION \
    --project $GCP_PROJECT \
    --allow-unauthenticated \
    --port 8080 \
    --cpu 2 \
    --memory 2Gi \
    --min-instances 1 \
    --max-instances 10 \
    --timeout 60 \
    --service-account llm-shield-api@$GCP_PROJECT.iam.gserviceaccount.com \
    --set-env-vars "GCP_PROJECT=$GCP_PROJECT,RUST_LOG=info" \
    --labels app=llm-shield,environment=production

  # Get service URL
  SERVICE_URL=$(gcloud run services describe $SERVICE_NAME \
    --platform managed \
    --region $GCP_REGION \
    --project $GCP_PROJECT \
    --format 'value(status.url)')

  echo "✅ Deployment complete!"
  echo ""
  echo "Service URL: $SERVICE_URL"
  echo "Health check: $SERVICE_URL/health"

elif [ "$DEPLOY_TARGET" == "gke" ]; then
  # Deploy to GKE
  echo "☸️  Deploying to GKE..."

  # Apply Kubernetes manifests
  kubectl apply -f - <<EOF
apiVersion: apps/v1
kind: Deployment
metadata:
  name: $SERVICE_NAME
  labels:
    app: llm-shield
spec:
  replicas: 3
  selector:
    matchLabels:
      app: llm-shield
  template:
    metadata:
      labels:
        app: llm-shield
    spec:
      serviceAccountName: llm-shield-api
      containers:
      - name: api
        image: gcr.io/$GCP_PROJECT/$SERVICE_NAME:$IMAGE_TAG
        ports:
        - containerPort: 8080
          name: http
        - containerPort: 9090
          name: metrics
        env:
        - name: GCP_PROJECT
          value: "$GCP_PROJECT"
        - name: RUST_LOG
          value: "info"
        resources:
          requests:
            memory: "1Gi"
            cpu: "500m"
          limits:
            memory: "2Gi"
            cpu: "1000m"
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
