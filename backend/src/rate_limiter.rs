use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, Semaphore};
use tokio::time::sleep;
use tracing::{debug, warn, error};

/// Configuration for rate limiting
#[derive(Clone, Debug)]
pub struct RateLimitConfig {
    /// Maximum requests per time window
    pub max_requests: u32,
    /// Time window duration
    pub window_duration: Duration,
    /// Initial backoff duration
    pub initial_backoff: Duration,
    /// Maximum backoff duration
    pub max_backoff: Duration,
    /// Backoff multiplier
    pub backoff_multiplier: f64,
    /// Maximum number of retry attempts
    pub max_retries: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests: 100,
            window_duration: Duration::from_secs(60),
            initial_backoff: Duration::from_millis(500),
            max_backoff: Duration::from_secs(30),
            backoff_multiplier: 2.0,
            max_retries: 3,
        }
    }
}

impl RateLimitConfig {
    pub fn spotify_config() -> Self {
        Self {
            max_requests: 100,
            window_duration: Duration::from_secs(60),
            initial_backoff: Duration::from_millis(200),
            max_backoff: Duration::from_secs(10),
            backoff_multiplier: 2.0,
            max_retries: 3,
        }
    }

    pub fn youtube_config() -> Self {
        Self {
            max_requests: 100,
            window_duration: Duration::from_secs(100),
            initial_backoff: Duration::from_millis(300),
            max_backoff: Duration::from_secs(15),
            backoff_multiplier: 1.5,
            max_retries: 3,
        }
    }
}

/// Tracks request timestamps for rate limiting
#[derive(Debug)]
struct RequestTracker {
    requests: VecDeque<Instant>,
    config: RateLimitConfig,
}

impl RequestTracker {
    fn new(config: RateLimitConfig) -> Self {
        Self {
            requests: VecDeque::new(),
            config,
        }
    }

    /// Check if we can make a request now, returns delay if we need to wait
    fn check_rate_limit(&mut self) -> Option<Duration> {
        let now = Instant::now();
        
        // Remove old requests outside the window
        while let Some(&front) = self.requests.front() {
            if now.duration_since(front) > self.config.window_duration {
                self.requests.pop_front();
            } else {
                break;
            }
        }

        // Check if we're at the limit
        if self.requests.len() >= self.config.max_requests as usize {
            if let Some(&oldest) = self.requests.front() {
                let wait_time = self.config.window_duration - now.duration_since(oldest);
                return Some(wait_time);
            }
        }

        None
    }

    /// Record a successful request
    fn record_request(&mut self) {
        self.requests.push_back(Instant::now());
    }
}

/// Rate limiter with request queuing and exponential backoff
pub struct RateLimiter {
    tracker: Arc<Mutex<RequestTracker>>,
    semaphore: Arc<Semaphore>,
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        let max_concurrent = std::cmp::min(config.max_requests / 4, 10); // Conservative concurrent limit
        
        Self {
            tracker: Arc::new(Mutex::new(RequestTracker::new(config))),
            semaphore: Arc::new(Semaphore::new(max_concurrent as usize)),
        }
    }

    /// Execute a request with rate limiting and exponential backoff
    pub async fn execute<T, E, F, Fut>(&self, request_fn: F) -> Result<T, E>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T, E>>,
        E: std::fmt::Display,
    {
        let _permit = self.semaphore.acquire().await.unwrap();
        
        let config = {
            let tracker = self.tracker.lock().await;
            tracker.config.clone()
        };

        for attempt in 0..=config.max_retries {
            // Check rate limit and wait if necessary
            {
                let mut tracker = self.tracker.lock().await;
                if let Some(delay) = tracker.check_rate_limit() {
                    debug!("Rate limit hit, waiting {:?}", delay);
                    drop(tracker); // Release the lock before sleeping
                    sleep(delay).await;
                }
            }

            // Execute the request
            match request_fn().await {
                Ok(result) => {
                    // Record successful request
                    let mut tracker = self.tracker.lock().await;
                    tracker.record_request();
                    debug!("Request successful on attempt {}", attempt + 1);
                    return Ok(result);
                }
                Err(e) => {
                    if attempt < config.max_retries {
                        let backoff_duration = calculate_backoff(
                            config.initial_backoff,
                            config.max_backoff,
                            config.backoff_multiplier,
                            attempt,
                        );
                        
                        warn!(
                            "Request failed on attempt {} ({}), retrying after {:?}",
                            attempt + 1,
                            e,
                            backoff_duration
                        );
                        
                        sleep(backoff_duration).await;
                    } else {
                        error!("Request failed after {} attempts: {}", config.max_retries + 1, e);
                        return Err(e);
                    }
                }
            }
        }

        unreachable!()
    }

    /// Execute a batch of requests with proper spacing
    pub async fn execute_batch<T, E, F, Fut>(
        &self,
        requests: Vec<F>,
        batch_size: usize,
    ) -> Vec<Result<T, E>>
    where
        F: Fn() -> Fut + Send,
        Fut: std::future::Future<Output = Result<T, E>> + Send,
        T: Send,
        E: std::fmt::Display + Send,
    {
        let mut results = Vec::new();
        
        for batch in requests.chunks(batch_size) {
            let batch_futures: Vec<_> = batch
                .iter()
                .map(|req| self.execute(req))
                .collect();
            
            let batch_results = futures::future::join_all(batch_futures).await;
            results.extend(batch_results);
            
            // Add a small delay between batches to be extra safe
            if results.len() < requests.len() {
                sleep(Duration::from_millis(100)).await;
            }
        }
        
        results
    }
}

/// Calculate exponential backoff duration
fn calculate_backoff(
    initial: Duration,
    max_duration: Duration,
    multiplier: f64,
    attempt: u32,
) -> Duration {
    let backoff_ms = initial.as_millis() as f64 * multiplier.powi(attempt as i32);
    let backoff_duration = Duration::from_millis(backoff_ms as u64);
    
    std::cmp::min(backoff_duration, max_duration)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{timeout, Duration as TokioDuration};

    #[tokio::test]
    async fn test_rate_limiter_basic() {
        let config = RateLimitConfig {
            max_requests: 2,
            window_duration: Duration::from_millis(1000),
            initial_backoff: Duration::from_millis(100),
            max_backoff: Duration::from_secs(1),
            backoff_multiplier: 2.0,
            max_retries: 1,
        };
        
        let rate_limiter = RateLimiter::new(config);
        
        // These should succeed quickly
        let start = Instant::now();
        let result1 = rate_limiter.execute(|| async { Ok::<_, &str>("success1") }).await;
        let result2 = rate_limiter.execute(|| async { Ok::<_, &str>("success2") }).await;
        
        assert!(result1.is_ok());
        assert!(result2.is_ok());
        
        // Third request should be delayed, but let's use a timeout to avoid test hanging
        let result3 = timeout(
            TokioDuration::from_secs(2),
            rate_limiter.execute(|| async { Ok::<_, &str>("success3") })
        ).await;
        
        let elapsed = start.elapsed();
        
        assert!(result3.is_ok());
        assert!(result3.unwrap().is_ok());
        assert!(elapsed >= Duration::from_millis(800)); // Should have waited
    }

    #[tokio::test]
    async fn test_exponential_backoff() {
        let backoff1 = calculate_backoff(
            Duration::from_millis(100),
            Duration::from_secs(10),
            2.0,
            0,
        );
        assert_eq!(backoff1, Duration::from_millis(100));

        let backoff2 = calculate_backoff(
            Duration::from_millis(100),
            Duration::from_secs(10),
            2.0,
            1,
        );
        assert_eq!(backoff2, Duration::from_millis(200));

        let backoff3 = calculate_backoff(
            Duration::from_millis(100),
            Duration::from_millis(150),
            2.0,
            2,
        );
        assert_eq!(backoff3, Duration::from_millis(150)); // Capped at max
    }
}