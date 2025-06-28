use validator::{Validate, ValidationError};
use regex::Regex;
use crate::models::request::ApiRequest;

lazy_static! {
    static ref JWT_REGEX: Regex = Regex::new(r"^Bearer [A-Za-z0-9-_]+\.[A-Za-z0-9-_]+\.[A-Za-z0-9-_]*$").unwrap();
    static ref PATH_REGEX: Regex = Regex::new(r"^/[a-zA-Z0-9\-_/.]+$").unwrap();
}

pub fn validate_request(request: &ApiRequest) -> Result<(), ValidationErrors> {
    // Validate headers
    if let Some(content_type) = request.content_type() {
        if !content_type.starts_with("application/json") {
            return Err(ValidationError::new("invalid_content_type"));
        }
    }

    // Validate JWT format if present
    if let Some(auth_header) = request.headers.get("Authorization") {
        if !JWT_REGEX.is_match(auth_header.to_str().unwrap_or_default()) {
            return Err(ValidationError::new("invalid_auth_header_format"));
        }
    }

    // Validate path format
    if !PATH_REGEX.is_match(request.uri.path()) {
        return Err(ValidationError::new("invalid_path_format"));
    }

    Ok(())
}

pub fn validate_config<T: Validate>(config: &T) -> Result<(), ValidationErrors> {
    config.validate()?;
    
    // Custom validation logic
    if let Some(rate_limit) = &config.rate_limiting {
        if rate_limit.requests == 0 || rate_limit.per_seconds == 0 {
            return Err(ValidationError::new("invalid_rate_limit_config"));
        }
    }
    
    Ok(())
}

pub fn validate_http_method(method: &str) -> Result<(), ValidationError> {
    let allowed = ["GET", "POST", "PUT", "DELETE", "PATCH"];
    if !allowed.contains(&method) {
        return Err(ValidationError::new("invalid_http_method"));
    }
    Ok(())
}

pub fn validate_endpoint_path(path: &str) -> Result<(), ValidationError> {
    if path.is_empty() || !path.starts_with('/') {
        return Err(ValidationError::new("invalid_endpoint_path"));
    }
    
    if path.contains("//") || path.ends_with('/') {
        return Err(ValidationError::new("invalid_endpoint_path"));
    }
    
    Ok(())
}

// Custom validation functions
pub fn validate_ip_address(ip: &str) -> Result<(), ValidationError> {
    if !ip.parse::<std::net::IpAddr>().is_ok() {
        return Err(ValidationError::new("invalid_ip_address"));
    }
    Ok(())
}

pub fn validate_rate_limit_window(window: u64) -> Result<(), ValidationError> {
    if window < 1 || window > 3600 {
        return Err(ValidationError::new("invalid_rate_limit_window"));
    }
    Ok(())
}