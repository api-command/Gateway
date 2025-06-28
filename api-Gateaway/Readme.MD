# 🚪 Gateway — High-Performance API Gateway

[![Build](https://github.com/MasterCaleb254/gateway/actions/workflows/test.yml/badge.svg)](https://github.com/MasterCaleb254/gateway/actions)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![Go Version](https://img.shields.io/badge/Go-1.20+-blue)](https://go.dev/)
[![K8s Ready](https://img.shields.io/badge/Kubernetes-ready-brightgreen.svg)](https://kubernetes.io/)

> A blazing-fast, modular API Gateway for modern microservices. Think Kong/Envoy/Apigee, but lean and open. Built for performance, security, and scalability.

---

## 🧰 Tech Stack

- **Backend**: Rust ⚙️ or Go 🧵
- **Proxy Layer**: NGINX / Envoy
- **Rate Limiting**: Redis + Token Bucket
- **Authentication**: JWT + OAuth2
- **Logging**: ELK Stack (Elasticsearch, Logstash, Kibana)
- **Orchestration**: Docker + Kubernetes
- **Infra-as-Code**: Terraform

---

## 🔥 Features

✅ Low-latency reverse proxy  
✅ JWT + OAuth2 secure authentication  
✅ Global and route-specific rate limiting  
✅ Circuit breaker and retry strategies  
✅ Configurable via YAML (hot reloadable)  
✅ Centralized logging with ELK  
✅ Docker & Kubernetes ready  
✅ Modular design (plug-in friendly)

---

## 📂 Project Structure

api-gateway/
├── configs/ # Gateway, NGINX, Logstash configs
├── src/ # Core services and routing/auth logic
├── tests/ # Unit, integration, and load tests
├── deployments/ # Docker, K8s, Terraform deployments
├── scripts/ # Shell scripts for setup & deployment
├── docs/ # Architecture, API specs, development
├── .github/ # CI/CD GitHub Actions workflows
├── Makefile # Build automation
├── Cargo.toml # Rust crate config (or go.mod)
└── README.md # You're here

yaml
Copy
Edit

---

## 🚀 Getting Started

```bash
# Clone the repo
git clone https://github.com/MasterCaleb254/gateway.git
cd gateway

# Build (Rust)
make build

# Run locally
make run

# Run tests
make test
Or via Docker:

bash
Copy
Edit
docker-compose up --build
🛠 Configuration
YAML-based configuration files live in:

bash
Copy
Edit
configs/gateway/
├── routes.yaml
├── rate_limits.yaml
└── environments/
    ├── dev.yaml
    ├── staging.yaml
    └── prod.yaml
Hot-reloading is supported for route and limit changes.

📡 API Endpoints
Full API specs are in docs/API_SPECS.md

/gateway/healthcheck — Liveness/readiness

/gateway/routes — Admin route control

/gateway/token/validate — JWT validator

/gateway/oauth/callback — OAuth2 exchange

📈 Observability
Integrated with Logstash and Kibana.

Exported metrics compatible with Prometheus.

Optional distributed tracing via OpenTelemetry.

🧪 Testing & Load
Unit tests in tests/unit/

Integration tests in tests/integration/

Load testing scenarios in tests/load/locustfile.py

🧳 Deploy
Dockerized via docker-compose

K8s manifests under deployments/kubernetes

Terraform scripts in deployments/terraform

🤝 Contributing
PRs are welcome! Please read docs/DEVELOPMENT.md

bash
Copy
Edit
# Fork + Clone + Branch
git checkout -b feature/my-awesome-change
📄 License
This project is licensed under the MIT License.