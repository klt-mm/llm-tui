//! Repository contracts.
//!
//! The concrete SQLite implementations should be added in the first implementation sprint.
//! Keep these traits independent of SQLx types.

use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::{Conversation, GenerationRun, Message, Model, Prompt, Provider};

#[derive(Debug, Clone)]
pub struct MessageSearchResult {
    pub message: Message,
    pub conversation_title: String,
}

#[derive(Debug, Clone)]
pub struct PromptSearchResult {
    pub prompt: Prompt,
}

#[async_trait]
pub trait ConversationRepository: Send + Sync {
    async fn create(&self, conversation: &Conversation) -> anyhow::Result<()>;
    async fn get(&self, id: Uuid) -> anyhow::Result<Option<Conversation>>;
    async fn list(&self) -> anyhow::Result<Vec<Conversation>>;
    async fn update(&self, conversation: &Conversation) -> anyhow::Result<()>;
    async fn delete(&self, id: Uuid) -> anyhow::Result<()>;
    async fn search_titles(&self, query: &str) -> anyhow::Result<Vec<Conversation>>;
}

#[async_trait]
pub trait MessageRepository: Send + Sync {
    async fn create(&self, message: &Message) -> anyhow::Result<()>;
    async fn list_for_conversation(&self, conversation_id: Uuid) -> anyhow::Result<Vec<Message>>;
    async fn search(&self, query: &str, limit: u32) -> anyhow::Result<Vec<MessageSearchResult>>;
}

#[async_trait]
pub trait ModelRepository: Send + Sync {
    async fn upsert(&self, provider_id: Uuid, models: &[Model]) -> anyhow::Result<()>;
    async fn list_for_provider(&self, provider_id: Uuid) -> anyhow::Result<Vec<Model>>;
}

#[async_trait]
pub trait PromptRepository: Send + Sync {
    async fn create(&self, prompt: &Prompt) -> anyhow::Result<()>;
    async fn get(&self, id: Uuid) -> anyhow::Result<Option<Prompt>>;
    async fn list(&self) -> anyhow::Result<Vec<Prompt>>;
    async fn update(&self, prompt: &Prompt) -> anyhow::Result<()>;
    async fn delete(&self, id: Uuid) -> anyhow::Result<()>;
    async fn search(&self, query: &str, limit: u32) -> anyhow::Result<Vec<PromptSearchResult>>;
}

#[async_trait]
pub trait ProviderRepository: Send + Sync {
    async fn create(&self, provider: &Provider) -> anyhow::Result<()>;
    async fn list(&self) -> anyhow::Result<Vec<Provider>>;
}

#[async_trait]
pub trait GenerationRunRepository: Send + Sync {
    async fn create(&self, run: &GenerationRun) -> anyhow::Result<()>;
    async fn list_for_message(&self, message_id: Uuid) -> anyhow::Result<Vec<GenerationRun>>;
}
