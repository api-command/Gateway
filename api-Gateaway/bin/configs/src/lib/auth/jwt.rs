use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation, errors::Error as JwtError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use reqwest::Url;
use async_trait::async_trait;
use moka::future::Cache;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
    pub iss: String,
    pub aud: String,
    pub scope: String,
}

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Invalid token")]
    InvalidToken,
    #[error("Token expired")]
    TokenExpired,
    #[error("Invalid issuer")]
    InvalidIssuer,
    #[error("JWT error: {0}")]
    JwtError(#[from] JwtError),
    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),
}

#[async_trait]
pub trait Authenticator {
    async fn validate_token(&self, token: &str) -> Result<Claims, AuthError>;
}

pub struct JwtValidator {
    jwks_url: Url,
    issuer: String,
    audience: String,
    cache: Cache<String, DecodingKey>,
}

impl JwtValidator {
    pub fn new(jwks_url: Url, issuer: String, audience: String) -> Self {
        Self {
            jwks_url,
            issuer,
            audience,
            cache: Cache::builder()
                .time_to_live(std::time::Duration::from_secs(3600))
                .build(),
        }
    }

    async fn get_jwks(&self) -> Result<HashMap<String, DecodingKey>, AuthError> {
        let res = reqwest::get(self.jwks_url.clone()).await?;
        let jwks: Jwks = res.json().await?;
        
        let mut keys = HashMap::new();
        for key in jwks.keys {
            if let Some(kid) = key.common.key_id {
                keys.insert(kid, DecodingKey::from_rsa_components(&key.n, &key.e)?);
            }
        }
        Ok(keys)
    }

    async fn get_key(&self, kid: &str) -> Result<DecodingKey, AuthError> {
        if let Some(key) = self.cache.get(kid) {
            return Ok(key);
        }
        
        let keys = self.get_jwks().await?;
        if let Some(key) = keys.get(kid) {
            self.cache.insert(kid.to_string(), key.clone()).await;
            Ok(key.clone())
        } else {
            Err(AuthError::InvalidToken)
        }
    }
}

#[async_trait]
impl Authenticator for JwtValidator {
    async fn validate_token(&self, token: &str) -> Result<Claims, AuthError> {
        let header = decode_header(token)?;
        let kid = header.kid.ok_or(AuthError::InvalidToken)?;
        let key = self.get_key(&kid).await?;
        
        let mut validation = Validation::new(header.alg);
        validation.set_issuer(&[&self.issuer]);
        validation.set_audience(&[&self.audience]);
        
        let token_data = decode::<Claims>(token, &key, &validation)?;
        
        Ok(token_data.claims)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Jwks {
    keys: Vec<Jwk>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Jwk {
    kid: Option<String>,
    kty: String,
    alg: Algorithm,
    n: String,
    e: String,
    r#use: String,
}