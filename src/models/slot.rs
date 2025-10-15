use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::models::types::TransactionType;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum SlotState {
    Available,

    JitAuction {
        current_bid: f64,
        bidder: String,
    },

    AotAuction {
        highest_bid: f64,
        highest_bidder: String,
        bids: Vec<(String, f64)>,
        ends_at: DateTime<Utc>,
    },

    Reserved {
        winner: String,
        winning_bid: f64,
        transaction_type: TransactionType,
    },

    Filled {
        winner: String,
        transaction_id: String,
        execution_time: DateTime<Utc>,
    },

    Expired,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Slot {
    pub slot_number: u64,
    pub state: SlotState,
    pub estimated_time: DateTime<Utc>,
    pub base_fee: f64,
    pub compute_units_available: u64,
    pub compute_units_used: u64,
    pub created_at: DateTime<Utc>,
}

impl Slot {
    pub fn new(slot_number: u64, estimated_time: DateTime<Utc>, base_fee: f64) -> Self {
        Self {
            slot_number,
            state: SlotState::Available,
            estimated_time,
            base_fee,
            compute_units_available: 48_000_000,
            compute_units_used: 0,
            created_at: Utc::now(),
        }
    }

    pub fn is_available(&self) -> bool {
        matches!(self.state, SlotState::Available)
    }

    pub fn is_expired(&self) -> bool {
        self.estimated_time < Utc::now()
    }

    pub fn reserve(&mut self, winner: String, winning_bid: f64, transaction_type: TransactionType) {
        self.state = SlotState::Reserved {
            winner,
            winning_bid,
            transaction_type,
        }
    }

    pub fn fill(&mut self, winner: String, transaction_id: String, compute_units_used: u64) {
        self.compute_units_used += compute_units_used;
        self.state = SlotState::Filled {
            winner,
            transaction_id,
            execution_time: Utc::now(),
        }
    }
}
