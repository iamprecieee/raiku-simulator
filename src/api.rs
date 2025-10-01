use std::convert::Infallible;

use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response, Sse},
    routing::{get, post},
    Json, Router,
};
use futures_util::stream::{self, Stream};
use serde::Deserialize;
use serde_json::{json, Value};
use tower_http::cors::CorsLayer;

use crate::{
    config::Config, rate_limiter::{rate_limit_middleware, RateLimiter}, session::SessionManager, state::AppState, transaction::Transaction
};

#[derive(Clone)]
pub struct AppContext {
    pub state: AppState,
    pub config: Config,
    pub rate_limiter: RateLimiter,
}

#[derive(Deserialize)]
pub struct JitBidRequest {
    bid_amount: f64,
    compute_units: u64,
    data: String,
}

#[derive(Deserialize)]
pub struct AotBidRequest {
    slot_number: u64,
    bid_amount: f64,
    compute_units: u64,
    data: String,
}

#[derive(Deserialize)]
pub struct TransactionQuery {
    session_id: Option<String>,
    page: Option<u32>,
    limit: Option<u32>,
    show_all: Option<bool>,
}

#[derive(Deserialize)]
pub struct TransactionBatchQuery {
    page: Option<u32>,
    limit: Option<u32>,
}

pub fn create_api_router(context: AppContext) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(
            context
                .config
                .server
                .cors_origins
                .iter()
                .map(|origin| origin.parse().unwrap())
                .collect::<Vec<_>>(),
        )
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::COOKIE,
            axum::http::header::CACHE_CONTROL,
        ])
        .allow_credentials(true);

    Router::new()
        .route("/sessions", post(create_or_validate_session))
        .route("/events", get(sse_handler))
        .route("/marketplace/status", get(marketplace_status))
        .route("/marketplace/slots", get(list_slots))
        .route("/marketplace/slots/{slot_number}", get(get_slot))
        .route("/auctions/jit", get(list_jit_auctions))
        .route("/auctions/aot", get(list_aot_auctions))
        .route("/transactions/jit", post(submit_jit_transaction))
        .route("/transactions/aot", post(submit_aot_transaction))
        .route("/transactions", get(list_transactions))
        .route("/transactions/all", get(list_all_transactions))
        .route("/transactions/{transaction_id}", get(get_transaction))
        .route("/health", get(health_check))
        .route("/game/stats", get(get_player_stats))
        .route("/game/leaderboard", get(get_leaderboard))
        .layer(axum::middleware::from_fn(rate_limit_middleware))
        .layer(axum::Extension(context.rate_limiter.clone()))
        .layer(cors)
        .with_state(context)
}

async fn create_or_validate_session(
    State(context): State<AppContext>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
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
        session.id,
        86400
    );

    let body = json!({
        "session_id": session.id,
        "status": if is_new { "created" } else { "validated" },
        "created_at": session.created_at,
        "expires_at": session.expires_at
    });

    let mut response = Json(body).into_response();
    response.headers_mut().insert(
        header::SET_COOKIE,
        cookie_value.parse().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    );
    Ok(response)
}

async fn get_session_from_cookie(
    headers: &HeaderMap,
    query_session_id: Option<&String>,
    sessions: &SessionManager,
) -> Result<String, StatusCode> {
    let session_id_from_cookie = headers
        .get(header::COOKIE)
        .and_then(|h| h.to_str().ok())
        .and_then(|cookies| {
            cookies.split(';')
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

async fn sse_handler(
    State(context): State<AppContext>,
) -> Sse<impl Stream<Item = Result<axum::response::sse::Event, Infallible>>> {
    let receiver = context.state.events.subscribe();

    let stream = stream::unfold(receiver, |mut rx| async move {
        match rx.recv().await {
            Ok(event) => {
                let event_data = serde_json::to_string(&event).unwrap_or_default();
                let sse_event = axum::response::sse::Event::default().data(event_data);
                Some((Ok(sse_event), rx))
            }
            Err(_) => None,
        }
    });

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(std::time::Duration::from_secs(30))
            .text("keep-alive"),
    )
}

async fn marketplace_status(State(context): State<AppContext>) -> Json<Value> {
    let stats = context.state.get_marketplace_stats().await;
    let current_slot = context.state.get_current_slot().await;

    Json(json!({
        "current_slot": current_slot,
        "stats": stats,
        "slot_time_ms": context.config.marketplace.slot_time_ms,
        "base_fee_sol": context.config.marketplace.base_fee_sol
    }))
}

async fn list_slots(
    State(context): State<AppContext>,
) -> Result<Json<Value>, StatusCode> {
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

        Ok(Json(json!({  
            "current_slot": current_slot,
            "slots": slots
        })))
}

async fn get_slot(
    State(context): State<AppContext>,
    Path(slot_number): Path<u64>,
) -> Result<Json<Value>, StatusCode> {
    let marketplace = context.state.marketplace.read().await;

    marketplace
        .slots
        .get(&slot_number)
        .map(|slot| {
            Json(json!({
                "slot_number": slot_number,
                "state": slot.state,
                "estimated_time": slot.estimated_time,
                "base_fee": slot.base_fee,
                "compute_units_available": slot.compute_units_available,
                "compute_units_used": slot.compute_units_used
            }))
        })
        .ok_or(StatusCode::NOT_FOUND)
}

async fn list_jit_auctions(State(context): State<AppContext>) -> Json<Value> {
    let auctions = context.state.auctions.read().await;
    let jit_auctions: Vec<Value> = auctions
        .get_active_jit_auctions()
        .iter()
        .map(|auction| {
            json!({
                "slot_number": auction.slot_number,
                "min_bid": auction.min_bid,
                "current_winner": auction.current_winner,
                "created_at": auction.created_at
            })
        })
        .collect();

    Json(json!({
        "auctions": jit_auctions,
        "count": jit_auctions.len()
    }))
}

async fn list_aot_auctions(State(context): State<AppContext>) -> Json<Value> {
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

    Json(json!({
        "auctions": aot_auctions,
        "count": aot_auctions.len()
    }))
}

async fn submit_jit_transaction(
    State(context): State<AppContext>,
    headers: HeaderMap,
    Json(req): Json<JitBidRequest>,
) -> Result<Json<Value>, StatusCode> {
    let session_id = get_session_from_cookie(&headers, None, &context.state.sessions).await?;
    let next_slot = {
        let marketplace = context.state.marketplace.read().await;
        marketplace.current_slot + 1
    };
    
    {
        let mut game = context.state.game.write().await;
        let stats = game.get_or_create_player(session_id.clone());

        if !stats.is_balance_sufficient(req.bid_amount) {
            return Err(StatusCode::PAYMENT_REQUIRED);
        }

        stats
            .deduct_balance(req.bid_amount)
            .map_err(|_| StatusCode::PAYMENT_REQUIRED)?;
        
        stats.track_bid(next_slot);
    }

    if !context
        .state
        .auctions
        .read()
        .await
        .jit_auctions
        .contains_key(&next_slot)
    {
        context
            .state
            .start_jit_auction(next_slot, context.config.marketplace.base_fee_sol)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    context
        .state
        .submit_jit_bid(next_slot, session_id.clone(), req.bid_amount)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    {
        let mut marketplace = context.state.marketplace.write().await;
        if let Some(slot) = marketplace.slots.get_mut(&next_slot) {
            slot.state = crate::slot::SlotState::JiTAuction {
                current_bid: req.bid_amount,
                bidder: session_id.clone(),
            };
        }
    }

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

    Ok(Json(json!({
        "transaction_id": transaction_id,
        "slot_number": next_slot,
        "bid_amount": req.bid_amount,
        "status": "auction_pending",
        "message": "JIT bid submitted for next available slot"
    })))
}

async fn submit_aot_transaction(
    State(context): State<AppContext>,
    headers: HeaderMap,
    Json(req): Json<AotBidRequest>,
) -> Result<Json<Value>, StatusCode> {
    let session_id = get_session_from_cookie(&headers, None, &context.state.sessions).await?;

    let current_slot = context.state.get_current_slot().await;
    if req.slot_number < current_slot {
        return Err(StatusCode::BAD_REQUEST);
    }

    {
        let mut game = context.state.game.write().await;
        let stats = game.get_or_create_player(session_id.clone());

        if !stats.is_balance_sufficient(req.bid_amount) {
            return Err(StatusCode::PAYMENT_REQUIRED);
        }

        stats
            .deduct_balance(req.bid_amount)
            .map_err(|_| StatusCode::PAYMENT_REQUIRED)?;
        
        stats.track_bid(req.slot_number);
    }

    if !context
        .state
        .auctions
        .read()
        .await
        .aot_auctions
        .contains_key(&req.slot_number)
    {
        context
            .state
            .start_aot_auction(
                req.slot_number,
                context.config.marketplace.base_fee_sol,
                context.config.auction.aot_default_duration_sec,
            )
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    context
        .state
        .submit_aot_bid(req.slot_number, session_id.clone(), req.bid_amount)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?;

    {
        let mut marketplace = context.state.marketplace.write().await;
        if let Some(slot) = marketplace.slots.get_mut(&req.slot_number) {
            let auctions = context.state.auctions.read().await;
            if let Some(auction) = auctions.aot_auctions.get(&req.slot_number) {
                let ends_at = auction.ends_at;
                slot.state = crate::slot::SlotState::AoTAuction {
                    highest_bid: req.bid_amount,
                    highest_bidder: session_id.clone(),
                    bids: vec![(session_id.clone(), req.bid_amount)],
                    ends_at,
                };
            }
        }
    }

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

    Ok(Json(json!({
        "transaction_id": transaction_id,
        "slot_number": req.slot_number,
        "bid_amount": req.bid_amount,
        "status": "auction_pending",
        "message": "AOT bid submitted for future slot"
    })))
}

async fn list_transactions(
    State(context): State<AppContext>,
    headers: HeaderMap,
    Query(query): Query<TransactionQuery>,
) -> Result<Json<Value>, StatusCode> {
    let session_id = get_session_from_cookie(&headers, query.session_id.as_ref(), &context.state.sessions).await?;

    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(20).min(100).max(1);
    let offset = (page - 1) * limit;

    if query.show_all.unwrap_or(false) {
        let all_transactions = context
            .state
            .get_all_transactions_paginated(offset, limit)
            .await;
        let total_count = context.state.get_total_transaction_count().await;
        let total_pages = (total_count + limit - 1) / limit;

        return Ok(Json(json!({
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
        })));
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

    Ok(Json(json!({
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
    })))
}

async fn list_all_transactions(
    State(context): State<AppContext>,
    Query(query): Query<TransactionBatchQuery>,
) -> Json<Value> {
    let page = query.page.unwrap_or(1).max(1);
    let limit = query.limit.unwrap_or(20).min(100).max(1);
    let offset = (page - 1) * limit;

    let all_transactions = context
        .state
        .get_all_transactions_paginated(offset, limit)
        .await;
    let total_count = context.state.get_total_transaction_count().await;
    let total_pages = (total_count + limit - 1) / limit;

    Json(json!({
        "transactions": all_transactions,
        "pagination": {
            "current_page": page,
            "total_pages": total_pages,
            "page_size": limit,
            "total_count": total_count,
            "has_next": page < total_pages,
            "has_prev": page > 1
        },
        "showing": "all"
    }))
}

async fn get_transaction(
    State(context): State<AppContext>,
    Path(transaction_id): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    context
        .state
        .get_transaction(&transaction_id)
        .await
        .map(|transaction| Json(json!(transaction)))
        .ok_or(StatusCode::NOT_FOUND)
}

async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now()
    }))
}

async fn get_player_stats(
    State(context): State<AppContext>,
    headers: HeaderMap,
    Query(query): Query<TransactionQuery>,
) -> Result<Json<Value>, StatusCode> {
    let session_id = get_session_from_cookie(&headers, query.session_id.as_ref(), &context.state.sessions).await?;

    let mut game = context.state.game.write().await;
    let stats = game.get_or_create_player(session_id.clone());

    Ok(Json(json!(stats)))
}

async fn get_leaderboard(State(context): State<AppContext>) -> Json<Value> {
    let leaderboard = context.state.get_leaderboard().await;
    Json(json!(leaderboard))
}
