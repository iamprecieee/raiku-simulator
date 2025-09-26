use std::collections::HashMap;

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::TransactionType;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Bid {
    pub bidder_id: String,
    pub amount: f64,
    pub slot_number: u64,
    pub timestamp: DateTime<Utc>,
    pub bid_type: TransactionType,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct JiTAuction {
    pub slot_number: u64,
    pub min_bid: f64,
    pub current_winner: Option<(String, f64)>,
    pub created_at: DateTime<Utc>,
}

impl JiTAuction {
    pub fn new(slot_number: u64, base_fee: f64) -> Self {
        Self {
            slot_number,
            min_bid: base_fee * 1.05,
            current_winner: None,
            created_at: Utc::now(),
        }
    }

    pub fn submit_bid(&mut self, bidder_id: String, amount: f64) -> Result<()> {
        if amount < self.min_bid {
            return Err(anyhow!(
                "Bid too low. Minimum: {} SOL, provided: {} SOL",
                self.min_bid,
                amount
            ));
        }

        match &self.current_winner {
            None => {
                self.current_winner = Some((bidder_id, amount));
                Ok(())
            }
            Some((_, current_amount)) => {
                if amount > *current_amount {
                    self.current_winner = Some((bidder_id, amount));
                    Ok(())
                } else {
                    Err(anyhow!(
                        "Bid must exceed current highest bid of {} SOL",
                        current_amount
                    ))
                }
            }
        }
    }

    pub fn resolve(&self) -> Option<(String, f64)> {
        self.current_winner.clone()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AoTAuction {
    pub slot_number: u64,
    pub min_bid: f64,
    pub bids: Vec<(String, f64, DateTime<Utc>)>,
    pub ends_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl AoTAuction {
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
        if Utc::now() > self.ends_at {
            return Err(anyhow!("Auction has ended"));
        }

        let min_required = self.get_min_next_bid();
        if amount < min_required {
            return Err(anyhow!(
                "Bid too low. Minimum: {} SOL, provided: {} SOL",
                min_required,
                amount
            ));
        }

        self.bids.push((bidder_id, amount, Utc::now()));
        Ok(())
    }

    pub fn get_min_next_bid(&self) -> f64 {
        match self.get_highest_bid() {
            Some((_, amount, _)) => amount + 0.0001,
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
}

#[derive(Clone, Debug, Default)]
pub struct AuctionManager {
    pub jit_auctions: HashMap<u64, JiTAuction>,
    pub aot_auctions: HashMap<u64, AoTAuction>,
}

impl AuctionManager {
    pub fn new() -> Self {
        Self {
            jit_auctions: HashMap::new(),
            aot_auctions: HashMap::new(),
        }
    }

    pub fn start_jit_auction(&mut self, slot_number: u64, base_fee: f64) -> Result<()> {
        if self.jit_auctions.contains_key(&slot_number) {
            return Err(anyhow!(
                "JIT auction already exists for slot {}",
                slot_number
            ));
        }

        let auction = JiTAuction::new(slot_number, base_fee);
        self.jit_auctions.insert(slot_number, auction);
        Ok(())
    }

    pub fn start_aot_auction(
        &mut self,
        slot_number: u64,
        base_fee: f64,
        duration_seconds: i64,
    ) -> Result<()> {
        if self.aot_auctions.contains_key(&slot_number) {
            return Err(anyhow!(
                "AOT auction already exists for slot {}",
                slot_number
            ));
        }

        let auction = AoTAuction::new(slot_number, base_fee, duration_seconds);
        self.aot_auctions.insert(slot_number, auction);
        Ok(())
    }

    pub fn submit_jit_bid(
        &mut self,
        slot_number: u64,
        bidder_id: String,
        amount: f64,
    ) -> Result<()> {
        let auction = self
            .jit_auctions
            .get_mut(&slot_number)
            .ok_or_else(|| anyhow!("No JIT auction for slot {}", slot_number))?;

        auction.submit_bid(bidder_id, amount)
    }

    pub fn submit_aot_bid(
        &mut self,
        slot_number: u64,
        bidder_id: String,
        amount: f64,
    ) -> Result<()> {
        let auction = self
            .aot_auctions
            .get_mut(&slot_number)
            .ok_or_else(|| anyhow!("No AOT auction for slot {}", slot_number))?;

        auction.submit_bid(bidder_id, amount)
    }

    pub fn resolve_jit(&mut self, slot_number: u64) -> Option<(String, f64)> {
        self.jit_auctions
            .remove(&slot_number)
            .and_then(|a| a.resolve())
    }

    pub fn resolve_ready_aot(&mut self, current_slot: u64) -> Vec<(u64, String, f64)> {
        let mut resolved = Vec::new();
        let ready_slots: Vec<u64> = self
            .aot_auctions
            .iter()
            .filter(|(_, auction)| auction.should_resolve(current_slot))
            .map(|(slot, _)| *slot)
            .collect();

        for slot in ready_slots {
            if let Some(auction) = self.aot_auctions.remove(&slot) {
                if let Some((winner, bid)) = auction.resolve() {
                    resolved.push((slot, winner, bid));
                }
            }
        }

        resolved
    }

    pub fn get_active_jit_auctions(&self) -> Vec<&JiTAuction> {
        self.jit_auctions.values().collect()
    }

    pub fn get_active_aot_auctions(&self) -> Vec<&AoTAuction> {
        self.aot_auctions.values().collect()
    }
}