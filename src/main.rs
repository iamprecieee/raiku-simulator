use std::time::Duration;

use axum::Router;
use tokio::net::TcpListener;
use tokio::time::interval;

use raiku_simulator::{
    api::{create_api_router, AppContext},
    config::Config,
    rate_limiter::RateLimiter,
    state::AppState,
    transaction::TransactionStatus,
    InclusionType, TransactionType,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    tracing::info!("Starting Raiku Simulator");

    let config = Config::from_env()?;
    let state = AppState::new(config.marketplace.slot_time_ms);
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
                    slot_obj.reserve(winner.clone(), bid, TransactionType::JiT);
                    slot_obj.fill(
                        winner.clone(),
                        format!("transaction_{}", current_slot),
                        200_000,
                    );
                }

                update_transaction_status(
                    &slot_state,
                    &winner,
                    current_slot,
                    bid,
                    InclusionType::JiT,
                    TransactionType::JiT,
                )
                .await;
            }

            let resolved_aot = slot_state.resolve_ready_aot_auctions(current_slot).await;
            for (slot, winner, bid, losers) in resolved_aot {
                tracing::info!(
                    "AOT auction resolved - Slot: {}, Winner: {}, Bid: {} SOL",
                    slot,
                    winner.chars().take(8).collect::<String>(),
                    bid
                );

                if let Some(slot_obj) = slot_state.marketplace.write().await.slots.get_mut(&slot) {
                    slot_obj.reserve(winner.clone(), bid, TransactionType::AoT);
                }

                update_transaction_status(
                    &slot_state,
                    &winner,
                    slot,
                    bid,
                    InclusionType::AoT {
                        reserved_slot: slot,
                    },
                    TransactionType::AoT,
                )
                .await;

                let mut game = slot_state.game.write().await;
                for loser in losers {
                    if let Some(stats) = game.player_stats.get_mut(&loser) {
                        stats.mark_auction_resolved(slot);
                    }

                    game.process_auction_loss(&loser);
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
    tracing::info!("Slot time: {}ms", config.marketplace.slot_time_ms);
    tracing::info!("Base fee: {} SOL", config.marketplace.base_fee_sol);

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await
    .map_err(|e| anyhow::anyhow!("Server error: {}", e))?;

    Ok(())
}

async fn update_transaction_status(
    state: &AppState,
    winner_session: &str,
    slot: u64,
    winning_bid: f64,
    inclusion_type: InclusionType,
    transaction_type: TransactionType,
) {
    let session_transactions = state.get_session_transactions(winner_session).await;

    for mut transaction in session_transactions {
        if transaction.inclusion_type == inclusion_type
            && matches!(transaction.status, TransactionStatus::Pending)
        {
            transaction.mark_included(slot);
            transaction.mark_auction_won(slot, winning_bid);

            state
                .update_transaction(&transaction.id, transaction.clone())
                .await;

            tracing::info!(
                "Updated transaction {} status to AuctionWon for slot {}",
                transaction.id.chars().take(8).collect::<String>(),
                slot
            );
            break;
        }
    }

    {
        let mut game = state.game.write().await;

        if let Some(stats) = game.player_stats.get_mut(winner_session) {
            stats.mark_auction_resolved(slot);
        }

        game.process_auction_win(winner_session, transaction_type);

        if let Some(stats) = game.player_stats.get(winner_session) {
            tracing::info!(
                "Player {} won auction! Level: {}, Wins: {}, Balance: {:.3} SOL",
                winner_session.chars().take(8).collect::<String>(),
                stats.level,
                stats.total_auctions_won,
                stats.balance
            );
        }
    }
}