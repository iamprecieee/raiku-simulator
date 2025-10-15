use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct JitBidRequest {
    pub session_id: Option<String>,
    pub bid_amount: f64,
    pub compute_units: u64,
    pub data: String,
}

#[derive(Deserialize, ToSchema)]
pub struct AotBidRequest {
    pub session_id: Option<String>,
    pub slot_number: u64,
    pub bid_amount: f64,
    pub compute_units: u64,
    pub data: String,
}

#[derive(Deserialize, ToSchema)]
pub struct TransactionQuery {
    pub session_id: Option<String>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
    pub show_all: Option<bool>,
}

#[derive(Deserialize, ToSchema)]
pub struct TransactionBatchQuery {
    pub page: Option<u32>,
    pub limit: Option<u32>,
}
