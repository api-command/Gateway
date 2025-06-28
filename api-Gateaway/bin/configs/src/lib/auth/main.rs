// src/main.rs
use std::{net::SocketAddr, sync::Arc, time::Duration};
use hyper::{Body, Request, Response, Server, StatusCode};
use hyper::service::{make_service_fn, service_fn};
use tokio::signal;
use crate::{
    config::GatewayConfig,
    logging::elk::ElkLogger,
    models::{ApiRequest, ApiResponse},
    routing::{matcher::RouteMatcher, proxy::ProxyHandler},
    services::{gateway::GatewayService, healthcheck::HealthCheckService, cache::CacheService},
    utils::error::ApiError,
    rate_limiting::redis_store::RedisRateLimiter,
    auth::{jwt::JwtValidator, oauth::OAuthIntrospector},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Load configuration
    let config = load_config().await?;
    
    // Initialize logging
    ElkLogger::new(
        &config.observability.logging.logstash_url,
        &config.server.name,
        &config.server.env
    )?.init_logging()?;

    // Initialize Redis rate limiter
    let rate_limiter = RedisRateLimiter::new(&config.rate_limiting.redis_url)
        .await
        .map_err(|e| ApiError::ConfigError(format!("Redis connection failed: {}", e)))?;

    // Build route matcher
    let route_matcher = RouteMatcher::new(config.routing.routes)
        .map_err(|e| ApiError::ConfigError(format!("Invalid route configuration: {}", e)))?;

    // Initialize auth components
    let jwt_validator = JwtValidator::new(
        config.auth.jwk_url.parse()?,
        config.auth.issuer.clone(),
        config.auth.audience.clone(),
    );
    
    let oauth_introspector = OAuthIntrospector::new(
        config.auth.introspection_url.parse()?,
        config.auth.client_id.clone(),
        config.auth.client_secret.clone(),
    );

    // Create services
    let gateway = Arc::new(GatewayService::new(
        route_matcher,
        rate_limiter,
        jwt_validator,
        oauth_introspector,
    ));

    let health_check = Arc::new(HealthCheckService::new());
    let cache_service = Arc::new(CacheService::new(
        config.caching.memory_size,
        Duration::from_secs(config.caching.ttl),
        config.redis_url.clone(),
    ));

    // Configure server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));
    let make_svc = make_service_fn(move |_conn| {
        let gateway = gateway.clone();
        let health_check = health_check.clone();
        let cache_service = cache_service.clone();

        async move {
            Ok::<_, ApiError>(service_fn(move |req: Request<Body>| {
                let gateway = gateway.clone();
                let health_check = health_check.clone();
                let cache_service = cache_service.clone();

                async move {
                    // Handle health checks separately
                    if req.uri().path() == "/health" {
                        return Ok(health_check.health_endpoint());
                    }

                    // Check cache first
                    let cache_key = cache_service.generate_cache_key(&ApiRequest::from(req));
                    if let Some(cached) = cache_service.get(&cache_key).await {
                        return Ok(Response::new(cached.into()));
                    }

                    // Process request
                    let start_time = Instant::now();
                    health_check.increment_requests();
                    
                    let result = gateway.handle_request(req).await;
                    
                    // Cache successful responses
                    if result.status.is_success() {
                        if let Some(ttl) = result.cache_control {
                            cache_service.set(&cache_key, result.body.clone(), ttl).await;
                        }
                    }

                    // Record latency
                    let latency = start_time.elapsed();
                    metrics::histogram!("request_latency", latency);

                    if !result.status.is_success() {
                        health_check.increment_errors();
                    }

                    Ok(result)
                }
            }))
        }
    });

    // Start server
    let server = Server::bind(&addr)
        .serve(make_svc)
        .with_graceful_shutdown(shutdown_signal());

    log::info!("Gateway running on {}", addr);
    server.await?;

    Ok(())
}

async fn load_config() -> Result<GatewayConfig, ApiError> {
    let mut settings = config::Config::default();
    settings
        .merge(config::File::with_name("config/gateway"))
        .map_err(|e| ApiError::ConfigError(e.to_string()))?
        .merge(config::Environment::with_prefix("GATEWAY"))
        .map_err(|e| ApiError::ConfigError(e.to_string()))?;

    settings.try_into()
        .map_err(|e| ApiError::ConfigError(e.to_string()))
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    log::info!("Shutting down gracefully...");
}

// Integration tests
#[cfg(test)]
mod tests {
    use super::*;
    use hyper::{Body, Request};
    use wiremock::{MockServer, Mock, ResponseTemplate};

    #[tokio::test]
    async fn test_health_endpoint() {
        let health = HealthCheckService::new();
        let response = health.health_endpoint();
        assert_eq!(response.status, StatusCode::OK);
    }

    #[tokio::test]
    async fn test_request_flow() {
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let config = GatewayConfig::test_config(mock_server.uri());
        let gateway = setup_gateway(config).await;

        let request = Request::builder()
            .uri("/test")
            .body(Body::empty())
            .unwrap();

        let response = gateway.handle_request(request).await;
        assert_eq!(response.status, StatusCode::OK);
    }
}