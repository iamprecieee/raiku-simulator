use axum::{
    Router,
    routing::{get, post},
};
use tower_http::cors::CorsLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    app::state::AppState,
    config::GlobalConfig,
    middleware::rate_limiter::rate_limit_middleware,
    routes::{
        auction::{list_aot_auctions, list_jit_auctions},
        event::sse_handler,
        health::health_check,
        session::create_or_validate_session,
        slot::{get_slot, list_slots},
        stats::{get_leaderboard, get_player_stats, marketplace_status},
        transaction::{
            get_transaction, list_transactions, submit_aot_transaction, submit_jit_transaction,
        },
    },
    utils::rate_limiter::RateLimiter,
};

#[derive(Clone)]
pub struct AppContext {
    pub state: AppState,
    pub config: GlobalConfig,
    pub rate_limiter: RateLimiter,
}

#[derive(OpenApi)]
#[openapi(
    info(title = "Raiku Simulator Backend API", version = "1.0.0"),
    paths(
        crate::routes::health::health_check,
        crate::routes::event::sse_handler,
        crate::routes::session::create_or_validate_session,
        crate::routes::slot::list_slots,
        crate::routes::slot::get_slot,
        crate::routes::stats::get_player_stats,
        crate::routes::stats::get_leaderboard,
        crate::routes::stats::marketplace_status,
        crate::routes::auction::list_aot_auctions,
        crate::routes::auction::list_jit_auctions,
        crate::routes::transaction::submit_aot_transaction,
        crate::routes::transaction::submit_jit_transaction,
        crate::routes::transaction::list_transactions,
        crate::routes::transaction::get_transaction,
    ),
    components(schemas(crate::models::responses::ApiResponse,),)
)]
struct ApiDoc;

pub fn create_api_router(context: AppContext) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(
            context
                .config
                .server
                .cors_allowed_origins
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
        .route("/transactions/{transaction_id}", get(get_transaction))
        .route("/health", get(health_check))
        .route("/game/player_stats", get(get_player_stats))
        .route("/game/leaderboard", get(get_leaderboard))
        .merge(SwaggerUi::new("/swagger-ui").url("/docs/openapi.json", ApiDoc::openapi()))
        .layer(axum::middleware::from_fn(rate_limit_middleware))
        .layer(axum::Extension(context.rate_limiter.clone()))
        .layer(cors)
        .with_state(context)
}
