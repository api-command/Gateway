#!/bin/bash
# API Gateway Environment Setup Script
set -euo pipefail

ENV=${1:-dev}
CONFIG_DIR="./configs/gateway/environments"
REDIS_VERSION="7.0.4"

echo -e "\033[1;36mSetting up ${ENV} environment...\033[0m"

check_dependency() {
    if ! command -v $1 &> /dev/null; then
        echo -e "\033[1;31mError: $1 is required but not installed.\033[0m"
        exit 1
    fi
}

setup_directories() {
    echo -e "\033[1;33mCreating data directories...\033[0m"
    mkdir -p ./data/{redis,logs}
    chmod -R 755 ./data
}

deploy_infrastructure() {
    echo -e "\033[1;34mDeploying supporting infrastructure...\033[0m"
    docker-compose -f deployments/docker/redis/docker-compose.yml up -d
    kubectl apply -f deployments/kubernetes/elk/ --namespace=logging
}

configure_environment() {
    echo -e "\033[1;35mConfiguring ${ENV} settings...\033[0m"
    cp "${CONFIG_DIR}/${ENV}.yaml" ./configs/gateway/active.yaml
    envsubst < ./configs/gateway/active.yaml > ./configs/gateway/runtime.yaml
}

main() {
    check_dependency docker
    check_dependency kubectl
    check_dependency envsubst
    
    setup_directories
    deploy_infrastructure
    configure_environment
    
    echo -e "\033[1;32mSetup completed successfully!\033[0m"
}

main "$@"