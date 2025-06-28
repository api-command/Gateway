# API Gateway Development Guide

## Prerequisites

- **Go 1.21+** (or **Node.js 20+**/Python 3.11+ depending on stack)
- **Docker** 24.0+ and **Docker Compose** v2.23+
- **Make** (for build automation)
- **Postman** or **curl** for API testing

## Local Setup

```bash
# Clone repository
git clone https://github.com/your-org/api-gateway.git
cd api-gateway

# Install dependencies
make install

# Generate TLS certificates (mTLS example)
openssl req -x509 -newkey rsa:4096 -nodes -keyout key.pem -out cert.pem -days 365
```
