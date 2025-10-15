use std::collections::HashMap;

use anyhow::{Result, anyhow};

use crate::models::auction::{AotAuction, JitAuction};

#[derive(Clone, Debug, Default)]
pub struct AuctionManager {
    pub jit_auctions: HashMap<u64, JitAuction>,
    pub aot_auctions: HashMap<u64, AotAuction>,
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

        let auction = JitAuction::new(slot_number, base_fee);
        self.jit_auctions.insert(slot_number, auction);
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
            .ok_or_else(|| anyhow!("No JIT auction exists for slot {}", slot_number))?;

        auction.submit_bid(bidder_id, amount)
    }

    pub fn resolve_jit(&mut self, slot_number: u64) -> Option<(String, f64)> {
        self.jit_auctions
            .remove(&slot_number)
            .and_then(|a| a.resolve())
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

        let auction = AotAuction::new(slot_number, base_fee, duration_seconds);
        self.aot_auctions.insert(slot_number, auction);
        Ok(())
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
            .ok_or_else(|| anyhow!("No AOT auction exists for slot {}", slot_number))?;

        auction.submit_bid(bidder_id, amount)
    }

    pub fn resolve_ready_aot(&mut self, current_slot: u64) -> Vec<(u64, String, f64, Vec<String>)> {
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
                    let losers = auction.get_losers();
                    resolved.push((slot, winner, bid, losers));
                }
            }
        }

        resolved
    }

    pub fn get_active_jit_auctions(&self) -> Vec<&JitAuction> {
        self.jit_auctions.values().collect()
    }

    pub fn get_active_aot_auctions(&self) -> Vec<&AotAuction> {
        self.aot_auctions.values().collect()
    }
}
