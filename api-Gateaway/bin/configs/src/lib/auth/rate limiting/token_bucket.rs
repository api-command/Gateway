use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RateLimitError {
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    #[error("Invalid rate limit configuration")]
    InvalidConfig,
}

pub struct TokenBucket {
    capacity: u64,
    refill_amount: u64,
    refill_interval: Duration,
    last_refill: AtomicU64,
    tokens: AtomicU64,
}

impl TokenBucket {
    pub fn new(capacity: u64, refill_amount: u64, refill_interval: Duration) -> Result<Self, RateLimitError> {
        if capacity == 0 || refill_amount == 0 || refill_interval.as_secs() == 0 {
            return Err(RateLimitError::InvalidConfig);
        }
        
        Ok(Self {
            capacity,
            refill_amount,
            refill_interval,
            last_refill: AtomicU64::new(Self::current_time()),
            tokens: AtomicU64::new(capacity),
        })
    }

    fn current_time() -> u64 {
        Instant::now().elapsed().as_secs()
    }

    fn refill(&self) {
        let now = Self::current_time();
        let last = self.last_refill.load(Ordering::Relaxed);
        
        if now - last >= self.refill_interval.as_secs() {
            let refill_count = ((now - last) / self.refill_interval.as_secs()) * self.refill_amount;
            
            self.tokens.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |t| {
                Some((t + refill_count).min(self.capacity))
            }).ok();
            
            self.last_refill.store(now, Ordering::Relaxed);
        }
    }

    pub fn try_acquire(&self, tokens: u64) -> Result<(), RateLimitError> {
        self.refill();
        
        let mut current = self.tokens.load(Ordering::Relaxed);
        loop {
            if current < tokens {
                return Err(RateLimitError::RateLimitExceeded);
            }
            
            match self.tokens.compare_exchange_weak(
                current,
                current - tokens,
                Ordering::Relaxed,
                Ordering::Relaxed
            ) {
                Ok(_) => return Ok(()),
                Err(updated) => current = updated,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_basic_rate_limiting() {
        let bucket = TokenBucket::new(10, 5, Duration::from_secs(1)).unwrap();
        
        // Exhaust initial tokens
        for _ in 0..10 {
            assert!(bucket.try_acquire(1).is_ok());
        }
        assert!(bucket.try_acquire(1).is_err());
        
        // Wait for refill
        thread::sleep(Duration::from_secs(1));
        assert!(bucket.try_acquire(5).is_ok());
        assert!(bucket.try_acquire(1).is_err());
    }
}