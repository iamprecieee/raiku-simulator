pub mod app;
pub mod config;
pub mod managers;
pub mod middleware;
pub mod models;
pub mod routes;
pub mod services;
pub mod utils;

pub const INITIAL_PLAYER_BALANCE: f64 = 100_000.0;
pub const MAX_COMPUTE_UNITS_PER_SLOT: u64 = 48_000_000;
pub const MIN_AOT_BID_INCREMENT: f64 = 0.001;
pub const JIT_PREMIUM_MULTIPLIER: f64 = 1.05;
