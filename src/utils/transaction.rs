use anyhow::{Result, anyhow};
use rand::Rng;

use crate::config::GlobalConfig;

pub fn calculate_base_fee() -> Result<f64> {
    let config = GlobalConfig::from_env().map_err(|e| anyhow!("Configuration error: {}", e))?;

    Ok(config.marketplace.base_fee_sol * rand::rng().random_range(1.0..10.0))
}
