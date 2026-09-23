use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct RateLimiter {
    requests: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
    limit: usize,
    window: Duration,
}

impl RateLimiter {
    pub fn new(limit: usize, window_secs: u64) -> Self {
        Self {
            requests: Arc::new(Mutex::new(HashMap::new())),
            limit,
            window: Duration::from_secs(window_secs),
        }
    }

    pub fn check(&self, key: &str) -> bool {
        let now = Instant::now();
        let mut map = self.requests.lock().unwrap();
        let timestamps = map.entry(key.to_string()).or_insert_with(Vec::new);

        // Retain only requests in window
        let window = self.window;
        timestamps.retain(|&t| now.duration_since(t) < window);

        if timestamps.len() >= self.limit {
            false
        } else {
            timestamps.push(now);
            true
        }
    }
}
