use axum::{
    extract::ConnectInfo,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use std::net::SocketAddr;

use crate::utils::rate_limiter::RateLimiter;

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
