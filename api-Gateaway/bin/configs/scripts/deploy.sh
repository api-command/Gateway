#!/bin/bash
# API Gateway Deployment Automation
set -euo pipefail

ENV=${1:-dev}
IMAGE_TAG=${2:-latest}
NAMESPACE="api-gateway"
TIMEOUT=300  # 5 minutes

deploy_services() {
    echo -e "\033[1;34mDeploying API Gateway (${IMAGE_TAG}) to ${ENV}...\033[0m"
    
    # Build and push Docker image
    docker build -t "myregistry/api-gateway:${IMAGE_TAG}" -f deployments/docker/gateway/Dockerfile .
    docker push "myregistry/api-gateway:${IMAGE_TAG}"

    # Kubernetes deployment
    kubectl config use-context "${ENV}-cluster"
    kubectl apply -f deployments/kubernetes/gateway/ -n ${NAMESPACE}
    
    # Wait for rollout
    kubectl rollout status deployment/api-gateway -n ${NAMESPACE} --timeout=${TIMEOUT}s
    
    # Post-deployment checks
    echo -e "\033[1;36mRunning smoke tests...\033[0m"
    ./scripts/test.sh smoke ${ENV}
}

main() {
    echo -e "\033[1;33mStarting deployment to ${ENV}\033[0m"
    deploy_services
    echo -e "\033[1;32mDeployment to ${ENV} successful!\033[0m"
}

main "$@"