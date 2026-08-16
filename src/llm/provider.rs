use async_trait::async_trait;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::domain::{
    Capabilities, GenerationParameters, GenerationUsage, Message, Model, ToolDefinition,
};

#[derive(Debug, thiserror::Error)]
pub enum LlmError {
    #[error("network error: {0}")]
    Network(String),
    #[error("provider error: {0}")]
    Provider(String),
    #[error("stream error: {0}")]
    Stream(String),
}

#[derive(Debug, Clone)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<Message>,
    pub generation: GenerationParameters,
    pub tools: Option<Vec<ToolDefinition>>,
}

#[derive(Debug, Clone)]
pub struct ChatResponse {
    pub content: String,
    pub usage: GenerationUsage,
}

#[derive(Debug, Clone)]
pub enum StreamEvent {
    Delta(String),
    ReasoningDelta(String),
    ToolCall {
        id: String,
        name: String,
        arguments: String,
    },
    Usage(GenerationUsage),
    Completed,
}

#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn models(&self) -> Result<Vec<Model>, LlmError>;
    async fn capabilities(&self) -> Result<Capabilities, LlmError>;
    async fn chat(
        &self,
        request: ChatRequest,
        cancel: CancellationToken,
    ) -> Result<ChatResponse, LlmError>;
    async fn stream_chat(
        &self,
        request: ChatRequest,
        cancel: CancellationToken,
    ) -> Result<mpsc::Receiver<Result<StreamEvent, LlmError>>, LlmError>;
}
