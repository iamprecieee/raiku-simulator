use std::{
    net::SocketAddr,
    sync::Arc,
    time::{Duration, Instant},
};

use axum::{
    extract::ConnectInfo,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use dashmap::DashMap;

#[derive(Clone)]
pub struct RateLimiter {
    buckets: Arc<DashMap<String, TokenBucket>>,
    requests_per_window: u32,
    window_duration: Duration,
}

#[derive(Debug)]
struct TokenBucket {
    tokens: u32,
    last_refill: Instant,
    window_start: Instant,
    request_count: u32,
}

impl RateLimiter {
    pub fn new(requests_per_second: u32) -> Self {
        Self {
            buckets: Arc::new(DashMap::new()),
            requests_per_window: requests_per_second * 60,
            window_duration: Duration::from_secs(60),
        }
    }

    fn get_client_key(&self, addr: &SocketAddr) -> String {
        addr.ip().to_string()
    }

    pub fn check_rate_limit(&self, client_key: &str) -> bool {
        let now = Instant::now();

        let mut entry = self
            .buckets
            .entry(client_key.to_string())
            .or_insert(TokenBucket {
                tokens: self.requests_per_window,
                last_refill: now,
                window_start: now,
                request_count: 0,
            });

        if now.duration_since(entry.window_start) >= self.window_duration {
            entry.window_start = now;
            entry.request_count = 0;
            entry.tokens = self.requests_per_window;
        }

        if entry.request_count >= self.requests_per_window {
            return false;
        }

        entry.request_count += 1;
        entry.last_refill = now;
        true
    }
}

pub async fn rate_limit_middleware(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    req: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let rate_limiter = req
        .extensions()
        .get::<RateLimiter>()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    let client_key = rate_limiter.get_client_key(&addr);

    if !rate_limiter.check_rate_limit(&client_key) {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }

    Ok(next.run(req).await)
}
