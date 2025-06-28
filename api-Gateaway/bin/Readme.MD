# API Gateway

[![CI Status](https://github.com/your-org/api-gateway/actions/workflows/test.yml/badge.svg)](https://github.com/your-org/api-gateway/actions)
[![Coverage](https://codecov.io/gh/your-org/api-gateway/branch/main/graph/badge.svg)](https://codecov.io/gh/your-org/api-gateway)

A high-performance API Gateway supporting HTTP/1.1, HTTP/2, and WebSockets, built with **Rust/Go**.

## Features

- JWT/OAuth2 Authentication
- Rate Limiting (Token Bucket Algorithm)
- gRPC Transcoding
- Prometheus Metrics
- Dynamic Routing

## Quick Start

```bash
# Clone and build
git clone https://github.com/your-org/api-gateway.git
cd api-gateway
make build

# Run (dev mode)
make run
```
