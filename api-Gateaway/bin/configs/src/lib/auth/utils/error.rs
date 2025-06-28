use thiserror::Error;
use hyper::{StatusCode, http};
use serde_json::Error as JsonError;
use validator::ValidationErrors;
use crate::models::response::ErrorResponse;

#[derive(Debug, Error)]
pub enum ApiError {
    // 4xx Errors
    #[error("Bad request: {0}")]
    BadRequest(String),
    
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    
    #[error("Forbidden: {0}")]
    Forbidden(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Method not allowed")]
    MethodNotAllowed,
    
    #[error("Request timeout")]
    RequestTimeout,
    
    #[error("Payload too large")]
    PayloadTooLarge,
    
    #[error("Too many requests")]
    TooManyRequests,

    // 5xx Errors
    #[error("Internal server error")]
    InternalServerError,
    
    #[error("Bad gateway")]
    BadGateway,
    
    #[error("Service unavailable")]
    ServiceUnavailable,
    
    #[error("Gateway timeout")]
    GatewayTimeout,

    // Custom Errors
    #[error("Validation error: {0}")]
    ValidationError(#[from] ValidationErrors),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] JsonError),
    
    #[error("External service error: {0}")]
    ExternalServiceError(String),
}

impl ApiError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ApiError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            ApiError::Forbidden(_) => StatusCode::FORBIDDEN,
            ApiError::NotFound(_) => StatusCode::NOT_FOUND,
            ApiError::MethodNotAllowed => StatusCode::METHOD_NOT_ALLOWED,
            ApiError::RequestTimeout => StatusCode::REQUEST_TIMEOUT,
            ApiError::PayloadTooLarge => StatusCode::PAYLOAD_TOO_LARGE,
            ApiError::TooManyRequests => StatusCode::TOO_MANY_REQUESTS,
            ApiError::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::BadGateway => StatusCode::BAD_GATEWAY,
            ApiError::ServiceUnavailable => StatusCode::SERVICE_UNAVAILABLE,
            ApiError::GatewayTimeout => StatusCode::GATEWAY_TIMEOUT,
            ApiError::ValidationError(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn to_response(&self) -> ErrorResponse {
        ErrorResponse {
            code: self.status_code().as_u16(),
            error: self.error_code(),
            message: self.to_string(),
            details: None,
        }
    }

    fn error_code(&self) -> String {
        match self {
            ApiError::BadRequest(_) => "bad_request".into(),
            ApiError::Unauthorized(_) => "unauthorized".into(),
            // ... other variants
            _ => "internal_error".into(),
        }
    }
}

impl From<http::Error> for ApiError {
    fn from(err: http::Error) -> Self {
        ApiError::InternalServerError.with_source(err)
    }
}

// Additional conversions and helper methods