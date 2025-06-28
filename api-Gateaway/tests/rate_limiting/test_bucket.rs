use crate::rate_limiting::token_bucket::TokenBucket;
use std::time::{Duration, Instant};

#[test]
fn test_token_bucket_refill() {
    let bucket = TokenBucket::new(10, 5, Duration::from_secs(1)).unwrap();
    
    // Exhaust initial tokens
    for _ in 0..10 {
        assert!(bucket.try_acquire(1).is_ok());
    }
    assert!(bucket.try_acquire(1).is_err());
    
    // Wait for refill
    std::thread::sleep(Duration::from_secs(1));
    assert!(bucket.try_acquire(5).is_ok());
    assert!(bucket.try_acquire(1).is_err());
}

#[tokio::test]
async fn test_redis_rate_limiting() {
    use crate::rate_limiting::redis_store::{RedisRateLimiter, RateLimitConfig};
    
    let limiter = RedisRateLimiter::new("redis://localhost:6379").unwrap();
    let test_key = format!("test:{}", uuid::Uuid::new_v4());
    let config = RateLimitConfig {
        capacity: 5,
        refill_amount: 1,
        refill_seconds: 1,
        burst: 2,
    };

    // First 5 requests should succeed
    for _ in 0..5 {
        let (allowed, _) = limiter.check_rate_limit(&test_key, &config).await.unwrap();
        assert!(allowed);
    }

    // Next request should be blocked
    let (allowed, _) = limiter.check_rate_limit(&test_key, &config).await.unwrap();
    assert!(!allowed);

    // Wait for refill
    tokio::time::sleep(Duration::from_secs(1)).await;
    let (allowed, remaining) = limiter.check_rate_limit(&test_key, &config).await.unwrap();
    assert!(allowed);
    assert_eq!(remaining, 1);
}