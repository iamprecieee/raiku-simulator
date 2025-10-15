use crate::{
    app::state::AppState,
    models::{
        transaction::TransactionStatus,
        types::{InclusionType, TransactionType},
    },
};

pub async fn update_transaction_status_win(
    state: &AppState,
    winner_session: &str,
    slot: u64,
    winning_bid: f64,
    inclusion_type: InclusionType,
    transaction_type: TransactionType,
) {
    let session_transactions = state.get_session_transactions(winner_session).await;

    let mut refund_total = 0.0;

    // Process all transactions for the winning session
    for mut transaction in session_transactions {
        if transaction.inclusion_type == inclusion_type
            && matches!(transaction.status, TransactionStatus::Pending)
        {
            if (transaction.priority_fee - winning_bid).abs() < 0.0001 {
                transaction.mark_included(slot);
                transaction.mark_auction_won(slot, winning_bid);
                
                state
                    .update_transaction_by_id(&transaction.id, transaction.clone())
                    .await;
                
                tracing::info!(
                    "Updated transaction {} status to AuctionWon for slot {} with bid {} SOL",
                    transaction.id.chars().take(8).collect::<String>(),
                    slot,
                    winning_bid
                );
        } else {
                transaction.mark_failed(format!(
                    "Outbid by higher amount. Refunding {} SOL",
                    transaction.priority_fee
                ));
                
                state
                    .update_transaction_by_id(&transaction.id, transaction.clone())
                    .await;
                
                refund_total += transaction.priority_fee;
                
                tracing::info!(
                    "Marked transaction {} as failed and queued {} SOL for refund",
                    transaction.id.chars().take(8).collect::<String>(),
                    transaction.priority_fee
                );
            }
        }
    }

    // Refund all outbid amounts at once
    if refund_total > 0.0 {
        let mut game = state.game.write().await;
        if let Some(stats) = game.player_stats.get_mut(winner_session) {
            stats.increment_balance(refund_total);
            tracing::info!(
                "Refunded total {} SOL to winner {} for outbid transactions",
                refund_total,
                winner_session.chars().take(8).collect::<String>()
            );
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

pub async fn update_transaction_status_lose(
    state: &AppState,
    loser_session: &str,
    slot: u64,
    inclusion_type: InclusionType,
) {
    let session_transactions = state.get_session_transactions(loser_session).await;

    for mut transaction in session_transactions {
        if transaction.inclusion_type == inclusion_type
            && matches!(transaction.status, TransactionStatus::Pending)
        {
            transaction.mark_failed(format!("Lost auction for slot {}", slot));

            state
                .update_transaction_by_id(&transaction.id, transaction.clone())
                .await;

            tracing::info!(
                "Updated transaction {} status to Failed (auction lost) for slot {}",
                transaction.id.chars().take(8).collect::<String>(),
                slot
            );
        }
    }
}
