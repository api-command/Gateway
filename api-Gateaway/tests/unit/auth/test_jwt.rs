use crate::auth::jwt::{JwtValidator, Claims};
use jsonwebtoken::{encode, EncodingKey, Header};
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};
use wiremock::{MockServer, Mock, ResponseTemplate};

const SECRET: &str = "test-secret-1234567890-1234567890-1234567890";

#[tokio::test]
async fn test_valid_jwt() {
    let claims = Claims {
        sub: "user-123".into(),
        exp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize + 3600,
        iat: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize,
        iss: "test-issuer".into(),
        aud: "test-audience".into(),
        scope: "api:read".into(),
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(SECRET.as_ref()),
    ).unwrap();

    let mock_server = MockServer::start().await;
    Mock::given(wiremock::matchers::path_prefix("/.well-known/jwks.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "keys": [{
                "kid": "1",
                "kty": "RSA",
                "n": "base64-modulus",
                "e": "AQAB",
                "alg": "HS256",
                "use": "sig"
            }]
        }))
        .mount(&mock_server)
        .await;

    let validator = JwtValidator::new(
        mock_server.uri().parse().unwrap(),
        "test-issuer".into(),
        "test-audience".into(),
    );

    let result = validator.validate_token(&token).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().sub, "user-123");
}

#[tokio::test]
async fn test_expired_jwt() {
    let claims = Claims {
        sub: "user-123".into(),
        exp: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize - 3600,
        iat: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize - 7200,
        iss: "test-issuer".into(),
        aud: "test-audience".into(),
        scope: "api:read".into(),
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(SECRET.as_ref()),
    ).unwrap();

    let validator = JwtValidator::new(
        "http://invalid-jwks".parse().unwrap(),
        "test-issuer".into(),
        "test-audience".into(),
    );

    let result = validator.validate_token(&token).await;
    assert!(matches!(result.err().unwrap(), crate::utils::error::ApiError::JwtError(_)));
}