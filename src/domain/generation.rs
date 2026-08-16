use serde::{Deserialize, Serialize};
use uuid::Uuid;

use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GenerationParameters {
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub max_tokens: Option<u32>,
    pub stop: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GenerationUsage {
    pub prompt_tokens: Option<u64>,
    pub completion_tokens: Option<u64>,
    pub total_tokens: Option<u64>,
    pub prompt_ms: Option<f64>,
    pub generation_ms: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationRun {
    pub id: Uuid,
    pub message_id: Uuid,
    pub provider_id: Uuid,
    pub model_id: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub prompt_tokens: Option<u64>,
    pub completion_tokens: Option<u64>,
    pub total_tokens: Option<u64>,
    pub prompt_ms: Option<f64>,
    pub generation_ms: Option<f64>,
    pub metadata: serde_json::Value,
}
