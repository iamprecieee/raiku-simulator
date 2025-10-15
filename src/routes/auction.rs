use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde_json::{Value, json};

use crate::{app::api::AppContext, models::responses::ApiResponse};

#[utoipa::path(
    get,
    path = "/auctions/jit",
    tag = "Auction",
    responses(
        (status = 200, description = "Active JIT auctions retrieved", body = ApiResponse),
    )
)]
pub async fn list_jit_auctions(State(context): State<AppContext>) -> impl IntoResponse {
    let auctions = context.state.auctions.read().await;

    let jit_auctions: Vec<Value> = auctions
        .get_active_jit_auctions()
        .iter()
        .map(|auction| {
            json!({
                "slot_number": auction.slot_number,
                "min_bid": auction.min_bid,
                "current_winner": auction.current_highest_bidder,
                "created_at": auction.created_at
            })
        })
        .collect();

    (
        StatusCode::OK,
        Json(ApiResponse::success(
            "JIT auctions fetched successfully.".into(),
            json!({
                "auctions": jit_auctions,
                "count": jit_auctions.len()
            }),
        )),
    )
        .into_response()
}

#[utoipa::path(
    get,
    path = "/auctions/aot",
    tag = "Auction",
    responses(
        (status = 200, description = "Active AOT auctions retrieved", body = ApiResponse),
    )
)]
pub async fn list_aot_auctions(State(context): State<AppContext>) -> impl IntoResponse {
    let auctions = context.state.auctions.read().await;

    let aot_auctions: Vec<Value> = auctions
        .get_active_aot_auctions()
        .iter()
        .map(|auction| {
            json!({
                "slot_number": auction.slot_number,
                "min_bid": auction.min_bid,
                "highest_bid": auction.get_highest_bid().map(|(_, amount, _)| amount),
                "bids_count": auction.bids.len(),
                "ends_at": auction.ends_at,
                "has_ended": auction.has_ended()
            })
        })
        .collect();

    (
        StatusCode::OK,
        Json(ApiResponse::success(
            "AOT auctions fetched successfully.".into(),
            json!({
                "auctions": aot_auctions,
                "count": aot_auctions.len()
            }),
        )),
    )
        .into_response()
}
