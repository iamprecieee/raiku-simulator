use axum::{Json, http::StatusCode, response::IntoResponse};
use chrono::Utc;
use serde_json::json;

use crate::models::responses::ApiResponse;

#[utoipa::path(
    get,
    path = "/health",
    tag = "Health",
    responses(
        (status = 200, description = "Server is healthy", body = ApiResponse),
    )
)]
pub async fn health_check() -> impl IntoResponse {
    let data = json!({
        "status": "healthy",
        "timestamp": Utc::now()
    });

    (
        StatusCode::OK,
        Json(ApiResponse::success("Server is healthy.".to_string(), data)),
    )
        .into_response()
}
