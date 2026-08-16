use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProviderProtocol {
    OpenAiCompatible,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provider {
    pub id: Uuid,
    pub name: String,
    pub base_url: String,
    pub protocol: ProviderProtocol,
    pub api_key_ref: Option<String>,
    pub default_model: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Capabilities {
    pub streaming: bool,
    pub tools: bool,
    pub vision: bool,
    pub reasoning: bool,
    pub embeddings: bool,
    pub responses_api: bool,
    // Phase 6 extensions
    pub tool_calling: bool,
    pub parallel_tool_calls: bool,
    pub image_input: bool,
    pub image_formats: Vec<String>,
    pub structured_output: bool,
    pub json_mode: bool,
    pub max_output_tokens: Option<u32>,
}

impl Capabilities {
    pub fn supports_feature(&self, feature: &str) -> bool {
        match feature {
            "streaming" => self.streaming,
            "tools" | "tool_calling" => self.tool_calling || self.tools,
            "parallel_tool_calls" => self.parallel_tool_calls,
            "vision" | "image_input" => self.image_input || self.vision,
            "reasoning" => self.reasoning,
            "embeddings" => self.embeddings,
            "structured_output" => self.structured_output,
            "json_mode" => self.json_mode || self.structured_output,
            _ => false,
        }
    }
}
