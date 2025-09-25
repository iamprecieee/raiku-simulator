use std::env;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub server: ServerConfig,
    pub marketplace: MarketplaceConfig,
    pub auction: AuctionConfig,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub cors_origins: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MarketplaceConfig {
    pub slot_time_ms: i64,
    pub base_fee_sol: f64,
    pub advance_slot_interval_ms: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuctionConfig {
    pub aot_default_duration_sec: i64,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        dotenv::dotenv().ok();

        Ok(Config {
            server: ServerConfig {
                host: env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
                port: env::var("PORT")
                    .or_else(|_| env::var("SERVER_PORT"))
                    .unwrap_or_else(|_| "8080".to_string())
                    .parse()
                    .unwrap_or(8080),
                cors_origins: env::var("CORS_ORIGINS")
                    .unwrap_or_else(|_| "http://localhost:3000".to_string())
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect(),
            },
            marketplace: MarketplaceConfig {
                slot_time_ms: env::var("SLOT_TIME_MS")
                    .unwrap_or_else(|_| "400".to_string())
                    .parse()
                    .unwrap_or(400),
                base_fee_sol: env::var("BASE_FEE_SOL")
                    .unwrap_or_else(|_| "0.001".to_string())
                    .parse()
                    .unwrap_or(0.001),
                advance_slot_interval_ms: env::var("ADVANCE_SLOT_INTERVAL_MS")
                    .unwrap_or_else(|_| "400".to_string())
                    .parse()
                    .unwrap_or(400),
            },
            auction: AuctionConfig {
                aot_default_duration_sec: env::var("AOT_DURATION_SEC")
                    .unwrap_or_else(|_| "35".to_string())
                    .parse()
                    .unwrap_or(5),
            },
        })
    }
}