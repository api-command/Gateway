use serde::{Deserialize, Serialize};
use reqwest::Url;
use thiserror::Error;
use async_trait::async_trait;
use moka::future::Cache;
use std::time::Duration;

#[derive(Debug, Serialize, Deserialize)]
pub struct IntrospectionResponse {
    pub active: bool,
    pub scope: String,
    pub client_id: String,
    pub username: String,
    pub exp: Option<u64>,
}

#[derive(Debug, Error)]
pub enum OAuthError {
    #[error("Invalid token")]
    InvalidToken,
    #[error("Introspection failed: {0}")]
    IntrospectionError(String),
    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),
}

pub struct OAuthIntrospector {
    client: reqwest::Client,
    introspection_url: Url,
    client_id: String,
    client_secret: String,
    cache: Cache<String, IntrospectionResponse>,
}

impl OAuthIntrospector {
    pub fn new(
        introspection_url: Url,
        client_id: String,
        client_secret: String,
    ) -> Self {
        Self {
            client: reqwest::Client::new(),
            introspection_url,
            client_id,
            client_secret,
            cache: Cache::builder()
                .time_to_live(Duration::from_secs(300))
                .build(),
        }
    }

    pub async fn introspect_token(
        &self,
        token: &str,
    ) -> Result<IntrospectionResponse, OAuthError> {
        if let Some(response) = self.cache.get(token) {
            return Ok(response);
        }

        let params = [
            ("token", token),
            ("client_id", &self.client_id),
            ("client_secret", &self.client_secret),
        ];

        let response = self.client
            .post(self.introspection_url.clone())
            .form(&params)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(OAuthError::IntrospectionError(
                response.status().to_string(),
            ));
        }

        let introspection: IntrospectionResponse = response.json().await?;
        
        if introspection.active {
            self.cache.insert(token.to_string(), introspection.clone()).await;
            Ok(introspection)
        } else {
            Err(OAuthError::InvalidToken)
        }
    }
}

#[async_trait]
pub trait TokenValidator {
    async fn validate_token(&self, token: &str) -> Result<(), OAuthError>;
}

pub struct OAuthTokenValidator {
    introspector: OAuthIntrospector,
    required_scopes: Vec<String>,
}

impl OAuthTokenValidator {
    pub fn new(introspector: OAuthIntrospector, required_scopes: Vec<String>) -> Self {
        Self {
            introspector,
            required_scopes,
        }
    }
}

#[async_trait]
impl TokenValidator for OAuthTokenValidator {
    async fn validate_token(&self, token: &str) -> Result<(), OAuthError> {
        let introspection = self.introspector.introspect_token(token).await?;
        
        if !self.required_scopes.iter().all(|scope| 
            introspection.scope.split(' ').any(|s| s == scope)
        ) {
            return Err(OAuthError::InvalidToken);
        }
        
        Ok(())
    }
}