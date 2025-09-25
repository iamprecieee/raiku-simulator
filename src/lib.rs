use serde::{Deserialize, Serialize};

pub mod slot;
pub mod transaction;
pub mod auction;
pub mod state;
pub mod events;
pub mod session;
pub mod rate_limiter;
pub mod config;
pub mod api;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum InclusionType {
    JiT,
    AoT { reserved_slot: u64 },
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum TransactionType {
    JiT,
    AoT,
}