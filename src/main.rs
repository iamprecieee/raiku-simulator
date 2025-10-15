use std::collections::HashMap;
use std::time::Duration;

use axum::Router;
use raiku_simulator::app::api::{AppContext, create_api_router};
use raiku_simulator::app::state::AppState;
use raiku_simulator::config::GlobalConfig;
use raiku_simulator::models::types::{InclusionType, TransactionType};
use raiku_simulator::services::transaction::{
    update_transaction_status_lose, update_transaction_status_win,
};
use raiku_simulator::utils::rate_limiter::RateLimiter;
use tokio::net::TcpListener;
use tokio::time::interval;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    tracing::info!("Starting Raiku Simulator");

    let config = GlobalConfig::from_env()?;
    let state = AppState::new(config.marketplace.slot_duration_ms);
    let rate_limiter = RateLimiter::new(100);

    let slot_state = state.clone();
    let session_state = state.clone();

    // Background task to advance slot and resolve auctions
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_millis(
            config.marketplace.advance_slot_interval_ms,
        ));

        loop {
            interval.tick().await;
            let current_slot = slot_state.advance_slot().await;

            if let Some((winner, bid)) = slot_state.resolve_jit_auction(current_slot).await {
                tracing::info!(
                    "JIT auction resolved - Slot: {}, Winner: {}, Bid: {} SOL",
                    current_slot,
                    winner.chars().take(8).collect::<String>(),
                    bid
                );

                if let Some(slot_obj) = slot_state
                    .marketplace
                    .write()
                    .await
                    .slots
                    .get_mut(&current_slot)
                {
                    slot_obj.reserve(winner.clone(), bid, TransactionType::Jit);
                    slot_obj.fill(
                        winner.clone(),
                        format!("transaction_{}", current_slot),
                        200_000,
                    );
                }

                update_transaction_status_win(
                    &slot_state,
                    &winner,
                    current_slot,
                    bid,
                    InclusionType::Jit,
                    TransactionType::Jit,
                )
                .await;
            }

            let resolved_aot = slot_state.resolve_ready_aot_auctions(current_slot).await;
            for (slot, winner, bid, losers_with_bids) in resolved_aot {
                tracing::info!(
                    "AOT auction resolved - Slot: {}, Winner: {}, Bid: {} SOL, Refunding {} losers",
                    slot,
                    winner.chars().take(8).collect::<String>(),
                    bid,
                    losers_with_bids.len()
                );

                if let Some(slot_obj) = slot_state.marketplace.write().await.slots.get_mut(&slot) {
                    slot_obj.reserve(winner.clone(), bid, TransactionType::Aot);
                }

                update_transaction_status_win(
                    &slot_state,
                    &winner,
                    slot,
                    bid,
                    InclusionType::Aot {
                        reserved_slot: slot,
                    },
                    TransactionType::Aot,
                )
                .await;

                // Group losing bids by player to process each player once
                let mut loser_totals: HashMap<String, f64> = HashMap::new();
                for (loser_id, bid_amount) in losers_with_bids {
                    *loser_totals.entry(loser_id).or_insert(0.0) += bid_amount;
                }

                let mut game = slot_state.game.write().await;

                // Loser processing with refunds
                for (loser_id, total_refund) in loser_totals {
                    if let Some(stats) = game.player_stats.get_mut(&loser_id) {
                        stats.mark_auction_resolved(slot);
                        stats.increment_balance(total_refund);

                        tracing::info!(
                            "Refunded {} SOL to {}",
                            total_refund,
                            loser_id.chars().take(8).collect::<String>()
                        );
                    }

                    drop(game); // Release the lock temporarily

                    update_transaction_status_lose(
                        &slot_state,
                        &loser_id,
                        slot,
                        InclusionType::Aot {
                            reserved_slot: slot,
                        },
                    )
                    .await;

                    game = slot_state.game.write().await; // Re-acquire the lock

                    game.process_auction_loss(&loser_id);
                }
            }
            if current_slot % 10 == 0 {
                tracing::info!("Current slot: {}", current_slot);
            }
        }
    });

    // Backgrouud task to cleanup expired sessions
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(300));

        loop {
            interval.tick().await;

            let removed_sessions = session_state.sessions.cleanup_expired_sessions().await;

            if !removed_sessions.is_empty() {
                let mut game = session_state.game.write().await;
                game.cleanup_players(&removed_sessions);

                tracing::info!(
                    "Cleaned up {} expired sessions and their player stats",
                    removed_sessions.len()
                );
            }

            let session_count = session_state.sessions.get_session_count().await;
            if session_count > 0 {
                tracing::info!("Active sessions: {}", session_count);
            }
        }
    });

    let context = AppContext {
        state: state.clone(),
        config: config.clone(),
        rate_limiter,
    };

    let app: Router = create_api_router(context);

    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = TcpListener::bind(&addr)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to bind to {}: {}", addr, e))?;

    tracing::info!("Raiku Simulator running on http://{}", addr);
    tracing::info!("Slot time: {}ms", config.marketplace.slot_duration_ms);
    tracing::info!("Base fee: {} SOL", config.marketplace.base_fee_sol);

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await
    .map_err(|e| anyhow::anyhow!("Server error: {}", e))?;

    Ok(())
}
