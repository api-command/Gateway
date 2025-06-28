use moka::future::Cache;
use redis::AsyncCommands;
use std::sync::Arc;
use std::time::Duration;
use crate::models::ApiRequest;

pub struct CacheService {
    local_cache: Cache<String, Vec<u8>>,
    redis: Option<Arc<redis::Client>>,
}

impl CacheService {
    pub fn new(local_cache_size: u64, ttl: Duration, redis_client: Option<redis::Client>) -> Self {
        let local_cache = Cache::builder()
            .max_capacity(local_cache_size)
            .time_to_live(ttl)
            .build();
        
        Self {
            local_cache,
            redis: redis_client.map(Arc::new),
        }
    }

    pub async fn get(&self, key: &str) -> Option<Vec<u8>> {
        // Check local cache first
        if let Some(data) = self.local_cache.get(key).await {
            return Some(data);
        }
        
        // Fallback to Redis
        if let Some(redis) = &self.redis {
            let mut conn = redis.get_async_connection().await.ok()?;
            let data: Vec<u8> = conn.get(key).await.ok()?;
            self.local_cache.insert(key.to_string(), data.clone()).await;
            Some(data)
        } else {
            None
        }
    }

    pub async fn set(&self, key: &str, value: Vec<u8>, ttl: Option<Duration>) {
        // Update local cache
        self.local_cache.insert(key.to_string(), value.clone()).await;
        
        // Propagate to Redis if available
        if let Some(redis) = &self.redis {
            let mut conn = redis.get_async_connection().await.unwrap();
            let _ = conn.set_ex(key, value, ttl.unwrap_or_else(|| self.local_cache.time_to_live()))
                .await;
        }
    }

    pub async fn invalidate(&self, key: &str) {
        self.local_cache.invalidate(key).await;
        if let Some(redis) = &self.redis {
            let mut conn = redis.get_async_connection().await.unwrap();
            let _ = conn.del(key).await;
        }
    }

    pub fn generate_cache_key(&self, req: &ApiRequest) -> String {
        format!(
            "{}:{}:{}",
            req.method,
            req.uri.path(),
            md5::compute(req.uri.query().unwrap_or_default())
        )
    }
}