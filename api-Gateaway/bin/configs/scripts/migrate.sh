#!/bin/bash
# Configuration Migration Tool
set -euo pipefail

CONFIG_FILE=${1:-configs/gateway/routes.yaml}
ENV=${2:-dev}
NAMESPACE="api-gateway"

validate_config() {
    echo -e "\033[1;34mValidating new configuration...\033[0m"
    docker run --rm -v "$(pwd)/configs:/config" myregistry/api-gateway \
        gateway --validate --config /config/gateway/routes.yaml
}

update_kubernetes() {
    echo -e "\033[1;35mUpdating Kubernetes configuration...\033[0m"
    kubectl create configmap gateway-routes \
        --from-file=routes.yaml=${CONFIG_FILE} \
        --dry-run=client -o yaml | kubectl apply -n ${NAMESPACE} -f -
    
    kubectl rollout restart deployment/api-gateway -n ${NAMESPACE}
}

update_local() {
    echo -e "\033[1;36mUpdating local configuration...\033[0m"
    cp ${CONFIG_FILE} ./configs/gateway/active.yaml
    docker-compose kill -s HUP api-gateway
}

main() {
    validate_config
    
    if [[ ${ENV} == "local" ]]; then
        update_local
    else
        update_kubernetes
    fi

    echo -e "\033[1;32mConfiguration migration successful!\033[0m"
}

main "$@"