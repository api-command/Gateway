use hyper::{Body, Request};
use wiremock::{MockServer, Mock, ResponseTemplate};
use crate::utils::test_helpers::{start_gateway, TestConfig};

mod utils;

#[tokio::test]
async fn test_round_robin_load_balancing() {
    // Create 3 mock backend servers
    let backends = vec![
        MockServer::start().await,
        MockServer::start().await,
        MockServer::start().await,
    ];

    // Setup response counters
    let counters = vec![std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0)); 3];

    // Configure mock responses
    for (i, backend) in backends.iter().enumerate() {
        let counter = counters[i].clone();
        Mock::given(wiremock::matchers::any())
            .respond_with(ResponseTemplate::new(200).set_body_string(i.to_string()))
            .expect(3..)
            .mount_with_async_assert(backend, move |_, _| {
                counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                async {}
            })
            .await;
    }

    // Configure gateway with load balancing
    let config = TestConfig {
        routes: vec![
            json!({
                "path": "/lb",
                "backend": [
                    backends[0].uri(),
                    backends[1].uri(),
                    backends[2].uri()
                ],
                "load_balancing": "round_robin"
            })
        ],
        ..Default::default()
    };
    
    let gateway = start_gateway(config).await;

    // Send 9 requests to test distribution
    for _ in 0..9 {
        let response = gateway.get("/lb").send().await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    // Verify even distribution
    for counter in counters {
        assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 3);
    }
}

#[tokio::test]
async fn test_health_checks() {
    let healthy_backend = MockServer::start().await;
    let unhealthy_backend = MockServer::start().await;

    // Configure health checks
    Mock::given(wiremock::matchers::path("/health"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&healthy_backend)
        .await;

    Mock::given(wiremock::matchers::path("/health"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&unhealthy_backend)
        .await;

    let config = TestConfig {
        routes: vec![
            json!({
                "path": "/service",
                "backend": [
                    healthy_backend.uri(),
                    unhealthy_backend.uri()
                ],
                "health_check": {"path": "/health", "interval": 1}
            })
        ],
        ..Default::default()
    };
    
    let gateway = start_gateway(config).await;

    // Send multiple requests
    for _ in 0..5 {
        let response = gateway.get("/service").send().await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    // Verify only healthy backend received traffic
    let healthy_count = healthy_backend.received_requests().await.unwrap().len();
    let unhealthy_count = unhealthy_backend.received_requests().await.unwrap().len();
    
    assert_eq!(healthy_count, 5);
    assert_eq!(unhealthy_count, 0);
}

#[tokio::test]
async fn test_least_connections_strategy() {
    let fast_backend = MockServer::start().await;
    let slow_backend = MockServer::start().await;

    Mock::given(wiremock::matchers::any())
        .respond_with(ResponseTemplate::new(200).set_delay(std::time::Duration::from_secs(0)))
        .mount(&fast_backend)
        .await;

    Mock::given(wiremock::matchers::any())
        .respond_with(ResponseTemplate::new(200).set_delay(std::time::Duration::from_secs(2)))
        .mount(&slow_backend)
        .await;

    let config = TestConfig {
        routes: vec![
            json!({
                "path": "/lc",
                "backend": [
                    fast_backend.uri(),
                    slow_backend.uri()
                ],
                "load_balancing": "least_connections"
            })
        ],
        ..Default::default()
    };
    
    let gateway = start_gateway(config).await;

    // Send concurrent requests
    let handles: Vec<_> = (0..10).map(|_| {
        let gateway = gateway.clone();
        tokio::spawn(async move {
            gateway.get("/lc").send().await.unwrap()
        })
    }).collect();

    futures::future::join_all(handles).await;

    // Verify more requests went to fast backend
    let fast_count = fast_backend.received_requests().await.unwrap().len();
    let slow_count = slow_backend.received_requests().await.unwrap().len();
    
    assert!(fast_count > slow_count);
    assert_eq!(fast_count + slow_count, 10);
}