use chrono::Utc;
use uuid::Uuid;

use llm_tui::domain::*;
use llm_tui::persistence::repositories::*;
use llm_tui::persistence::{
    Database, SqliteConversationRepository, SqliteMessageRepository, SqlitePromptRepository,
    SqliteProviderRepository,
};

async fn test_db() -> (Database, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.db");
    let url = format!("sqlite://{}?mode=rwc", path.display());
    let db = Database::connect(&url).await.unwrap();
    (db, dir)
}

fn make_provider() -> Provider {
    Provider {
        id: Uuid::new_v4(),
        name: "test".into(),
        base_url: "http://localhost:8080/v1".into(),
        protocol: ProviderProtocol::OpenAiCompatible,
        api_key_ref: None,
        default_model: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

async fn seed_provider(db: &Database) -> Uuid {
    let repo = SqliteProviderRepository::new(db.pool.clone());
    let provider = make_provider();
    repo.create(&provider).await.unwrap();
    provider.id
}

fn make_conversation(provider_id: Uuid) -> Conversation {
    Conversation {
        id: Uuid::new_v4(),
        title: "Test".into(),
        provider_id,
        model_id: "test-model".into(),
        system_prompt: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        archived_at: None,
    }
}

fn make_message(conversation_id: Uuid, role: Role, content: &str) -> Message {
    Message {
        id: Uuid::new_v4(),
        conversation_id,
        parent_id: None,
        role,
        content: content.into(),
        reasoning_content: None,
        metadata: serde_json::json!({}),
        created_at: Utc::now(),
    }
}

// -----------------------------------------------------------------------
// Conversation
// -----------------------------------------------------------------------

#[tokio::test]
async fn conversation_create_and_get() {
    let (db, _dir) = test_db().await;
    let repo = SqliteConversationRepository::new(db.pool.clone());
    let provider_id = seed_provider(&db).await;

    let conv = make_conversation(provider_id);
    repo.create(&conv).await.unwrap();

    let loaded = repo.get(conv.id).await.unwrap().unwrap();
    assert_eq!(loaded.id, conv.id);
    assert_eq!(loaded.title, "Test");
    assert_eq!(loaded.provider_id, provider_id);
}

#[tokio::test]
async fn conversation_list_ordered_by_updated_at() {
    let (db, _dir) = test_db().await;
    let repo = SqliteConversationRepository::new(db.pool.clone());
    let pid = seed_provider(&db).await;

    let c1 = make_conversation(pid);
    let mut c2 = make_conversation(pid);
    c2.updated_at = c1.updated_at + chrono::Duration::seconds(10);

    repo.create(&c1).await.unwrap();
    repo.create(&c2).await.unwrap();

    let list = repo.list().await.unwrap();
    assert_eq!(list.len(), 2);
    assert_eq!(list[0].id, c2.id);
    assert_eq!(list[1].id, c1.id);
}

// -----------------------------------------------------------------------
// Message
// -----------------------------------------------------------------------

#[tokio::test]
async fn message_create_and_list() {
    let (db, _dir) = test_db().await;
    let conv_repo = SqliteConversationRepository::new(db.pool.clone());
    let msg_repo = SqliteMessageRepository::new(db.pool.clone());
    let pid = seed_provider(&db).await;

    let conv = make_conversation(pid);
    conv_repo.create(&conv).await.unwrap();

    let m1 = make_message(conv.id, Role::User, "hello");
    let m2 = make_message(conv.id, Role::Assistant, "hi there");
    msg_repo.create(&m1).await.unwrap();
    msg_repo.create(&m2).await.unwrap();

    let messages = msg_repo.list_for_conversation(conv.id).await.unwrap();
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0].role, Role::User);
    assert_eq!(messages[0].content, "hello");
    assert_eq!(messages[1].role, Role::Assistant);
    assert_eq!(messages[1].content, "hi there");
}

#[tokio::test]
async fn message_parent_branch_relationship() {
    let (db, _dir) = test_db().await;
    let conv_repo = SqliteConversationRepository::new(db.pool.clone());
    let msg_repo = SqliteMessageRepository::new(db.pool.clone());
    let pid = seed_provider(&db).await;

    let conv = make_conversation(pid);
    conv_repo.create(&conv).await.unwrap();

    let user_msg = make_message(conv.id, Role::User, "question");
    msg_repo.create(&user_msg).await.unwrap();

    let mut assistant_msg = make_message(conv.id, Role::Assistant, "answer");
    assistant_msg.parent_id = Some(user_msg.id);
    msg_repo.create(&assistant_msg).await.unwrap();

    let messages = msg_repo.list_for_conversation(conv.id).await.unwrap();
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[1].parent_id, Some(user_msg.id));
}

#[tokio::test]
async fn cascade_delete_conversation_removes_messages() {
    let (db, _dir) = test_db().await;
    let conv_repo = SqliteConversationRepository::new(db.pool.clone());
    let msg_repo = SqliteMessageRepository::new(db.pool.clone());
    let pid = seed_provider(&db).await;

    let conv = make_conversation(pid);
    conv_repo.create(&conv).await.unwrap();

    let msg = make_message(conv.id, Role::User, "will be deleted");
    msg_repo.create(&msg).await.unwrap();

    sqlx::query("DELETE FROM conversations WHERE id = ?")
        .bind(conv.id.to_string())
        .execute(&db.pool)
        .await
        .unwrap();

    let messages = msg_repo.list_for_conversation(conv.id).await.unwrap();
    assert!(messages.is_empty());
}

// -----------------------------------------------------------------------
// Prompt
// -----------------------------------------------------------------------

#[tokio::test]
async fn prompt_create_and_get() {
    let (db, _dir) = test_db().await;
    let repo = SqlitePromptRepository::new(db.pool.clone());

    let prompt = Prompt {
        id: Uuid::new_v4(),
        name: "test-prompt".into(),
        description: Some("A test prompt".into()),
        content: "Hello {{name}}".into(),
        system_prompt: Some("You are helpful".into()),
        tags: vec!["test".into()],
        variables: vec!["name".into()],
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    repo.create(&prompt).await.unwrap();

    let loaded = repo.get(prompt.id).await.unwrap().unwrap();
    assert_eq!(loaded.name, "test-prompt");
    assert_eq!(loaded.content, "Hello {{name}}");
    assert_eq!(loaded.tags, vec!["test"]);
    assert_eq!(loaded.variables, vec!["name"]);
}
