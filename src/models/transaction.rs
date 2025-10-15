use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::types::InclusionType;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum TransactionStatus {
    Pending,

    Included {
        slot: u64,
        execution_time: DateTime<Utc>,
    },

    Failed {
        reason: String,
    },

    AuctionWon {
        slot: u64,
        winning_bid: f64,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Transaction {
    pub id: String,
    pub sender: String,
    pub inclusion_type: InclusionType,
    pub status: TransactionStatus,
    pub compute_units: u64,
    pub priority_fee: f64,
    pub data: String,
    pub created_at: DateTime<Utc>,
    pub included_at: Option<DateTime<Utc>>,
}

impl Transaction {
    pub fn jit(sender: String, compute_units: u64, bid_amount: f64, data: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            sender,
            inclusion_type: InclusionType::Jit,
            status: TransactionStatus::Pending,
            compute_units,
            priority_fee: bid_amount,
            data,
            created_at: Utc::now(),
            included_at: None,
        }
    }

    pub fn aot(
        sender: String,
        compute_units: u64,
        bid_amount: f64,
        reserved_slot: u64,
        data: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            sender,
            inclusion_type: InclusionType::Aot { reserved_slot },
            status: TransactionStatus::Pending,
            compute_units,
            priority_fee: bid_amount,
            data,
            created_at: Utc::now(),
            included_at: None,
        }
    }

    pub fn mark_included(&mut self, slot: u64) {
        self.status = TransactionStatus::Included {
            slot,
            execution_time: Utc::now(),
        };
        self.included_at = Some(Utc::now());
    }

    pub fn mark_failed(&mut self, reason: String) {
        self.status = TransactionStatus::Failed { reason };
    }

    pub fn mark_auction_won(&mut self, slot: u64, winning_bid: f64) {
        self.status = TransactionStatus::AuctionWon { slot, winning_bid };
    }
}
