use std::collections::HashMap;

use chrono::{DateTime, Duration, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::TransactionType;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum SlotState {
    Available,
    JiTAuction {
        current_bid: f64,
        bidder: String,
    },
    AoTAuction {
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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SlotMarketplace {
    pub slots: HashMap<u64, Slot>,
    pub current_slot: u64,
    pub slot_time_ms: i64,
}

impl SlotMarketplace {
    pub fn new(slot_time_ms: i64) -> Self {
        let mut marketplace = Self {
            slots: HashMap::new(),
            current_slot: 0,
            slot_time_ms,
        };

        marketplace.initialise_slots(100);
        marketplace
    }

    fn initialise_slots(&mut self, count: u64) {
        for i in 0..count {
            let slot_number = self.current_slot + i;
            let estimated_time = Utc::now() + Duration::milliseconds(self.slot_time_ms * i as i64);
            let base_fee = calculate_base_fee();

            let slot = Slot::new(slot_number, estimated_time, base_fee);
            self.slots.insert(slot_number, slot);
        }
    }

    pub fn advance_slot(&mut self) {
        self.current_slot += 1;

        for slot in self.slots.values_mut() {
            if slot.is_expired()
                && !matches!(slot.state, SlotState::Expired | SlotState::Filled { .. })
            {
                slot.state = SlotState::Expired;
            }
        }

        let furthest_slot = self.current_slot + 100;
        if !self.slots.contains_key(&furthest_slot) {
            let estimated_time = Utc::now() + Duration::milliseconds(self.slot_time_ms * 100);
            let slot = Slot::new(
                furthest_slot,
                estimated_time,
                calculate_base_fee(),
            );
            self.slots.insert(furthest_slot, slot);
        }
    }

    pub fn get_next_available_slot(&self) -> Option<u64> {
        (self.current_slot..self.current_slot + 100).find(|&slot_num| {
            self.slots
                .get(&slot_num)
                .map(|s| s.is_available())
                .unwrap_or(false)
        })
    }
}

fn calculate_base_fee() -> f64 {
    0.001 * rand::rng().random_range(1.0..10.0)
}
