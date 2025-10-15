use anyhow::{Result, anyhow};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{JIT_PREMIUM_MULTIPLIER, MIN_AOT_BID_INCREMENT, models::types::TransactionType};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Bid {
    pub bidder_id: String,
    pub amount: f64,
    pub slot_number: u64,
    pub timestamp: DateTime<Utc>,
    pub bid_type: TransactionType,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct JitAuction {
    pub slot_number: u64,
    pub min_bid: f64,
    pub current_highest_bidder: Option<(String, f64)>,
    pub created_at: DateTime<Utc>,
}

impl JitAuction {
    pub fn new(slot_number: u64, base_fee: f64) -> Self {
        Self {
            slot_number,
            min_bid: base_fee * JIT_PREMIUM_MULTIPLIER,
            current_highest_bidder: None,
            created_at: Utc::now(),
        }
    }

    pub fn submit_bid(&mut self, bidder_id: String, amount: f64) -> Result<()> {
        if amount < self.min_bid {
            return Err(anyhow!("Bid too low for JIT auction",));
        }

        // Check against current highest bidder
        match &self.current_highest_bidder {
            None => {
                self.current_highest_bidder = Some((bidder_id, amount));
                Ok(())
            }
            Some((_current_highest_bidder, current_amount)) => {
                if amount > *current_amount {
                    self.current_highest_bidder = Some((bidder_id, amount));
                    Ok(())
                } else {
                    Err(anyhow!(
                        "Bid must exceed current highest bid of {:.4} SOL",
                        current_amount,
                    ))
                }
            }
        }
    }

    pub fn resolve(&self) -> Option<(String, f64)> {
        self.current_highest_bidder.clone()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AotAuction {
    pub slot_number: u64,
    pub min_bid: f64,
    pub bids: Vec<(String, f64, DateTime<Utc>)>,
    pub ends_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl AotAuction {
    pub fn new(slot_number: u64, base_fee: f64, duration_seconds: i64) -> Self {
        Self {
            slot_number,
            min_bid: base_fee,
            bids: Vec::new(),
            ends_at: Utc::now() + chrono::Duration::seconds(duration_seconds),
            created_at: Utc::now(),
        }
    }

    pub fn submit_bid(&mut self, bidder_id: String, amount: f64) -> Result<()> {
        if self.has_ended() {
            return Err(anyhow!(
                "AOT auction for slot {} has ended. Closed at: {}",
                self.slot_number,
                self.ends_at.format("%H:%M:%S UTC")
            ));
        }

        let min_required = self.get_min_next_bid();
        if amount < min_required {
            return Err(anyhow!("Bid too low for AOT auction",));
        }

        // Note: users can bid multiple times
        self.bids.push((bidder_id, amount, Utc::now()));
        Ok(())
    }

    pub fn get_min_next_bid(&self) -> f64 {
        match self.get_highest_bid() {
            Some((_, amount, _)) => amount + MIN_AOT_BID_INCREMENT,
            None => self.min_bid,
        }
    }

    pub fn get_highest_bid(&self) -> Option<&(String, f64, DateTime<Utc>)> {
        self.bids
            .iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
    }

    pub fn has_ended(&self) -> bool {
        Utc::now() > self.ends_at
    }

    pub fn should_resolve(&self, current_slot: u64) -> bool {
        self.has_ended() || self.slot_number <= current_slot
    }

    pub fn resolve(&self) -> Option<(String, f64)> {
        self.get_highest_bid()
            .map(|(bidder, amount, _)| (bidder.clone(), *amount))
    }

    // Get a list of all losing bidders for refund processing
    pub fn get_losers(&self) -> Vec<String> {
        if let Some((winner, _, _)) = self.get_highest_bid() {
            self.bids
                .iter()
                .map(|(bidder, _, _)| bidder.clone())
                .filter(|bidder| bidder != winner)
                .collect()
        } else {
            // No winner means everyone gets refunds
            self.bids
                .iter()
                .map(|(bidder, _, _)| bidder.clone())
                .collect()
        }
    }
}
