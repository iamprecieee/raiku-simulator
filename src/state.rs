use std::{collections::HashMap, sync::Arc};

use tokio::sync::RwLock;

use crate::{
    auction::AuctionManager, 
    slot::SlotMarketplace, 
    transaction::Transaction,
    events::{EventBroadcaster, AppEvent},
    session::SessionManager
};

#[derive(Clone)]
pub struct AppState {
    pub marketplace: Arc<RwLock<SlotMarketplace>>,
    pub auctions: Arc<RwLock<AuctionManager>>,
    pub transactions: Arc<RwLock<HashMap<String, Transaction>>>,
    pub session_transactions: Arc<RwLock<HashMap<String, Vec<String>>>>,
    pub sessions: SessionManager,
    pub events: EventBroadcaster,
}

impl AppState {
    pub fn new(slot_time_ms: i64) -> Self {
        Self {
            marketplace: Arc::new(RwLock::new(SlotMarketplace::new(slot_time_ms))),
            auctions: Arc::new(RwLock::new(AuctionManager::new())),
            transactions: Arc::new(RwLock::new(HashMap::new())),
            session_transactions: Arc::new(RwLock::new(HashMap::new())),
            sessions: SessionManager::new(),
            events: EventBroadcaster::new(),
        }
    }

    pub async fn add_transaction(&self, session_id: String, transaction: Transaction) {
        let transaction_id = transaction.id.clone();

        self.transactions
            .write()
            .await
            .insert(transaction_id.clone(), transaction.clone());

        self.session_transactions
            .write()
            .await
            .entry(session_id)
            .or_insert_with(Vec::new)
            .push(transaction_id);

        self.events.broadcast(AppEvent::TransactionUpdated { transaction });
    }

    pub async fn get_session_transactions(&self, session_id: &str) -> Vec<Transaction> {
        let session_transactions = self.session_transactions.read().await;
        let transaction_ids = session_transactions
            .get(session_id)
            .cloned()
            .unwrap_or_default();

        let transactions = self.transactions.read().await;
        transaction_ids
            .iter()
            .filter_map(|id| transactions.get(id).cloned())
            .collect()
    }

    pub async fn get_session_transactions_paginated(&self, session_id: &str, offset: u32, limit: u32) -> Vec<Transaction> {
        let session_transactions = self.session_transactions.read().await;
        let transaction_ids = session_transactions
            .get(session_id)
            .cloned()
            .unwrap_or_default();

        let transactions = self.transactions.read().await;
        
        transaction_ids
            .iter()
            .skip(offset as usize)
            .take(limit as usize)
            .filter_map(|id| transactions.get(id).cloned())
            .collect()
    }

    pub async fn get_session_transaction_count(&self, session_id: &str) -> u32 {
        let session_transactions = self.session_transactions.read().await;
        session_transactions
            .get(session_id)
            .map(|ids| ids.len() as u32)
            .unwrap_or(0)
    }

    pub async fn get_transaction(&self, transaction_id: &str) -> Option<Transaction> {
        self.transactions.read().await.get(transaction_id).cloned()
    }

    pub async fn get_all_transactions_paginated(&self, offset: u32, limit: u32) -> Vec<Transaction> {
        let transactions = self.transactions.read().await;
        
        let mut all_transactions: Vec<Transaction> = transactions.values().cloned().collect();
        
        all_transactions.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        
        all_transactions
            .into_iter()
            .skip(offset as usize)
            .take(limit as usize)
            .collect()
    }

    pub async fn get_total_transaction_count(&self) -> u32 {
        self.transactions.read().await.len() as u32
    }

    pub async fn update_transaction(&self, transaction_id: &str, transaction: Transaction) {
        self.transactions
            .write()
            .await
            .insert(transaction_id.to_string(), transaction.clone());
        
        self.events.broadcast(AppEvent::TransactionUpdated { transaction });
    }

    pub async fn advance_slot(&self) -> u64 {
        let current_slot = {
            let mut marketplace = self.marketplace.write().await;
            marketplace.advance_slot();
            marketplace.current_slot
        };

        self.events.broadcast(AppEvent::SlotAdvanced { current_slot });
        
        let slots: Vec<_> = {
            let marketplace = self.marketplace.read().await;
            marketplace.slots
                .iter()
                .filter(|(slot_num, _)| **slot_num >= current_slot && **slot_num < current_slot + 50)
                .map(|(_, slot)| slot.clone())
                .collect()
        };

        self.events.broadcast(AppEvent::SlotsUpdated { slots });
        self.broadcast_stats().await;
        current_slot
    }

    pub async fn get_current_slot(&self) -> u64 {
        self.marketplace.read().await.current_slot
    }

    pub async fn get_marketplace_stats(&self) -> MarketplaceStats {
        let marketplace = self.marketplace.read().await;
        let auctions = self.auctions.read().await;

        MarketplaceStats {
            current_slot: marketplace.current_slot,
            total_slots: marketplace.slots.len(),
            active_jit_auctions: auctions.jit_auctions.len(),
            active_aot_auctions: auctions.aot_auctions.len(),
            total_transactions: self.transactions.read().await.len(),
        }
    }

    pub async fn broadcast_stats(&self) {
        let stats = self.get_marketplace_stats().await;
        
        self.events.broadcast(AppEvent::MarketplaceStats {
            current_slot: stats.current_slot,
            active_jit_auctions: stats.active_jit_auctions,
            active_aot_auctions: stats.active_aot_auctions,
            total_transactions: stats.total_transactions,
        });
    }

    pub async fn start_jit_auction(&self, slot_number: u64, base_fee: f64) -> anyhow::Result<()> {
        {
            let mut auctions = self.auctions.write().await;
            auctions.start_jit_auction(slot_number, base_fee)?;
        }

        self.events.broadcast(AppEvent::JitAuctionStarted {
            slot_number,
            min_bid: base_fee * 1.05,
        });

        Ok(())
    }

    pub async fn start_aot_auction(&self, slot_number: u64, base_fee: f64, duration_seconds: i64) -> anyhow::Result<()> {
        let ends_at = chrono::Utc::now() + chrono::Duration::seconds(duration_seconds);
        
        {
            let mut auctions = self.auctions.write().await;
            auctions.start_aot_auction(slot_number, base_fee, duration_seconds)?;
        }

        self.events.broadcast(AppEvent::AotAuctionStarted {
            slot_number,
            min_bid: base_fee,
            ends_at,
        });

        Ok(())
    }

    pub async fn submit_jit_bid(&self, slot_number: u64, bidder_id: String, amount: f64) -> anyhow::Result<()> {
        {
            let mut auctions = self.auctions.write().await;
            auctions.submit_jit_bid(slot_number, bidder_id.clone(), amount)?;
        }

        self.events.broadcast(AppEvent::JitBidSubmitted {
            slot_number,
            bidder: bidder_id,
            amount,
        });

        Ok(())
    }

    pub async fn submit_aot_bid(&self, slot_number: u64, bidder_id: String, amount: f64) -> anyhow::Result<()> {
        {
            let mut auctions = self.auctions.write().await;
            auctions.submit_aot_bid(slot_number, bidder_id.clone(), amount)?;
        }

        self.events.broadcast(AppEvent::AotBidSubmitted {
            slot_number,
            bidder: bidder_id,
            amount,
        });

        Ok(())
    }

    pub async fn resolve_jit_auction(&self, slot_number: u64) -> Option<(String, f64)> {
        let result = {
            let mut auctions = self.auctions.write().await;
            auctions.resolve_jit(slot_number)
        };

        if let Some((winner, winning_bid)) = &result {
            self.events.broadcast(AppEvent::JitAuctionResolved {
                slot_number,
                winner: winner.clone(),
                winning_bid: *winning_bid,
            });
        }

        result
    }

    pub async fn resolve_ready_aot_auctions(&self, current_slot: u64) -> Vec<(u64, String, f64)> {
        let results = {
            let mut auctions = self.auctions.write().await;
            auctions.resolve_ready_aot(current_slot)
        };

        for (slot_number, winner, winning_bid) in &results {
            self.events.broadcast(AppEvent::AotAuctionResolved {
                slot_number: *slot_number,
                winner: winner.clone(),
                winning_bid: *winning_bid,
            });
        }

        results
    }
}

#[derive(Debug, serde::Serialize)]
pub struct MarketplaceStats {
    pub current_slot: u64,
    pub total_slots: usize,
    pub active_jit_auctions: usize,
    pub active_aot_auctions: usize,
    pub total_transactions: usize,
}