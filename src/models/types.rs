use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum TransactionType {
    Jit,
    Aot,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum InclusionType {
    Jit,
    Aot { reserved_slot: u64 },
}
