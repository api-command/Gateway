use hyper::{HeaderMap, StatusCode, Body};
use std::time::{Duration, Instant};
use serde::Serialize;

#[derive(Debug)]
pub struct ApiResponse {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub body: Body,
    pub latency: Duration,
    pub served_by: String,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub code: u16,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl ApiResponse {
    pub fn new(status: StatusCode) -> Self {
        Self {
            status,
            headers: HeaderMap::new(),
            body: Body::empty(),
            latency: Duration::default(),
            served_by: String::new(),
        }
    }

    pub fn with_body<T: Into<Body>>(mut self, body: T) -> Self {
        self.body = body.into();
        self
    }

    pub fn with_header<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<hyper::header::HeaderName>,
        V: Into<hyper::header::HeaderValue>,
    {
        self.headers.insert(key.into(), value.into());
        self
    }

    pub fn json<T: Serialize>(self, data: &T) -> Self {
        let body = match serde_json::to_vec(data) {
            Ok(b) => b,
            Err(_) => return self.status(StatusCode::INTERNAL_SERVER_ERROR)
        };
        
        self.with_body(body)
            .with_header("Content-Type", "application/json")
    }

    pub fn status(mut self, status: StatusCode) -> Self {
        self.status = status;
        self
    }
}

impl From<hyper::Response<Body>> for ApiResponse {
    fn from(res: hyper::Response<Body>) -> Self {
        let (parts, body) = res.into_parts();
        Self {
            status: parts.status,
            headers: parts.headers,
            body,
            latency: Duration::default(),
            served_by: String::new(),
        }
    }
}