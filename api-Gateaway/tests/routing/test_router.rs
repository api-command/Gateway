use crate::routing::matcher::{RouteMatcher, Route};
use hyper::Method;

#[test]
fn test_route_matching() {
    let routes = vec![
        Route {
            path: "/users".into(),
            methods: vec!["GET".into()],
            backend: "http://user-service".into(),
            rewrite: None,
            prefix: false,
            regex: None,
        },
        Route {
            path: "/products".into(),
            methods: vec!["GET".into(), "POST".into()],
            backend: "http://product-service".into(),
            rewrite: None,
            prefix: true,
            regex: None,
        },
    ];

    let matcher = RouteMatcher::new(routes).unwrap();
    
    // Exact match
    let (route, _) = matcher.find_route("/users", "GET").unwrap();
    assert_eq!(route.backend, "http://user-service");
    
    // Prefix match
    let (route, params) = matcher.find_route("/products/123", "GET").unwrap();
    assert_eq!(route.backend, "http://product-service");
    assert_eq!(params.get("prefix_remainder").unwrap(), "/123");
    
    // Method not allowed
    assert!(matcher.find_route("/users", "POST").is_err());
}

#[tokio::test]
async fn test_proxy_handler() {
    use crate::routing::proxy::ProxyHandler;
    use hyper::{Request, Body};
    use wiremock::{MockServer, Mock, ResponseTemplate};

    let mock_server = MockServer::start().await;
    Mock::given(wiremock::matchers::method("GET"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&mock_server)
        .await;

    let handler = ProxyHandler::new();
    let route = Route {
        backend: mock_server.uri(),
        ..Default::default()
    };

    let request = Request::builder()
        .method("GET")
        .uri("/test")
        .body(Body::empty())
        .unwrap();

    let response = handler.forward_request(&route, "/test", Default::default(), request).await.unwrap();
    assert_eq!(response.status(), 200);
}