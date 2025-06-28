# Build stage
FROM rust:1.65-slim-buster as builder

WORKDIR /app
COPY . .

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

RUN cargo build --release

# Runtime stage
FROM debian:buster-slim

RUN apt-get update && apt-get install -y \
    libssl1.1 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy built binary and configs
COPY --from=builder /app/target/release/api-gateway /usr/local/bin/
COPY configs/ /app/config/

# Health check
HEALTHCHECK --interval=30s --timeout=3s \
    CMD curl -f http://localhost:8080/health || exit 1

# Non-root user
RUN useradd -m gateway
USER gateway

EXPOSE 8080
ENTRYPOINT ["api-gateway"]