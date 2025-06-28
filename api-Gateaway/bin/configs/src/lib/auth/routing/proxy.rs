use reqwest::{Client, Response, StatusCode};
use std::time::Duration;
use thiserror::Error;
use url::Url;
use crate::routing::matcher::Route;

#[derive(Debug, Error)]
pub enum ProxyError {
    #[error("Backend error: {0}")]
    BackendError(#[from] reqwest::Error),
    #[error("Circuit breaker triggered")]
    CircuitBreaker,
    #[error("Invalid URL: {0}")]
    InvalidUrl(#[from] url::ParseError),
    #[error("Timeout reached")]
    Timeout,
}

pub struct ProxyHandler {
    client: Client,
    circuit_breaker: CircuitBreaker,
}

struct CircuitBreaker {
    failure_count: std::sync::atomic::AtomicU32,
    last_failure: std::sync::Mutex<std::time::Instant>,
    cooldown: Duration,
    max_failures: u32,
}

impl ProxyHandler {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(5))
                .build()
                .unwrap(),
            circuit_breaker: CircuitBreaker {
                failure_count: std::sync::atomic::AtomicU32::new(0),
                last_failure: std::sync::Mutex::new(std::time::Instant::now()),
                cooldown: Duration::from_secs(30),
                max_failures: 5,
            },
        }
    }

    pub async fn forward_request(
        &self,
        route: &Route,
        path: &str,
        params: HashMap<String, String>,
        mut request: http::Request<Vec<u8>>,
    ) -> Result<Response, ProxyError> {
        if self.circuit_breaker.is_tripped() {
            return Err(ProxyError::CircuitBreaker);
        }

        let target_url = self.build_target_url(route, path, params)?;
        let method = request.method().clone();
        let headers = request.headers().clone();

        let response = self.client
            .request(method, target_url)
            .headers(headers)
            .body(request.body().clone())
            .send()
            .await?;

        self.handle_circuit_breaker(&response).await;
        Ok(response)
    }

    fn build_target_url(
        &self,
        route: &Route,
        path: &str,
        params: HashMap<String, String>,
    ) -> Result<Url, ProxyError> {
        let mut url = Url::parse(&route.backend)?;
        
        if let Some(rewrite) = &route.rewrite {
            let rewritten = rewrite.from.replace_all(path, &rewrite.to);
            url.set_path(&rewritten);
        } else if route.prefix {
            if let Some(remainder) = params.get("prefix_remainder") {
                url.set_path(&format!("{}{}", route.path, remainder));
            }
        } else {
            url.set_path(path);
        }

        Ok(url)
    }

    async fn handle_circuit_breaker(&self, response: &Response) {
        if response.status().is_server_error() {
            let failures = self.circuit_breaker.failure_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            let mut last_failure = self.circuit_breaker.last_failure.lock().unwrap();
            *last_failure = std::time::Instant::now();

            if failures >= self.circuit_breaker.max_failures {
                log::error!("Circuit breaker triggered!");
            }
        } else if response.status().is_success() {
            self.circuit_breaker.failure_count.store(0, std::sync::atomic::Ordering::SeqCst);
        }
    }
}

impl CircuitBreaker {
    fn is_tripped(&self) -> bool {
        let failures = self.failure_count.load(std::sync::atomic::Ordering::Relaxed);
        let last_failure = self.last_failure.lock().unwrap();
        
        failures >= self.max_failures && 
            last_failure.elapsed() < self.cooldown
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use wiremock::matchers::{method, path};

    #[tokio::test]
    async fn test_proxy_forward() {
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&mock_server)
            .await;

        let handler = ProxyHandler::new();
        let route = Route {
            backend: mock_server.uri(),
            ..test_route()
        };

        let request = http::Request::builder()
            .method("GET")
            .uri("/test")
            .body(Vec::new())
            .unwrap();

        let response = handler.forward_request(&route, "/test", HashMap::new(), request).await.unwrap();
        assert_eq!(response.status(), 200);
    }
}