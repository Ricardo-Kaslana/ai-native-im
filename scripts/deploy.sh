#!/bin/bash

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
ENVIRONMENT=${1:-production}
NAMESPACE="ai-im"

echo -e "${GREEN}🚀 Deploying AI-IM to ${ENVIRONMENT}${NC}"
echo ""

# Check prerequisites
echo "📋 Checking prerequisites..."

if ! command -v kubectl &> /dev/null; then
    echo -e "${RED}❌ kubectl not found. Please install kubectl.${NC}"
    exit 1
fi

if ! command -v docker &> /dev/null; then
    echo -e "${RED}❌ Docker not found. Please install Docker.${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Prerequisites check passed${NC}"
echo ""

# Build Docker image
echo "🔨 Building Docker image..."
docker build -t ai-im-server:latest -f server/Dockerfile .

if [ $? -ne 0 ]; then
    echo -e "${RED}❌ Docker build failed${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Docker image built successfully${NC}"
echo ""

# Tag and push image (if registry is configured)
if [ ! -z "$DOCKER_REGISTRY" ]; then
    echo "📤 Pushing image to registry..."
    docker tag ai-im-server:latest $DOCKER_REGISTRY/ai-im-server:latest
    docker tag ai-im-server:latest $DOCKER_REGISTRY/ai-im-server:$(git rev-parse --short HEAD)
    docker push $DOCKER_REGISTRY/ai-im-server:latest
    docker push $DOCKER_REGISTRY/ai-im-server:$(git rev-parse --short HEAD)
    echo -e "${GREEN}✅ Image pushed to registry${NC}"
    echo ""
fi

# Create namespace if it doesn't exist
echo "📦 Creating namespace..."
kubectl create namespace $NAMESPACE --dry-run=client -o yaml | kubectl apply -f -

# Apply Kubernetes configurations
echo "⚙️  Applying Kubernetes configurations..."

kubectl apply -f k8s/namespace.yaml
kubectl apply -f k8s/configmap.yaml

# Apply secrets (if they exist)
if [ -f k8s/secrets.yaml ]; then
    kubectl apply -f k8s/secrets.yaml
else
    echo -e "${YELLOW}⚠️  No secrets.yaml found. Make sure to create secrets manually.${NC}"
fi

kubectl apply -f k8s/deployment.yaml
kubectl apply -f k8s/service.yaml
kubectl apply -f k8s/ingress.yaml
kubectl apply -f k8s/hpa.yaml

echo -e "${GREEN}✅ Configurations applied${NC}"
echo ""

# Wait for rollout
echo "⏳ Waiting for deployment to complete..."
kubectl rollout status deployment/ai-im-server -n $NAMESPACE --timeout=5m

if [ $? -ne 0 ]; then
    echo -e "${RED}❌ Deployment failed${NC}"
    echo ""
    echo "Recent pod events:"
    kubectl get events -n $NAMESPACE --sort-by='.lastTimestamp' | tail -20
    exit 1
fi

echo -e "${GREEN}✅ Deployment completed successfully${NC}"
echo ""

# Show deployment status
echo "📊 Deployment Status:"
echo ""
kubectl get pods -n $NAMESPACE
echo ""
kubectl get svc -n $NAMESPACE
echo ""
kubectl get ingress -n $NAMESPACE

echo ""
echo -e "${GREEN}🎉 Deployment completed!${NC}"
echo ""
echo "Next steps:"
echo "  - Monitor logs: kubectl logs -f deployment/ai-im-server -n $NAMESPACE"
echo "  - Check status: kubectl get pods -n $NAMESPACE"
echo "  - Scale: kubectl scale deployment/ai-im-server --replicas=5 -n $NAMESPACE"
