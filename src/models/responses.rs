use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

#[derive(Deserialize, Serialize, ToSchema)]
pub struct ApiResponse {
    success: bool,
    message: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,

    #[schema(example = 200)]
    code: u32,
}

impl ApiResponse {
    pub fn success(message: String, data: Value) -> Self {
        Self {
            success: true,
            message,
            data: { if data.is_null() { None } else { Some(data) } },
            code: 200,
        }
    }

    pub fn failure(message: impl Into<String>, code: u32) -> Self {
        Self {
            success: false,
            message: message.into(),
            data: None,
            code,
        }
    }
}
