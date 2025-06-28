use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use serde::Serialize;
use crate::models::ApiResponse;

#[derive(Serialize)]
pub struct HealthStatus {
    status: &'static str,
    uptime: String,
    checks: HashMap<&'static str, CheckResult>,
}

#[derive(Serialize)]
pub struct CheckResult {
    status: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<String>,
}

pub struct HealthCheckService {
    start_time: Instant,
    request_count: AtomicU64,
    error_count: AtomicU64,
}

impl HealthCheckService {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            request_count: AtomicU64::new(0),
            error_count: AtomicU64::new(0),
        }
    }

    pub fn increment_requests(&self) {
        self.request_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_errors(&self) {
        self.error_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn get_status(&self) -> HealthStatus {
        let mut checks = HashMap::new();
        
        // Basic self check
        checks.insert("self", CheckResult {
            status: "OK",
            details: None,
        });

        // Add more checks here (database, redis, etc.)
        
        HealthStatus {
            status: "OK",
            uptime: format!("{:?}", self.start_time.elapsed()),
            checks,
        }
    }

    pub fn health_endpoint(&self) -> ApiResponse {
        ApiResponse::new(StatusCode::OK)
            .json(&self.get_status())
    }

    pub fn metrics_endpoint(&self) -> ApiResponse {
        let metrics = format!(
            "gateway_requests_total {}\ngateway_errors_total {}",
            self.request_count.load(Ordering::Relaxed),
            self.error_count.load(Ordering::Relaxed)
        );
        
        ApiResponse::new(StatusCode::OK)
            .with_header("Content-Type", "text/plain")
            .with_body(metrics)
    }
}use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use serde::Serialize;
use crate::models::ApiResponse;

#[derive(Serialize)]
pub struct HealthStatus {
    status: &'static str,
    uptime: String,
    checks: HashMap<&'static str, CheckResult>,
}

#[derive(Serialize)]
pub struct CheckResult {
    status: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<String>,
}

pub struct HealthCheckService {
    start_time: Instant,
    request_count: AtomicU64,
    error_count: AtomicU64,
}

impl HealthCheckService {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            request_count: AtomicU64::new(0),
            error_count: AtomicU64::new(0),
        }
    }

    pub fn increment_requests(&self) {
        self.request_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_errors(&self) {
        self.error_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn get_status(&self) -> HealthStatus {
        let mut checks = HashMap::new();
        
        // Basic self check
        checks.insert("self", CheckResult {
            status: "OK",
            details: None,
        });

        // Add more checks here (database, redis, etc.)
        
        HealthStatus {
            status: "OK",
            uptime: format!("{:?}", self.start_time.elapsed()),
            checks,
        }
    }

    pub fn health_endpoint(&self) -> ApiResponse {
        ApiResponse::new(StatusCode::OK)
            .json(&self.get_status())
    }

    pub fn metrics_endpoint(&self) -> ApiResponse {
        let metrics = format!(
            "gateway_requests_total {}\ngateway_errors_total {}",
            self.request_count.load(Ordering::Relaxed),
            self.error_count.load(Ordering::Relaxed)
        );
        
        ApiResponse::new(StatusCode::OK)
            .with_header("Content-Type", "text/plain")
            .with_body(metrics)
    }
}