use std::collections::HashMap;

use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    models::slot::{Slot, SlotState},
    utils::transaction::calculate_base_fee,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SlotMarketplace {
    pub slots: HashMap<u64, Slot>,
    pub current_slot: u64,
    pub slot_duration_ms: i64,
}

impl SlotMarketplace {
    pub fn new(slot_duration_ms: i64) -> Self {
        let mut marketplace = Self {
            slots: HashMap::new(),
            current_slot: 0,
            slot_duration_ms,
        };

        // Initializes a rolling window of slots
        marketplace.initialize_slots(100);
        marketplace
    }

    fn initialize_slots(&mut self, num_slots_ahead: u64) {
        for i in 0..num_slots_ahead {
            let slot_number = self.current_slot + i;
            let estimated_time =
                Utc::now() + Duration::milliseconds(self.slot_duration_ms * i as i64);

            let base_fee = calculate_base_fee().unwrap_or(0.001);

            let slot = Slot::new(slot_number, estimated_time, base_fee);
            self.slots.insert(slot_number, slot);
        }
    }

    /// Advances to the next slot and expires old slots
    pub fn advance_slot(&mut self) {
        self.current_slot += 1;

        for slot in self.slots.values_mut() {
            if slot.is_expired()
                && !matches!(slot.state, SlotState::Expired | SlotState::Filled { .. })
            {
                slot.state = SlotState::Expired;
            }
        }

        // Create the next slot in the rolling window
        let furthest_slot = self.current_slot + 100;
        if !self.slots.contains_key(&furthest_slot) {
            let estimated_time = Utc::now() + Duration::milliseconds(self.slot_duration_ms * 100);

            let base_fee = calculate_base_fee().unwrap_or(0.001);

            let slot = Slot::new(furthest_slot, estimated_time, base_fee);
            self.slots.insert(furthest_slot, slot);
        }
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
