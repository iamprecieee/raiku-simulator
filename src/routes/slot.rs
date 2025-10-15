use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Json, extract::State};
use serde_json::{Value, json};

use crate::app::api::AppContext;
use crate::models::responses::ApiResponse;

#[utoipa::path(
    get,
    path = "/marketplace/slots",
    tag = "Marketplace",
    responses(
        (status = 200, description = "List of available slots", body = ApiResponse)
    )
)]
pub async fn list_slots(State(context): State<AppContext>) -> impl IntoResponse {
    let marketplace = context.state.marketplace.read().await;
    let current_slot = marketplace.current_slot;

    let slots: Vec<Value> = marketplace
        .slots
        .iter()
        .filter(|(slot_num, _)| **slot_num >= current_slot && **slot_num < current_slot + 50)
        .map(|(slot_num, slot)| {
            json!({
                "slot_number": slot_num,
                "state": slot.state,
                "estimated_time": slot.estimated_time,
                "base_fee": slot.base_fee,
                "compute_units_available": slot.compute_units_available,
                "compute_units_used": slot.compute_units_used
            })
        })
        .collect();

    let data = json!({
        "current_slot": current_slot,
        "slots": slots
    });

    (
        StatusCode::OK,
        Json(ApiResponse::success(
            "Slots fetched successfully.".into(),
            data,
        )),
    )
        .into_response()
}

#[utoipa::path(
    get,
    path = "/marketplace/slots/{slot_number}",
    tag = "Marketplace",
    params(
        ("slot_number" = u64, Path, description = "Slot number to fetch")
    ),
    responses(
        (status = 200, description = "Slot details", body = ApiResponse),
        (status = 404, description = "Slot not found", body = ApiResponse)
    )
)]
pub async fn get_slot(
    State(context): State<AppContext>,
    Path(slot_number): Path<u64>,
) -> impl IntoResponse {
    let marketplace = context.state.marketplace.read().await;

    if let Some(slot) = marketplace.slots.get(&slot_number) {
        let data = json!({
            "slot_number": slot_number,
            "state": slot.state,
            "estimated_time": slot.estimated_time,
            "base_fee": slot.base_fee,
            "compute_units_available": slot.compute_units_available,
            "compute_units_used": slot.compute_units_used
        });

        (
            StatusCode::OK,
            Json(ApiResponse::success("Slot found.".into(), data)),
        )
            .into_response()
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::failure("Slot not found", 404)),
        )
            .into_response()
    }
}
