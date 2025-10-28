use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::Semaphore;
use tokio::time::{Duration, interval};

#[derive(Debug, Clone)]
pub struct RateLimiter(Arc<BucketToken>);

impl RateLimiter {
    pub fn new(requests_per_second: usize) -> Self {
        Self(Arc::new(BucketToken::new(requests_per_second)))
    }

    pub async fn acquire(&self) {
        // This can return an error if the semaphore is closed, but we
        // never close it, so this error can never happen.
        let permit = self.0.sem.acquire().await.unwrap();
        // To avoid releasing the permit back to the semaphore, we use
        // the `SemaphorePermit::forget` method.
        permit.forget();
    }

    /// Decrease the rate per second, beware that if we decrease too much it wont panic nor raise an error but the usize will increase to 18446744073709551615
    pub fn decrease_rate(&self) {
        if self.0.requests_per_second.load(Ordering::Relaxed) > 0 {
            self.0.requests_per_second.fetch_sub(1, Ordering::Relaxed);
        }
    }
}

#[derive(Debug)]
struct BucketToken {
    sem: Arc<Semaphore>,
    jh: tokio::task::JoinHandle<()>,
    requests_per_second: Arc<AtomicUsize>,
}

impl BucketToken {
    fn new(requests_per_second: usize) -> Self {
        // Start with only 1 permit to force rate limiting from the beginning
        let sem = Arc::new(Semaphore::new(1));

        let requests_per_second = Arc::new(AtomicUsize::new(requests_per_second));
        let rps_clone = requests_per_second.clone();

        // refills the tokens at the specified interval
        let jh = tokio::spawn({
            let sem = sem.clone();
            async move {
                let mut last_rps = rps_clone.load(Ordering::Relaxed);

                // Add 1 permit every (1/requests_per_second) seconds. Make one tick so the following ones are spaced out correctly
                let mut interval_period = interval(Duration::from_secs_f32(1.0 / last_rps as f32));
                interval_period.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
                interval_period.tick().await;

                loop {
                    interval_period.tick().await;

                    // Check if the rate has changed
                    let current_rps = rps_clone.load(Ordering::Relaxed);
                    if current_rps != last_rps {
                        last_rps = current_rps;
                        interval_period = interval(Duration::from_secs_f32(1.0 / last_rps as f32));
                        interval_period
                            .set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
                        interval_period.tick().await;
                    }

                    if sem.available_permits() < current_rps {
                        sem.add_permits(1);
                    }
                }
            }
        });

        Self {
            jh,
            sem,
            requests_per_second,
        }
    }
}

impl Drop for BucketToken {
    fn drop(&mut self) {
        // Kill the background task so it stops taking up resources when we
        // don't need it anymore.
        self.jh.abort();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::{task::JoinSet, time::Instant};

    async fn send_request(value: f64) -> f64 {
        tokio::time::sleep(Duration::from_secs_f64(0.3 / value + 1.0)).await;
        value
    }

    #[tokio::test]
    async fn test_token_bucket_basic_rate_limiting() {
        let requests_per_second = 2;
        let ratelimiter = RateLimiter::new(requests_per_second);

        let start = Instant::now();
        let mut jhs = JoinSet::new();

        for i in 1..11 {
            let bucket = ratelimiter.clone();
            jhs.spawn(async move {
                bucket.acquire().await;
                // Acquire permit before sending request.
                let r = start.elapsed();
                // Send the request.
                send_request(i as f64).await;
                r
            });
        }

        let mut request_times = Vec::new();
        while let Some(jh) = jhs.join_next_with_id().await {
            let (_, response) = jh.unwrap();
            request_times.push(response);
        }
        // First request should be immediate
        assert!(request_times[0] < Duration::from_millis(10));

        // Subsequent requests should be spaced by ~500ms (2 requests/second)
        for i in 1..request_times.len() {
            let time_between = request_times[i] - request_times[i - 1];
            assert!(
                time_between >= Duration::from_millis(450),
                "Request {i} was too fast: {time_between:?}",
            );
            assert!(
                time_between <= Duration::from_millis(550),
                "Request {i} was too slow: {time_between:?}",
            );
        }
    }
}
