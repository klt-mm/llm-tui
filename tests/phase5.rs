use std::sync::Arc;

use chrono::Utc;
use tokio::sync::mpsc;
use uuid::Uuid;

use llm_tui::app::{App, Modal};
use llm_tui::config::{ContextConfig, GenerationConfig};
use llm_tui::domain::*;
use llm_tui::events::{AppEvent, UserEvent};
use llm_tui::llm::FakeProvider;
use llm_tui::persistence::repositories::*;
use llm_tui::persistence::{
    Database, SqliteConversationRepository, SqliteGenerationRunRepository, SqliteMessageRepository,
    SqliteModelRepository, SqlitePromptRepository, SqliteProviderRepository,
};

async fn test_db() -> (Database, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.db");
    let url = format!("sqlite://{}?mode=rwc", path.display());
    let db = Database::connect(&url).await.unwrap();
    (db, dir)
}

fn make_app(db: &Database, event_tx: mpsc::Sender<AppEvent>) -> App {
    let provider = Arc::new(FakeProvider::new());
    let conv_repo = Arc::new(SqliteConversationRepository::new(db.pool.clone()));
    let msg_repo = Arc::new(SqliteMessageRepository::new(db.pool.clone()));
    let model_repo = Arc::new(SqliteModelRepository::new(db.pool.clone()));
    let provider_repo = Arc::new(SqliteProviderRepository::new(db.pool.clone()));
    let prompt_repo = Arc::new(SqlitePromptRepository::new(db.pool.clone()));
    let generation_run_repo = Arc::new(SqliteGenerationRunRepository::new(db.pool.clone()));
    App::new(
        provider,
        "fake".into(),
        conv_repo,
        msg_repo,
        model_repo,
        provider_repo,
        prompt_repo,
        generation_run_repo,
        GenerationConfig::default(),
        ContextConfig::default(),
        event_tx,
    )
}

// -----------------------------------------------------------------------
// Generation Run Repository
// -----------------------------------------------------------------------

#[tokio::test]
async fn generation_run_create_and_list() {
    let (db, _dir) = test_db().await;
    let repo = SqliteGenerationRunRepository::new(db.pool.clone());

    // Create a provider and message first (foreign key constraints)
    let provider_repo = SqliteProviderRepository::new(db.pool.clone());
    let provider = Provider {
        id: Uuid::new_v4(),
        name: "test".into(),
        base_url: "http://localhost".into(),
        protocol: ProviderProtocol::OpenAiCompatible,
        api_key_ref: None,
        default_model: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    provider_repo.create(&provider).await.unwrap();

    let conv_repo = SqliteConversationRepository::new(db.pool.clone());
    let conv = Conversation {
        id: Uuid::new_v4(),
        title: "Test".into(),
        provider_id: provider.id,
        model_id: "test-model".into(),
        system_prompt: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        archived_at: None,
    };
    conv_repo.create(&conv).await.unwrap();

    let msg_repo = SqliteMessageRepository::new(db.pool.clone());
    let msg = Message {
        id: Uuid::new_v4(),
        conversation_id: conv.id,
        parent_id: None,
        role: Role::Assistant,
        content: "test response".into(),
        reasoning_content: None,
        tool_calls: None,
        tool_call_id: None,
        images: None,
        metadata: serde_json::json!({}),
        created_at: Utc::now(),
    };
    msg_repo.create(&msg).await.unwrap();

    // Now create a generation run
    let run = GenerationRun {
        id: Uuid::new_v4(),
        message_id: msg.id,
        provider_id: provider.id,
        model_id: "test-model".into(),
        started_at: Utc::now(),
        completed_at: Some(Utc::now()),
        prompt_tokens: Some(10),
        completion_tokens: Some(20),
        total_tokens: Some(30),
        prompt_ms: Some(100.0),
        generation_ms: Some(500.0),
        metadata: serde_json::json!({}),
    };
    repo.create(&run).await.unwrap();

    // List runs for the message
    let runs = repo.list_for_message(msg.id).await.unwrap();
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].id, run.id);
    assert_eq!(runs[0].prompt_tokens, Some(10));
    assert_eq!(runs[0].completion_tokens, Some(20));
    assert_eq!(runs[0].total_tokens, Some(30));
}

// -----------------------------------------------------------------------
// Branch History Modal
// -----------------------------------------------------------------------

#[tokio::test]
async fn branch_history_opens_with_messages() {
    let (db, _dir) = test_db().await;
    let (event_tx, _event_rx) = mpsc::channel::<AppEvent>(256);
    let mut app = make_app(&db, event_tx);
    app.init().await;

    // Create a conversation and add some messages
    app.handle_event(AppEvent::User(UserEvent::NewConversation))
        .await;

    // Add messages manually for testing
    app.messages.push(Message {
        id: Uuid::new_v4(),
        conversation_id: app.active_conversation.unwrap(),
        parent_id: None,
        role: Role::User,
        content: "Hello".into(),
        reasoning_content: None,
        tool_calls: None,
        tool_call_id: None,
        images: None,
        metadata: serde_json::json!({}),
        created_at: Utc::now(),
    });
    app.messages.push(Message {
        id: Uuid::new_v4(),
        conversation_id: app.active_conversation.unwrap(),
        parent_id: None,
        role: Role::Assistant,
        content: "Hi there!".into(),
        reasoning_content: None,
        tool_calls: None,
        tool_call_id: None,
        images: None,
        metadata: serde_json::json!({}),
        created_at: Utc::now(),
    });

    // Open branch history
    app.handle_event(AppEvent::User(UserEvent::OpenBranchHistory))
        .await;

    assert!(matches!(app.modal, Modal::BranchHistory { selected: 0 }));
}

#[tokio::test]
async fn branch_history_navigation() {
    let (db, _dir) = test_db().await;
    let (event_tx, _event_rx) = mpsc::channel::<AppEvent>(256);
    let mut app = make_app(&db, event_tx);
    app.init().await;

    app.handle_event(AppEvent::User(UserEvent::NewConversation))
        .await;

    // Add messages
    for i in 0..3 {
        app.messages.push(Message {
            id: Uuid::new_v4(),
            conversation_id: app.active_conversation.unwrap(),
            parent_id: None,
            role: if i % 2 == 0 {
                Role::User
            } else {
                Role::Assistant
            },
            content: format!("Message {}", i),
            reasoning_content: None,
            tool_calls: None,
            tool_call_id: None,
            images: None,
            metadata: serde_json::json!({}),
            created_at: Utc::now(),
        });
    }

    // Open branch history
    app.handle_event(AppEvent::User(UserEvent::OpenBranchHistory))
        .await;

    // Navigate down
    app.handle_event(AppEvent::User(UserEvent::NavigateDown))
        .await;
    if let Modal::BranchHistory { selected } = app.modal {
        assert_eq!(selected, 1);
    } else {
        panic!("Expected BranchHistory modal");
    }

    // Navigate down again
    app.handle_event(AppEvent::User(UserEvent::NavigateDown))
        .await;
    if let Modal::BranchHistory { selected } = app.modal {
        assert_eq!(selected, 2);
    } else {
        panic!("Expected BranchHistory modal");
    }

    // Navigate up
    app.handle_event(AppEvent::User(UserEvent::NavigateUp))
        .await;
    if let Modal::BranchHistory { selected } = app.modal {
        assert_eq!(selected, 1);
    } else {
        panic!("Expected BranchHistory modal");
    }
}

#[tokio::test]
async fn edit_as_branch_copies_to_input() {
    let (db, _dir) = test_db().await;
    let (event_tx, _event_rx) = mpsc::channel::<AppEvent>(256);
    let mut app = make_app(&db, event_tx);
    app.init().await;

    app.handle_event(AppEvent::User(UserEvent::NewConversation))
        .await;

    // Add a message
    let msg_content = "This is the original message";
    app.messages.push(Message {
        id: Uuid::new_v4(),
        conversation_id: app.active_conversation.unwrap(),
        parent_id: None,
        role: Role::User,
        content: msg_content.into(),
        reasoning_content: None,
        tool_calls: None,
        tool_call_id: None,
        images: None,
        metadata: serde_json::json!({}),
        created_at: Utc::now(),
    });

    // Open branch history
    app.handle_event(AppEvent::User(UserEvent::OpenBranchHistory))
        .await;

    // Confirm (Enter) to copy message to input
    app.handle_event(AppEvent::User(UserEvent::SendMessage))
        .await;

    // Modal should be closed
    assert!(matches!(app.modal, Modal::None));

    // Input should contain the message content
    assert_eq!(app.input, msg_content);
}

// -----------------------------------------------------------------------
// Generation Metrics
// -----------------------------------------------------------------------

#[tokio::test]
async fn last_generation_metrics_stored_after_completion() {
    let (db, _dir) = test_db().await;
    let (event_tx, mut event_rx) = mpsc::channel::<AppEvent>(256);
    let mut app = make_app(&db, event_tx);
    app.init().await;

    // Initially no metrics
    assert!(app.last_generation_metrics.is_none());

    // Create conversation and send message
    app.handle_event(AppEvent::User(UserEvent::NewConversation))
        .await;
    app.handle_event(AppEvent::User(UserEvent::InputChar('h')))
        .await;
    app.handle_event(AppEvent::User(UserEvent::SendMessage))
        .await;

    // Wait for generation to complete
    let mut completed = false;
    for _ in 0..200 {
        tokio::select! {
            Some(event) = event_rx.recv() => {
                app.handle_event(event).await;
                if app.streaming.is_none() && app.messages.len() >= 2 {
                    completed = true;
                    break;
                }
            }
            _ = tokio::time::sleep(std::time::Duration::from_millis(50)) => {}
        }
        if completed {
            break;
        }
    }

    assert!(completed, "generation should complete");

    // Metrics should now be stored
    assert!(
        app.last_generation_metrics.is_some(),
        "generation metrics should be stored after completion"
    );

    let metrics = app.last_generation_metrics.as_ref().unwrap();
    assert!(metrics.total_tokens > 0, "should have token count");
    assert!(metrics.duration_ms > 0.0, "should have duration");
}
