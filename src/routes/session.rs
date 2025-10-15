use axum::{
    Json,
    extract::State,
    http::{HeaderMap, StatusCode, header},
    response::IntoResponse,
};
use serde_json::json;

use crate::app::api::AppContext;

use crate::models::responses::ApiResponse;

#[utoipa::path(
    post,
    path = "/sessions",
    tag = "Session",
    responses(
        (status = 200, description = "Session created or validated", body = ApiResponse),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn create_or_validate_session(
    State(context): State<AppContext>,
    headers: HeaderMap,
) -> impl IntoResponse {
    let session_id = headers
        .get(header::COOKIE)
        .and_then(|h| h.to_str().ok())
        .and_then(|cookies| {
            cookies
                .split(';')
                .find(|c| c.trim().starts_with("raiku_session="))
                .and_then(|c| c.split('=').nth(1))
        });

    let (session, is_new) = if let Some(sid) = session_id {
        if let Some(sess) = context.state.sessions.get_session(sid).await {
            (sess, false)
        } else {
            (context.state.sessions.create_session().await, true)
        }
    } else {
        (context.state.sessions.create_session().await, true)
    };

    let cookie_value = format!(
        "raiku_session={}; Path=/; HttpOnly; SameSite=None; Secure; Max-Age={}",
        session.id, 86400
    );

    let data = json!({
        "session_id": session.id,
        "status": if is_new { "created" } else { "validated" },
        "created_at": session.created_at,
        "expires_at": session.expires_at
    });

    let api_response = ApiResponse::success("Session created or validated.".to_string(), data);

    let mut response = Json(api_response).into_response();

    if let Ok(cookie_header) = cookie_value.parse() {
        response
            .headers_mut()
            .insert(header::SET_COOKIE, cookie_header);
        response
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::failure("Failed to set session cookie", 500)),
        )
            .into_response()
    }
}
