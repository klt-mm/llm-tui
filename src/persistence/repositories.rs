//! Repository contracts.
//!
//! The concrete SQLite implementations should be added in the first implementation sprint.
//! Keep these traits independent of SQLx types.

use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::{Conversation, Message, Prompt, Provider};

#[async_trait]
pub trait ConversationRepository {
    async fn create(&self, conversation: &Conversation) -> anyhow::Result<()>;
    async fn get(&self, id: Uuid) -> anyhow::Result<Option<Conversation>>;
    async fn list(&self) -> anyhow::Result<Vec<Conversation>>;
}

#[async_trait]
pub trait MessageRepository {
    async fn create(&self, message: &Message) -> anyhow::Result<()>;
    async fn list_for_conversation(&self, conversation_id: Uuid) -> anyhow::Result<Vec<Message>>;
}

#[async_trait]
pub trait PromptRepository {
    async fn create(&self, prompt: &Prompt) -> anyhow::Result<()>;
    async fn get(&self, id: Uuid) -> anyhow::Result<Option<Prompt>>;
}

#[async_trait]
pub trait ProviderRepository {
    async fn create(&self, provider: &Provider) -> anyhow::Result<()>;
    async fn list(&self) -> anyhow::Result<Vec<Provider>>;
}
