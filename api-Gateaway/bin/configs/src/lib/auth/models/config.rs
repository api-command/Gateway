use serde::{Deserialize, Serialize};
use std::time::Duration;
use validator::Validate;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct GatewayConfig {
    #[validate]
    pub server: ServerConfig,
    #[validate]
    pub routing: RoutingConfig,
    #[validate]
    pub security: SecurityConfig,
    #[validate]
    pub observability: ObservabilityConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ServerConfig {
    #[validate(range(min = 1, max = 65535))]
    pub port: u16,
    pub workers: Option<usize>,
    pub max_connections: Option<u32>,
    pub timeout: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct RoutingConfig {
    #[validate]
    pub routes: Vec<RouteConfig>,
    pub default_backend: Option<String>,
    pub cache: CacheConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct RouteConfig {
    #[validate(length(min = 1))]
    pub path: String,
    #[validate(length(min = 1))]
    pub backend: String,
    #[serde(default)]
    pub methods: Vec<String>,
    #[validate]
    pub rate_limiting: Option<RateLimitConfig>,
    #[validate]
    pub authentication: AuthConfig,
    #[validate]
    pub circuit_breaker: Option<CircuitBreakerConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct RateLimitConfig {
    #[validate(range(min = 1))]
    pub requests: u32,
    #[validate(range(min = 1))]
    pub per_seconds: u32,
    pub algorithm: RateLimitAlgorithm,
    pub scope: RateLimitScope,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CacheConfig {
    pub enabled: bool,
    #[validate(range(min = 1))]
    pub ttl_seconds: u64,
    pub excluded_statuses: Vec<u16>,
    pub cache_key_strategy: CacheKeyStrategy,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct SecurityConfig {
    #[validate]
    pub cors: CorsConfig,
    #[validate]
    pub headers: SecurityHeaders,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct ObservabilityConfig {
    #[validate]
    pub logging: LoggingConfig,
    #[validate]
    pub metrics: MetricsConfig,
    #[validate]
    pub tracing: TracingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct AuthConfig {
    pub required: bool,
    pub jwt: Option<JwtConfig>,
    pub oauth: Option<OAuthConfig>,
}

// Enum definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RateLimitAlgorithm {
    TokenBucket,
    FixedWindow,
    SlidingWindow,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RateLimitScope {
    Global,
    PerIp,
    PerUser,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CacheKeyStrategy {
    FullUrl,
    PathOnly,
    Custom(String),
}

// Validation implementations
impl Validate for GatewayConfig {
    fn validate(&self) -> Result<(), validator::ValidationErrors> {
        self.server.validate()?;
        self.routing.validate()?;
        self.security.validate()?;
        self.observability.validate()
    }
}

// Additional config structs and validation implementations...