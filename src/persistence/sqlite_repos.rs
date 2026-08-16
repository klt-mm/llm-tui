use async_trait::async_trait;
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

use crate::domain::{Conversation, Message, Model, Prompt, Provider, ProviderProtocol, Role};
use crate::persistence::repositories::{
    ConversationRepository, MessageRepository, MessageSearchResult, ModelRepository,
    PromptRepository, PromptSearchResult, ProviderRepository,
};

// ---------------------------------------------------------------------------
// Provider
// ---------------------------------------------------------------------------

pub struct SqliteProviderRepository {
    pool: SqlitePool,
}

impl SqliteProviderRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ProviderRepository for SqliteProviderRepository {
    async fn create(&self, provider: &Provider) -> anyhow::Result<()> {
        let protocol = match provider.protocol {
            ProviderProtocol::OpenAiCompatible => "openai_compatible",
        };
        sqlx::query(
            "INSERT INTO providers (id, name, base_url, protocol, api_key_ref, default_model, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(provider.id.to_string())
        .bind(&provider.name)
        .bind(&provider.base_url)
        .bind(protocol)
        .bind(&provider.api_key_ref)
        .bind(&provider.default_model)
        .bind(provider.created_at.to_rfc3339())
        .bind(provider.updated_at.to_rfc3339())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn list(&self) -> anyhow::Result<Vec<Provider>> {
        let rows = sqlx::query("SELECT * FROM providers ORDER BY created_at")
            .fetch_all(&self.pool)
            .await?;

        rows.iter().map(row_to_provider).collect()
    }
}

fn row_to_provider(row: &sqlx::sqlite::SqliteRow) -> anyhow::Result<Provider> {
    let _protocol_str: String = row.try_get("protocol")?;
    let protocol = ProviderProtocol::OpenAiCompatible;
    Ok(Provider {
        id: Uuid::parse_str(&row.try_get::<String, _>("id")?)?,
        name: row.try_get("name")?,
        base_url: row.try_get("base_url")?,
        protocol,
        api_key_ref: row.try_get("api_key_ref")?,
        default_model: row.try_get("default_model")?,
        created_at: chrono::DateTime::parse_from_rfc3339(&row.try_get::<String, _>("created_at")?)
            .map(|dt| dt.with_timezone(&chrono::Utc))?,
        updated_at: chrono::DateTime::parse_from_rfc3339(&row.try_get::<String, _>("updated_at")?)
            .map(|dt| dt.with_timezone(&chrono::Utc))?,
    })
}

// ---------------------------------------------------------------------------
// Conversation
// ---------------------------------------------------------------------------

pub struct SqliteConversationRepository {
    pool: SqlitePool,
}

impl SqliteConversationRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ConversationRepository for SqliteConversationRepository {
    async fn create(&self, conversation: &Conversation) -> anyhow::Result<()> {
        sqlx::query(
            "INSERT INTO conversations (id, title, provider_id, model_id, system_prompt, created_at, updated_at, archived_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(conversation.id.to_string())
        .bind(&conversation.title)
        .bind(conversation.provider_id.to_string())
        .bind(&conversation.model_id)
        .bind(&conversation.system_prompt)
        .bind(conversation.created_at.to_rfc3339())
        .bind(conversation.updated_at.to_rfc3339())
        .bind(conversation.archived_at.map(|t| t.to_rfc3339()))
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn get(&self, id: Uuid) -> anyhow::Result<Option<Conversation>> {
        let row = sqlx::query("SELECT * FROM conversations WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await?;
        row.as_ref().map(row_to_conversation).transpose()
    }

    async fn list(&self) -> anyhow::Result<Vec<Conversation>> {
        let rows = sqlx::query(
            "SELECT * FROM conversations WHERE archived_at IS NULL ORDER BY updated_at DESC",
        )
        .fetch_all(&self.pool)
        .await?;
        rows.iter().map(row_to_conversation).collect()
    }

    async fn update(&self, conversation: &Conversation) -> anyhow::Result<()> {
        sqlx::query("UPDATE conversations SET title = ?, updated_at = ? WHERE id = ?")
            .bind(&conversation.title)
            .bind(conversation.updated_at.to_rfc3339())
            .bind(conversation.id.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn delete(&self, id: Uuid) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM conversations WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn search_titles(&self, query: &str) -> anyhow::Result<Vec<Conversation>> {
        let pattern = format!("%{}%", query.to_lowercase());
        let rows = sqlx::query(
            "SELECT * FROM conversations WHERE archived_at IS NULL AND LOWER(title) LIKE ? ORDER BY updated_at DESC",
        )
        .bind(&pattern)
        .fetch_all(&self.pool)
        .await?;
        rows.iter().map(row_to_conversation).collect()
    }
}

fn row_to_conversation(row: &sqlx::sqlite::SqliteRow) -> anyhow::Result<Conversation> {
    let archived_at: Option<String> = row.try_get("archived_at")?;
    Ok(Conversation {
        id: Uuid::parse_str(&row.try_get::<String, _>("id")?)?,
        title: row.try_get("title")?,
        provider_id: Uuid::parse_str(&row.try_get::<String, _>("provider_id")?)?,
        model_id: row.try_get("model_id")?,
        system_prompt: row.try_get("system_prompt")?,
        created_at: chrono::DateTime::parse_from_rfc3339(&row.try_get::<String, _>("created_at")?)
            .map(|dt| dt.with_timezone(&chrono::Utc))?,
        updated_at: chrono::DateTime::parse_from_rfc3339(&row.try_get::<String, _>("updated_at")?)
            .map(|dt| dt.with_timezone(&chrono::Utc))?,
        archived_at: archived_at
            .map(|s| {
                chrono::DateTime::parse_from_rfc3339(&s).map(|dt| dt.with_timezone(&chrono::Utc))
            })
            .transpose()?,
    })
}

// ---------------------------------------------------------------------------
// Message
// ---------------------------------------------------------------------------

pub struct SqliteMessageRepository {
    pool: SqlitePool,
}

impl SqliteMessageRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl MessageRepository for SqliteMessageRepository {
    async fn create(&self, message: &Message) -> anyhow::Result<()> {
        let role = match message.role {
            Role::System => "system",
            Role::User => "user",
            Role::Assistant => "assistant",
            Role::Tool => "tool",
        };
        sqlx::query(
            "INSERT INTO messages (id, conversation_id, parent_id, role, content, reasoning_content, metadata_json, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(message.id.to_string())
        .bind(message.conversation_id.to_string())
        .bind(message.parent_id.map(|id| id.to_string()))
        .bind(role)
        .bind(&message.content)
        .bind(&message.reasoning_content)
        .bind(message.metadata.to_string())
        .bind(message.created_at.to_rfc3339())
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "INSERT INTO messages_fts (message_id, conversation_id, content) VALUES (?, ?, ?)",
        )
        .bind(message.id.to_string())
        .bind(message.conversation_id.to_string())
        .bind(&message.content)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn list_for_conversation(&self, conversation_id: Uuid) -> anyhow::Result<Vec<Message>> {
        let rows =
            sqlx::query("SELECT * FROM messages WHERE conversation_id = ? ORDER BY created_at")
                .bind(conversation_id.to_string())
                .fetch_all(&self.pool)
                .await?;
        rows.iter().map(row_to_message).collect()
    }

    async fn search(&self, query: &str, limit: u32) -> anyhow::Result<Vec<MessageSearchResult>> {
        if query.trim().is_empty() {
            return Ok(Vec::new());
        }
        let fts_query = query
            .split_whitespace()
            .map(|w| format!("\"{}\"", w.replace('"', "")))
            .collect::<Vec<_>>()
            .join(" ");
        let rows = sqlx::query(
            "SELECT m.*, c.title AS conversation_title \
             FROM messages_fts f \
             JOIN messages m ON m.id = f.message_id \
             JOIN conversations c ON c.id = m.conversation_id \
             WHERE messages_fts MATCH ? \
             ORDER BY rank \
             LIMIT ?",
        )
        .bind(&fts_query)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await?;

        rows.iter()
            .map(|row| {
                let title: String = row.try_get("conversation_title")?;
                let message = row_to_message(row)?;
                Ok(MessageSearchResult {
                    message,
                    conversation_title: title,
                })
            })
            .collect()
    }
}

fn row_to_message(row: &sqlx::sqlite::SqliteRow) -> anyhow::Result<Message> {
    let role_str: String = row.try_get("role")?;
    let role = match role_str.as_str() {
        "system" => Role::System,
        "user" => Role::User,
        "assistant" => Role::Assistant,
        "tool" => Role::Tool,
        other => anyhow::bail!("unknown role: {other}"),
    };
    let parent_id: Option<String> = row.try_get("parent_id")?;
    let metadata_str: String = row.try_get("metadata_json")?;
    let metadata: serde_json::Value =
        serde_json::from_str(&metadata_str).unwrap_or(serde_json::json!({}));
    Ok(Message {
        id: Uuid::parse_str(&row.try_get::<String, _>("id")?)?,
        conversation_id: Uuid::parse_str(&row.try_get::<String, _>("conversation_id")?)?,
        parent_id: parent_id.map(|s| Uuid::parse_str(&s)).transpose()?,
        role,
        content: row.try_get("content")?,
        reasoning_content: row.try_get("reasoning_content")?,
        metadata,
        created_at: chrono::DateTime::parse_from_rfc3339(&row.try_get::<String, _>("created_at")?)
            .map(|dt| dt.with_timezone(&chrono::Utc))?,
    })
}

// ---------------------------------------------------------------------------
// Model
// ---------------------------------------------------------------------------

pub struct SqliteModelRepository {
    pool: SqlitePool,
}

impl SqliteModelRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ModelRepository for SqliteModelRepository {
    async fn upsert(&self, provider_id: Uuid, models: &[Model]) -> anyhow::Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        for model in models {
            sqlx::query(
                "INSERT INTO models (provider_id, model_id, display_name, context_length, metadata_json, created_at, updated_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?)
                 ON CONFLICT(provider_id, model_id) DO UPDATE SET
                   display_name = excluded.display_name,
                   context_length = excluded.context_length,
                   metadata_json = excluded.metadata_json,
                   updated_at = excluded.updated_at",
            )
            .bind(provider_id.to_string())
            .bind(&model.id)
            .bind(&model.display_name)
            .bind(model.context_length.map(|v| v as i64))
            .bind(model.metadata.to_string())
            .bind(&now)
            .bind(&now)
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }

    async fn list_for_provider(&self, provider_id: Uuid) -> anyhow::Result<Vec<Model>> {
        let rows = sqlx::query("SELECT * FROM models WHERE provider_id = ? ORDER BY model_id")
            .bind(provider_id.to_string())
            .fetch_all(&self.pool)
            .await?;

        rows.iter().map(row_to_model).collect()
    }
}

fn row_to_model(row: &sqlx::sqlite::SqliteRow) -> anyhow::Result<Model> {
    let metadata_str: String = row.try_get("metadata_json")?;
    let metadata: serde_json::Value =
        serde_json::from_str(&metadata_str).unwrap_or(serde_json::json!({}));
    let context_length: Option<i64> = row.try_get("context_length")?;
    Ok(Model {
        id: row.try_get("model_id")?,
        display_name: row.try_get("display_name")?,
        context_length: context_length.map(|v| v as u64),
        metadata,
    })
}

// ---------------------------------------------------------------------------
// Prompt
// ---------------------------------------------------------------------------

pub struct SqlitePromptRepository {
    pool: SqlitePool,
}

impl SqlitePromptRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PromptRepository for SqlitePromptRepository {
    async fn create(&self, prompt: &Prompt) -> anyhow::Result<()> {
        let tags_json = serde_json::to_string(&prompt.tags)?;
        let variables_json = serde_json::to_string(&prompt.variables)?;
        sqlx::query(
            "INSERT INTO prompts (id, name, description, content, system_prompt, tags_json, variables_json, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(prompt.id.to_string())
        .bind(&prompt.name)
        .bind(&prompt.description)
        .bind(&prompt.content)
        .bind(&prompt.system_prompt)
        .bind(&tags_json)
        .bind(&variables_json)
        .bind(prompt.created_at.to_rfc3339())
        .bind(prompt.updated_at.to_rfc3339())
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "INSERT INTO prompts_fts (prompt_id, name, description, content) VALUES (?, ?, ?, ?)",
        )
        .bind(prompt.id.to_string())
        .bind(&prompt.name)
        .bind(&prompt.description)
        .bind(&prompt.content)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get(&self, id: Uuid) -> anyhow::Result<Option<Prompt>> {
        let row = sqlx::query("SELECT * FROM prompts WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await?;
        row.as_ref().map(row_to_prompt).transpose()
    }

    async fn list(&self) -> anyhow::Result<Vec<Prompt>> {
        let rows = sqlx::query("SELECT * FROM prompts ORDER BY updated_at DESC")
            .fetch_all(&self.pool)
            .await?;
        rows.iter().map(row_to_prompt).collect()
    }

    async fn update(&self, prompt: &Prompt) -> anyhow::Result<()> {
        let tags_json = serde_json::to_string(&prompt.tags)?;
        let variables_json = serde_json::to_string(&prompt.variables)?;
        sqlx::query(
            "UPDATE prompts SET name = ?, description = ?, content = ?, system_prompt = ?, \
             tags_json = ?, variables_json = ?, updated_at = ? WHERE id = ?",
        )
        .bind(&prompt.name)
        .bind(&prompt.description)
        .bind(&prompt.content)
        .bind(&prompt.system_prompt)
        .bind(&tags_json)
        .bind(&variables_json)
        .bind(prompt.updated_at.to_rfc3339())
        .bind(prompt.id.to_string())
        .execute(&self.pool)
        .await?;

        sqlx::query("DELETE FROM prompts_fts WHERE prompt_id = ?")
            .bind(prompt.id.to_string())
            .execute(&self.pool)
            .await?;
        sqlx::query(
            "INSERT INTO prompts_fts (prompt_id, name, description, content) VALUES (?, ?, ?, ?)",
        )
        .bind(prompt.id.to_string())
        .bind(&prompt.name)
        .bind(&prompt.description)
        .bind(&prompt.content)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn delete(&self, id: Uuid) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM prompts_fts WHERE prompt_id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;
        sqlx::query("DELETE FROM prompts WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn search(&self, query: &str, limit: u32) -> anyhow::Result<Vec<PromptSearchResult>> {
        if query.trim().is_empty() {
            return Ok(Vec::new());
        }
        let fts_query = query
            .split_whitespace()
            .map(|w| format!("\"{}\"", w.replace('"', "")))
            .collect::<Vec<_>>()
            .join(" ");
        let rows = sqlx::query(
            "SELECT p.* \
             FROM prompts_fts f \
             JOIN prompts p ON p.id = f.prompt_id \
             WHERE prompts_fts MATCH ? \
             ORDER BY rank \
             LIMIT ?",
        )
        .bind(&fts_query)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await?;

        rows.iter()
            .map(|row| {
                let prompt = row_to_prompt(row)?;
                Ok(PromptSearchResult { prompt })
            })
            .collect()
    }
}

fn row_to_prompt(row: &sqlx::sqlite::SqliteRow) -> anyhow::Result<Prompt> {
    let tags_json: String = row.try_get("tags_json")?;
    let variables_json: String = row.try_get("variables_json")?;
    let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();
    let variables: Vec<String> = serde_json::from_str(&variables_json).unwrap_or_default();
    Ok(Prompt {
        id: Uuid::parse_str(&row.try_get::<String, _>("id")?)?,
        name: row.try_get("name")?,
        description: row.try_get("description")?,
        content: row.try_get("content")?,
        system_prompt: row.try_get("system_prompt")?,
        tags,
        variables,
        created_at: chrono::DateTime::parse_from_rfc3339(&row.try_get::<String, _>("created_at")?)
            .map(|dt| dt.with_timezone(&chrono::Utc))?,
        updated_at: chrono::DateTime::parse_from_rfc3339(&row.try_get::<String, _>("updated_at")?)
            .map(|dt| dt.with_timezone(&chrono::Utc))?,
    })
}
