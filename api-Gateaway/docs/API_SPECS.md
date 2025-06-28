# API Gateway Specifications

## Core Features

- Protocol Support: HTTP/1.1, HTTP/2, WebSocket, gRPC
- Authentication: JWT, OAuth2, API Keys
- Rate Limiting: Token bucket algorithm
- Observability: Prometheus metrics, OpenTelemetry traces

## Route Configuration Schema (YAML)

```yaml
routes:
  - path: /api/v1/users/*
    methods: [GET, POST, PUT]
    upstream: user-service:8080
    plugins:
      - name: jwt-auth
        config:
          secret: $JWT_SECRET
      - name: rate-limit
        config:
          requests_per_minute: 100
          burst: 20

  - path: /api/v1/products/{id}
    methods: [GET]
    upstream: product-service:8081
    timeout: 5s
```
