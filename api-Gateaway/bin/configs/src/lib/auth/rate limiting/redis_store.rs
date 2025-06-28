use redis::{Client, RedisResult, cmd};
use redis::AsyncCommands;
use thiserror::Error;
use serde::{Serialize, Deserialize};
use std::time::Duration;

#[derive(Debug, Error)]
pub enum RedisRateLimitError {
    #[error("Redis error: {0}")]
    RedisError(#[from] redis::RedisError),
    #[error("Rate limit exceeded for key {0}")]
    RateLimitExceeded(String),
}

#[derive(Clone)]
pub struct RedisRateLimiter {
    client: Client,
    lua_script: redis::Script,
}

#[derive(Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub capacity: i64,
    pub refill_amount: i64,
    pub refill_seconds: u64,
    pub burst: i64,
}

impl RedisRateLimiter {
    pub fn new(redis_url: &str) -> RedisResult<Self> {
        let client = Client::open(redis_url)?;
        let lua_script = redis::Script::new(include_str!("token_bucket.lua"));
        
        Ok(Self { client, lua_script })
    }

    pub async fn check_rate_limit(
        &self,
        key: &str,
        config: &RateLimitConfig,
    ) -> Result<(bool, i64), RedisRateLimitError> {
        let mut conn = self.client.get_async_connection().await?;
        
        let result: (i64, i64) = self.lua_script
            .key(key)
            .arg(config.capacity)
            .arg(config.refill_amount)
            .arg(config.refill_seconds)
            .arg(config.burst)
            .invoke_async(&mut conn)
            .await?;

        Ok((result.0 == 1, result.1))
    }

    pub async fn get_remaining(&self, key: &str) -> RedisResult<i64> {
        let mut conn = self.client.get_async_connection().await?;
        let remaining: i64 = conn.get(format!("{key}:remaining")).await?;
        Ok(remaining)
    }

    pub async fn reset_rate_limit(&self, key: &str) -> RedisResult<()> {
        let mut conn = self.client.get_async_connection().await?;
        cmd("DEL")
            .arg(&[format!("{key}:tokens"), format!("{key}:last_refill")])
            .query_async(&mut conn)
            .await
    }
}

// token_bucket.lua
/*
local key = KEYS[1]
local capacity = tonumber(ARGV[1])
local refill_amount = tonumber(ARGV[2])
local refill_seconds = tonumber(ARGV[3])
local burst = tonumber(ARGV[4])

local current_time = redis.call('TIME')[1]
local last_refill = tonumber(redis.call('GET', key..':last_refill') or current_time)
local tokens = tonumber(redis.call('GET', key..':tokens') or capacity)

local time_passed = current_time - last_refill
if time_passed > 0 then
    local refill_count = math.floor(time_passed / refill_seconds) * refill_amount
    tokens = math.min(tokens + refill_count, capacity + burst)
    last_refill = current_time - (time_passed % refill_seconds)
end

local allowed = 0
if tokens >= 1 then
    tokens = tokens - 1
    allowed = 1
end

redis.call('SET', key..':tokens', tokens)
redis.call('SET', key..':last_refill', last_refill)
redis.call('EXPIRE', key..':tokens', refill_seconds * 2)
redis.call('EXPIRE', key..':last_refill', refill_seconds * 2)

return { allowed, tokens }
*/