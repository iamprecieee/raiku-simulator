use axum::{
    Json,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use serde_json::json;

use crate::{
    MAX_COMPUTE_UNITS_PER_SLOT,
    app::api::AppContext,
    models::{
        requests::{AotBidRequest, JitBidRequest, TransactionQuery},
        responses::ApiResponse,
        slot::SlotState,
        transaction::Transaction,
    },
    services::session::get_session_from_cookie,
};

#[utoipa::path(
    post,
    path = "/transactions/jit",
    tag = "Transactions",
    request_body = JitBidRequest,
    responses(
        (status = 200, description = "JIT transaction submitted", body = ApiResponse),
        (status = 402, description = "Insufficient balance", body = ApiResponse),
        (status = 400, description = "Bad request", body = ApiResponse),
        (status = 500, description = "Internal server error", body = ApiResponse)
    )
)]
pub async fn submit_jit_transaction(
    State(context): State<AppContext>,
    headers: HeaderMap,
    Json(req): Json<JitBidRequest>,
) -> impl IntoResponse {
    let session_id =
        match get_session_from_cookie(&headers, req.session_id.as_ref(), &context.state.sessions)
            .await
        {
            Ok(sid) => sid,
            Err(_) => {
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(ApiResponse::failure(
                        "Session ID is missing or invalid",
                        401,
                    )),
                )
                    .into_response();
            }
        };

    let next_available_slot = {
        let marketplace = context.state.marketplace.read().await;
        marketplace.current_slot + 1
    };

    // Lock and update the game state for the current player
    {
        let mut game = context.state.game.write().await;
        let stats = game.get_or_create_player(session_id.clone());

        // Ensure the player has sufficient balance
        if !stats.is_balance_sufficient(req.bid_amount) {
            return (
                StatusCode::PAYMENT_REQUIRED,
                Json(ApiResponse::failure("Insufficient balance", 400)),
            )
                .into_response();
        }

        // Deduct balance or return an error
        if let Err(_) = stats.deduct_balance(req.bid_amount) {
            return (
                StatusCode::PAYMENT_REQUIRED,
                Json(ApiResponse::failure("Payment failed", 400)),
            )
                .into_response();
        } else {
            stats.track_bid(next_available_slot);
        }
    }

    // Reject if compute units exceed the max per slot
    if req.compute_units > MAX_COMPUTE_UNITS_PER_SLOT {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::failure(
                &format!(
                    "Compute units exceed maximum per slot: {}",
                    MAX_COMPUTE_UNITS_PER_SLOT
                ),
                400,
            )),
        )
            .into_response();
    }

    // Start JIT auction if it doesn't already exist
    if !context
        .state
        .auctions
        .read()
        .await
        .jit_auctions
        .contains_key(&next_available_slot)
    {
        if let Err(_) = context
            .state
            .start_jit_auction(next_available_slot, context.config.marketplace.base_fee_sol)
            .await
        {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::failure("JIT auction failed to start", 400)),
            )
                .into_response();
        }
    }

    // Submit the JIT bid for this slot
    if let Err(_) = context
        .state
        .submit_jit_bid(next_available_slot, session_id.clone(), req.bid_amount)
        .await
    {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::failure("JIT Bid submission failed", 400)),
        )
            .into_response();
    }

    // Update marketplace slot state with the bid
    {
        let mut marketplace = context.state.marketplace.write().await;
        if let Some(slot) = marketplace.slots.get_mut(&next_available_slot) {
            slot.state = SlotState::JitAuction {
                current_bid: req.bid_amount,
                bidder: session_id.clone(),
            };
        }
    }

    // Create and store the transaction
    let transaction = Transaction::jit(
        session_id.clone(),
        req.compute_units,
        req.bid_amount,
        req.data,
    );

    let transaction_id = transaction.id.clone();
    context
        .state
        .add_transaction(session_id.clone(), transaction)
        .await;

    (
        StatusCode::CREATED,
        Json(ApiResponse::success(
            "JIT bid submitted for next available slot".into(),
            json!({
                "transaction_id": transaction_id,
                "slot_number": next_available_slot,
                "bid_amount": req.bid_amount,
                "status": "auction_pending",
            }),
        )),
    )
        .into_response()
}

#[utoipa::path(
    post,
    path = "/transactions/aot",
    tag = "Transactions",
    request_body = AotBidRequest,
    responses(
        (status = 200, description = "AOT transaction submitted", body = ApiResponse),
        (status = 402, description = "Insufficient balance", body = ApiResponse),
        (status = 400, description = "Bad request", body = ApiResponse),
        (status = 500, description = "Internal server error", body = ApiResponse)
    )
)]
pub async fn submit_aot_transaction(
    State(context): State<AppContext>,
    headers: HeaderMap,
    Json(req): Json<AotBidRequest>,
) -> impl IntoResponse {
    let session_id =
        match get_session_from_cookie(&headers, req.session_id.as_ref(), &context.state.sessions)
            .await
        {
            Ok(sid) => sid,
            Err(_) => {
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(ApiResponse::failure(
                        "Session ID is missing or invalid",
                        401,
                    )),
                )
                    .into_response();
            }
        };

    // Validate the requested slot number
    let current_slot = context.state.get_current_slot().await;
    if req.slot_number < current_slot {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::failure("Invalid slot number", 400)),
        )
            .into_response();
    }

    // Lock and update the game state for the current player
    {
        let mut game = context.state.game.write().await;
        let stats = game.get_or_create_player(session_id.clone());

        // Ensure the player has sufficient balance
        if !stats.is_balance_sufficient(req.bid_amount) {
            return (
                StatusCode::PAYMENT_REQUIRED,
                Json(ApiResponse::failure("Insufficient balance", 400)),
            )
                .into_response();
        }

        // Deduct balance or return an error
        if let Err(_) = stats.deduct_balance(req.bid_amount) {
            return (
                StatusCode::PAYMENT_REQUIRED,
                Json(ApiResponse::failure("Payment failed", 400)),
            )
                .into_response();
        } else {
            stats.track_bid(req.slot_number);
        }
    }

    // Reject if compute units exceed the max per slot
    if req.compute_units > MAX_COMPUTE_UNITS_PER_SLOT {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::failure(
                &format!(
                    "Compute units exceed maximum per slot: {}",
                    MAX_COMPUTE_UNITS_PER_SLOT
                ),
                400,
            )),
        )
            .into_response();
    }

    // Start AOT auction for the requested slot if it doesn't already exist
    if !context
        .state
        .auctions
        .read()
        .await
        .aot_auctions
        .contains_key(&req.slot_number)
    {
        if let Err(_) = context
            .state
            .start_aot_auction(
                req.slot_number,
                context.config.marketplace.base_fee_sol,
                context.config.auction.aot_default_duration_sec,
            )
            .await
        {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::failure("AOT auction failed to start", 400)),
            )
                .into_response();
        }
    }

    // Submit the AOT bid for this slot
    if let Err(_) = context
        .state
        .submit_aot_bid(req.slot_number, session_id.clone(), req.bid_amount)
        .await
    {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::failure("AOT Bid submission failed", 400)),
        )
            .into_response();
    }

    // Update marketplace slot state with the bid
    {
        let mut marketplace = context.state.marketplace.write().await;
        if let Some(slot) = marketplace.slots.get_mut(&req.slot_number) {
            let auctions = context.state.auctions.read().await;
            if let Some(auction) = auctions.aot_auctions.get(&req.slot_number) {
                let ends_at = auction.ends_at;
                slot.state = SlotState::AotAuction {
                    highest_bid: req.bid_amount,
                    highest_bidder: session_id.clone(),
                    bids: vec![(session_id.clone(), req.bid_amount)],
                    ends_at,
                };
            }
        }
    }

    // Create and store the transaction
    let transaction = Transaction::aot(
        session_id.clone(),
        req.compute_units,
        req.bid_amount,
        req.slot_number,
        req.data,
    );

    let transaction_id = transaction.id.clone();
    context
        .state
        .add_transaction(session_id.clone(), transaction)
        .await;

    (
        StatusCode::CREATED,
        Json(ApiResponse::success(
            "AOT bid submitted for future slot".into(),
            json!({
                "transaction_id": transaction_id,
                "slot_number": req.slot_number,
                "bid_amount": req.bid_amount,
                "status": "auction_pending",
            }),
        )),
    )
        .into_response()
}

#[utoipa::path(
    get,
    path = "/transactions",
    tag = "Transactions",
    responses(
        (status = 200, description = "List of transactions", body = ApiResponse),
        (status = 401, description = "Unauthorized", body = ApiResponse),
    )
)]
pub async fn list_transactions(
    State(context): State<AppContext>,
    headers: HeaderMap,
    Query(query): Query<TransactionQuery>,
) -> impl IntoResponse {
    let session_id =
        match get_session_from_cookie(&headers, query.session_id.as_ref(), &context.state.sessions)
            .await
        {
            Ok(sid) => sid,
            Err(_) => {
                return (
                    StatusCode::UNAUTHORIZED,
                    Json(ApiResponse::failure(
                        "Session ID is missing or invalid",
                        401,
                    )),
                )
                    .into_response();
            }
        };

    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(20).min(100).max(1);
    let offset = (page - 1) * limit;

    if query.show_all.unwrap_or(false) {
        let all_transactions = context
            .state
            .get_all_transactions_paginated(offset, limit)
            .await;
        let total_count = context.state.get_global_transaction_count().await;
        let total_pages = (total_count + limit - 1) / limit;

        return (
            StatusCode::OK,
            Json(ApiResponse::success(
                "Global transactions fetched successfully".into(),
                json!({
                    "transactions": all_transactions,
                    "pagination": {
                        "current_page": page,
                        "total_pages": total_pages,
                        "page_size": limit,
                        "total_count": total_count,
                        "has_next": page < total_pages,
                        "has_prev": page > 1
                    },
                    "session_id": query.session_id,
                    "showing": "all"
                }),
            )),
        )
            .into_response();
    }

    let session_transactions = context
        .state
        .get_session_transactions_paginated(&session_id, offset, limit)
        .await;
    let total_count = context
        .state
        .get_session_transaction_count(&session_id)
        .await;
    let total_pages = (total_count + limit - 1) / limit;

    (
        StatusCode::OK,
        Json(ApiResponse::success(
            "Transactions for current session fetched successfully".into(),
            json!({
                "session_id": session_id,
                "transactions": session_transactions,
                "pagination": {
                    "current_page": page,
                    "total_pages": total_pages,
                    "page_size": limit,
                    "total_count": total_count,
                    "has_next": page < total_pages,
                    "has_prev": page > 1
                },
                "showing": "session_only"
            }),
        )),
    )
        .into_response()
}

#[utoipa::path(
    get,
    path = "/transactions/{transaction_id}",
    tag = "Transactions",
    params(
        ("transaction_id" = u64, Path, description = "ID of transaction to fetch")
    ),
    responses(
        (status = 200, description = "Transaction details", body = ApiResponse),
        (status = 404, description = "Transaction not found", body = ApiResponse)
    )
)]
pub async fn get_transaction(
    State(context): State<AppContext>,
    Path(transaction_id): Path<String>,
) -> impl IntoResponse {
    if let Some(transaction) = context.state.get_transaction_by_id(&transaction_id).await {
        (
            StatusCode::OK,
            Json(ApiResponse::success(
                "Transaction fetched successfully".into(),
                json!({
                    "transaction": transaction
                }),
            )),
        )
            .into_response()
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::failure("Transaction not found", 404)),
        )
            .into_response()
    }
}
