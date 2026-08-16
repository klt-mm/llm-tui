use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model {
    pub id: String,
    pub display_name: Option<String>,
    pub context_length: Option<u64>,
    pub metadata: serde_json::Value,
}
