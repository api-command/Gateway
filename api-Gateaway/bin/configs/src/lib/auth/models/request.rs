use hyper::{HeaderMap, Method, Uri, Body};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::Instant;
use jsonwebtoken::errors::Error as JwtError;
use jsonwebtoken::{decode, DecodingKey, Validation};
use crate::auth::jwt::Claims;

#[derive(Debug, Clone)]
pub struct ApiRequest {
    pub method: Method,
    pub uri: Uri,
    pub headers: HeaderMap,
    pub body: Body,
    pub remote_addr: Option<SocketAddr>,
    pub received_at: Instant,
    pub path_params: HashMap<String, String>,
    pub query_params: HashMap<String, String>,
}

impl ApiRequest {
    pub fn jwt_claims(&self, secret: &str) -> Result<Claims, JwtError> {
        let token = self.headers.get("Authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "))
            .unwrap_or("");
        
        let key = DecodingKey::from_secret(secret.as_ref());
        decode::<Claims>(token, &key, &Validation::default()).map(|data| data.claims)
    }

    pub fn client_ip(&self) -> Option<String> {
        self.headers.get("X-Forwarded-For")
            .or_else(|| self.headers.get("X-Real-IP"))
            .and_then(|h| h.to_str().ok())
            .map(|s| s.split(',').next().unwrap_or("").trim().to_string())
            .or_else(|| self.remote_addr.map(|a| a.ip().to_string()))
    }

    pub fn content_type(&self) -> Option<String> {
        self.headers.get("Content-Type")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestInfo {
    pub method: String,
    pub path: String,
    pub query: HashMap<String, String>,
    pub headers: HashMap<String, String>,
    pub client_ip: Option<String>,
    pub timestamp: u64,
}