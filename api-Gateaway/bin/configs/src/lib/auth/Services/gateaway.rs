use hyper::{Body, Request, Response, Server, StatusCode};
use std::{sync::Arc, time::Instant};
use crate::{models::{ApiRequest, ApiResponse}, routing::{matcher::RouteMatcher, proxy::ProxyHandler}, auth::{jwt::JwtValidator, oauth::OAuthIntrospector}, rate_limiting::redis_store::RedisRateLimiter};
use tokio::sync::Mutex;

pub struct GatewayService {
    router: Arc<RouteMatcher>,
    proxy: ProxyHandler,
    rate_limiter: Arc<RedisRateLimiter>,
    jwt_validator: Arc<JwtValidator>,
    oauth_introspector: Arc<OAuthIntrospector>,
}

impl GatewayService {
    pub fn new(
        router: RouteMatcher,
        rate_limiter: RedisRateLimiter,
        jwt_validator: JwtValidator,
        oauth_introspector: OAuthIntrospector,
    ) -> Self {
        Self {
            router: Arc::new(router),
            proxy: ProxyHandler::new(),
            rate_limiter: Arc::new(rate_limiter),
            jwt_validator: Arc::new(jwt_validator),
            oauth_introspector: Arc::new(oauth_introspector),
        }
    }

    pub async fn handle_request(&self, req: Request<Body>) -> ApiResponse {
        let start_time = Instant::now();
        let api_request = self.build_api_request(req).await;
        
        // Authentication
        if let Err(e) = self.authenticate(&api_request).await {
            return self.handle_error(e, start_time);
        }

        // Rate Limiting
        if let Err(e) = self.check_rate_limits(&api_request).await {
            return self.handle_error(e, start_time);
        }

        // Routing
        let (route, params) = match self.router.find_route(api_request.uri.path(), api_request.method.as_str()) {
            Ok(r) => r,
            Err(e) => return self.handle_error(e.into(), start_time),
        };

        // Proxying
        match self.proxy.forward_request(&route, api_request.uri.path(), params, api_request).await {
            Ok(res) => self.finalize_response(res, start_time),
            Err(e) => self.handle_error(e, start_time),
        }
    }

    async fn build_api_request(&self, req: Request<Body>) -> ApiRequest {
        let (parts, body) = req.into_parts();
        ApiRequest {
            method: parts.method,
            uri: parts.uri,
            headers: parts.headers,
            body: Body::empty(), // Actual body handling would go here
            remote_addr: None,
            received_at: Instant::now(),
            path_params: HashMap::new(),
            query_params: HashMap::new(),
        }
    }

    async fn authenticate(&self, req: &ApiRequest) -> Result<(), GatewayError> {
        if let Some(route) = self.router.find_route(req.uri.path(), req.method.as_str()).ok() {
            if route.authentication.required {
                let token = req.headers.get("Authorization")
                    .ok_or(GatewayError::Unauthorized)?;
                
                if route.authentication.jwt.is_some() {
                    self.jwt_validator.validate_token(token).await?;
                } else if route.authentication.oauth.is_some() {
                    self.oauth_introspector.validate_token(token).await?;
                }
            }
        }
        Ok(())
    }

    async fn check_rate_limits(&self, req: &ApiRequest) -> Result<(), GatewayError> {
        let client_id = req.client_ip().unwrap_or_default();
        let config = RateLimitConfig {
            capacity: 100,
            refill_amount: 10,
            refill_seconds: 60,
            burst: 20,
        };
        
        let (allowed, _) = self.rate_limiter
            .check_rate_limit(&format!("ip:{}", client_id), &config)
            .await?;
        
        if !allowed {
            return Err(GatewayError::RateLimitExceeded);
        }
        Ok(())
    }

    fn finalize_response(&self, mut response: ApiResponse, start_time: Instant) -> ApiResponse {
        response.latency = start_time.elapsed();
        response.headers.insert("X-Served-By", "api-gateway".parse().unwrap());
        response
    }

    fn handle_error(&self, error: GatewayError, start_time: Instant) -> ApiResponse {
        let status = match error {
            GatewayError::Unauthorized => StatusCode::UNAUTHORIZED,
            GatewayError::RateLimitExceeded => StatusCode::TOO_MANY_REQUESTS,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        
        ApiResponse::new(status)
            .json(&ErrorResponse::from(error))
            .with_latency(start_time.elapsed())
    }
}

#[derive(Debug)]
pub enum GatewayError {
    Unauthorized,
    RateLimitExceeded,
    RoutingError,
    BackendError,
}

// Implementations for error handling and conversions...