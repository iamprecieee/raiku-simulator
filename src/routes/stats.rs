use axum::{
    Json,
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use serde_json::json;

use crate::{
    app::api::AppContext,
    models::{requests::TransactionQuery, responses::ApiResponse},
    services::session::get_session_from_cookie,
};

#[utoipa::path(
    get,
    path = "/game/player_stats",
    tag = "Game",
    params(
        ("session_id" = String, Query, description = "Optional session id in query")
    ),
    responses(
        (status = 200, description = "Player stats retrieved", body = ApiResponse),
        (status = 401, description = "Unauthorized", body = ApiResponse)
    )
)]
pub async fn get_player_stats(
    State(context): State<AppContext>,
    headers: HeaderMap,
    Query(query): Query<TransactionQuery>,
) -> impl IntoResponse {
    if let Ok(session_id) =
        get_session_from_cookie(&headers, query.session_id.as_ref(), &context.state.sessions).await
    {
        let mut game = context.state.game.write().await;
        let stats = game.get_or_create_player(session_id.clone());

        (
            StatusCode::OK,
            Json(ApiResponse::success(
                "Player stats fetched.".into(),
                json!(stats),
            )),
        )
            .into_response()
    } else {
        (
            StatusCode::UNAUTHORIZED,
            Json(ApiResponse::failure(
                "Session ID is missing or invalid",
                401,
            )),
        )
            .into_response()
    }
}

#[utoipa::path(
    get,
    path = "/game/leaderboard",
    tag = "Game",
    responses(
        (status = 200, description = "Leaderboard retrieved", body = ApiResponse)
    )
)]
pub async fn get_leaderboard(State(context): State<AppContext>) -> impl IntoResponse {
    let leaderboard = context.state.get_leaderboard().await;
    (
        StatusCode::OK,
        Json(ApiResponse::success(
            "Leaderboard fetched successfully".into(),
            json!(leaderboard),
        )),
    )
        .into_response()
}

#[utoipa::path(
    get,
    path = "/marketplace/status",
    tag = "Marketplace",
    responses(
        (status = 200, description = "Marketplace status retrieved", body = ApiResponse)
    )
)]
pub async fn marketplace_status(State(context): State<AppContext>) -> impl IntoResponse {
    let stats = context.state.get_marketplace_stats().await;
    let current_slot = context.state.get_current_slot().await;

    (
        StatusCode::OK,
        Json(ApiResponse::success(
            "Marketplace status fetched successfully".into(),
            json!({
                "current_slot": current_slot,
                "stats": stats,
                "slot_time_ms": context.config.marketplace.slot_duration_ms,
                "base_fee_sol": context.config.marketplace.base_fee_sol
            }),
        )),
    )
        .into_response()
}
