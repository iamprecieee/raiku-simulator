use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast::{channel, Sender, Receiver};

use crate::{slot::Slot, transaction::Transaction};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AppEvent {
    SlotAdvanced { 
        current_slot: u64 
    },
    SlotsUpdated { 
        slots: Vec<Slot> 
    },
    JitAuctionStarted { 
        slot_number: u64, 
        min_bid: f64 
    },
    AotAuctionStarted { 
        slot_number: u64, 
        min_bid: f64,
        ends_at: DateTime<Utc>
    },
    JitBidSubmitted { 
        slot_number: u64, 
        bidder: String, 
        amount: f64 
    },
    AotBidSubmitted { 
        slot_number: u64, 
        bidder: String, 
        amount: f64 
    },
    JitAuctionResolved { 
        slot_number: u64, 
        winner: String, 
        winning_bid: f64 
    },
    AotAuctionResolved { 
        slot_number: u64, 
        winner: String, 
        winning_bid: f64 
    },
    TransactionUpdated { 
        transaction: Transaction 
    },
    MarketplaceStats { 
        current_slot: u64,
        active_jit_auctions: usize,
        active_aot_auctions: usize,
        total_transactions: usize
    }
}

#[derive(Clone)]
pub struct EventBroadcaster {
    sender: Sender<AppEvent>,
}

impl EventBroadcaster {
    pub fn new() -> Self {
        let (sender, _) = channel(10000);
        Self { sender }
    }

    pub fn broadcast(&self, event: AppEvent) {
        let _ = self.sender.send(event);
    }

    pub fn subscribe(&self) -> Receiver<AppEvent> {
        self.sender.subscribe()
    }
}