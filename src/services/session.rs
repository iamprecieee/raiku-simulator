use axum::http::{HeaderMap, StatusCode, header};

use crate::managers::session::SessionManager;

pub async fn get_session_from_cookie(
    headers: &HeaderMap,
    query_session_id: Option<&String>,
    sessions: &SessionManager,
) -> Result<String, StatusCode> {
    let session_id_from_cookie = headers
        .get(header::COOKIE)
        .and_then(|h| h.to_str().ok())
        .and_then(|cookies| {
            cookies
                .split(';')
                .find(|c| c.trim().starts_with("raiku_session="))
                .and_then(|c| c.split('=').nth(1))
                .map(|s| s.to_string())
        });

    // Fall back to query parameter
    let session_id = session_id_from_cookie
        .or_else(|| query_session_id.cloned())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if sessions.validate_session(&session_id).await {
        Ok(session_id)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}
