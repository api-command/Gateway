use hyper::{Body, Request, StatusCode};
use serde_json::json;
use wiremock::{MockServer, Mock, ResponseTemplate};
use crate::utils::test_helpers::{start_gateway, TestConfig};

mod utils;

#[tokio::test]
async fn test_authentication_flow() {
    // Setup mock authentication service
    let auth_mock = MockServer::start().await;
    Mock::given(wiremock::matchers::path("/.well-known/jwks.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "keys": [{"kid":"1","kty":"RSA","n":"...","e":"AQAB"}]
        }))
        .mount(&auth_mock)
        .await;

    // Configure test gateway
    let config = TestConfig {
        auth_jwks_url: auth_mock.uri(),
        routes: vec![
            json!({
                "path": "/secure",
                "backend": "http://backend-service",
                "authentication": {"required": true}
            })
        ],
        ..Default::default()
    };
    
    let gateway = start_gateway(config).await;

    // Test unauthenticated request
    let response = gateway.get("/secure").send().await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    // Test with valid token
    let valid_token = "valid.jwt.token";
    let response = gateway.get("/secure")
        .header("Authorization", format!("Bearer {}", valid_token))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_rate_limiting_integration() {
    let config = TestConfig {
        rate_limiting: json!({
            "default": {"requests": 2, "per_seconds": 60}
        }),
        ..Default::default()
    };
    
    let gateway = start_gateway(config).await;

    // First two requests should succeed
    for _ in 0..2 {
        let response = gateway.get("/api").send().await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    // Third request should be rate limited
    let response = gateway.get("/api").send().await.unwrap();
    assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
}

#[tokio::test]
async fn test_request_proxying() {
    let backend_mock = MockServer::start().await;
    Mock::given(wiremock::matchers::path("/data"))
        .respond_with(ResponseTemplate::new(200).set_body_string("OK"))
        .mount(&backend_mock)
        .await;

    let config = TestConfig {
        routes: vec![
            json!({
                "path": "/proxy",
                "backend": backend_mock.uri(),
                "rewrite": {"from": "^/proxy/(.*)", "to": "/$1"}
            })
        ],
        ..Default::default()
    };
    
    let gateway = start_gateway(config).await;
    let response = gateway.get("/proxy/data").send().await.unwrap();
    assert_eq!(response.text().await.unwrap(), "OK");
}

#[tokio::test]
async fn test_error_handling() {
    let config = TestConfig {
        routes: vec![
            json!({
                "path": "/unavailable",
                "backend": "http://invalid-backend",
                "circuit_breaker": {"failure_threshold": 3}
            })
        ],
        ..Default::default()
    };
    
    let gateway = start_gateway(config).await;

    // First 3 failures should go through
    for _ in 0..3 {
        let response = gateway.get("/unavailable").send().await.unwrap();
        assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
    }

    // Fourth request should trigger circuit breaker
    let response = gateway.get("/unavailable").send().await.unwrap();
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
}