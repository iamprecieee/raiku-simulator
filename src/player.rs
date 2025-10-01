use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::metrics::Achievement;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayerStats {
    pub session_id: String,
    pub balance: f64,
    pub total_sol_spent: f64,
    pub total_auctions_participated: u32,
    pub total_auctions_won: u32,
    pub level: u32,
    pub current_streak: u32,
    pub best_streak: u32,
    pub xp: u32,
    pub achievements: Vec<Achievement>,
    pub participated_slots: HashSet<u64>,
    pub resolved_slots: HashSet<u64>,
}

impl PlayerStats {
    pub fn new(session_id: String) -> Self {
        Self {
            session_id,
            balance: 1000.0,
            total_sol_spent: 0.0,
            total_auctions_participated: 0,
            total_auctions_won: 0,
            level: 1,
            current_streak: 0,
            best_streak: 0,
            xp: 0,
            achievements: Vec::new(),
            participated_slots: HashSet::new(),
            resolved_slots: HashSet::new(),
        }
    }

    pub fn increment_balance(&mut self, amount: f64) {
        self.balance += amount;
    }

    pub fn deduct_balance(&mut self, amount: f64) -> Result<(), String> {
        if self.is_balance_sufficient(amount) {
            self.balance -= amount;
            self.total_sol_spent += amount;
            Ok(())
        } else {
            Err(format!(
                "Insufficient balance. Have: {}, Need: {}",
                self.balance, amount
            ))
        }
    }

    pub fn is_balance_sufficient(&self, amount: f64) -> bool {
        self.balance >= amount
    }

    pub fn win_rate(&self) -> f64 {
        if self.total_auctions_participated == 0 {
            0.0
        } else {
            (self.total_auctions_won as f64 / self.total_auctions_participated as f64) * 100.0
        }
    }

    pub fn add_xp(&mut self, amount: u32) {
        self.xp += amount;
        self.check_level_up();
    }

    fn check_level_up(&mut self) {
        let required_xp = self.level * 100;
        if self.xp >= required_xp {
            self.level += 1;
            self.xp -= required_xp;
        }
    }

    pub fn track_bid(&mut self, slot_number: u64) {
        self.participated_slots.insert(slot_number);
    }

    pub fn mark_auction_resolved(&mut self, slot_number: u64) {
        if self.participated_slots.contains(&slot_number) 
            && self.resolved_slots.insert(slot_number) {
            self.total_auctions_participated += 1;
        }
    }
}
