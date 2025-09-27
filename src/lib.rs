use serde::{Deserialize, Serialize};

pub mod api;
pub mod auction;
pub mod config;
pub mod events;
pub mod rate_limiter;
pub mod session;
pub mod slot;
pub mod state;
pub mod transaction;

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
