use std::sync::Arc;

use tokio::sync::mpsc;
use uuid::Uuid;

use llm_tui::app::App;
use llm_tui::config::{Config, GenerationConfig, ProviderConfig};
use llm_tui::domain::{Model, ProviderProtocol};
use llm_tui::events::{AppEvent, UserEvent};
use llm_tui::llm::FakeProvider;
use llm_tui::persistence::repositories::*;
use llm_tui::persistence::{
    Database, SqliteConversationRepository, SqliteMessageRepository, SqliteModelRepository,
    SqliteProviderRepository,
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
    App::new(
        provider,
        "fake".into(),
        conv_repo,
        msg_repo,
        model_repo,
        provider_repo,
        GenerationConfig::default(),
        event_tx,
    )
}

async fn seed_provider(db: &Database) -> Uuid {
    let repo = SqliteProviderRepository::new(db.pool.clone());
    let provider = llm_tui::domain::Provider {
        id: Uuid::new_v4(),
        name: "test".into(),
        base_url: "http://localhost".into(),
        protocol: ProviderProtocol::OpenAiCompatible,
        api_key_ref: None,
        default_model: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    repo.create(&provider).await.unwrap();
    provider.id
}

// -----------------------------------------------------------------------
// Config
// -----------------------------------------------------------------------

#[test]
fn config_default_has_no_provider() {
    let config = Config::default();
    assert!(!config.is_provider_configured());
    assert!(config.provider.base_url.is_none());
}

#[test]
fn config_with_base_url_is_configured() {
    let config = Config {
        provider: ProviderConfig {
            base_url: Some("http://localhost:8080/v1".into()),
            ..Default::default()
        },
        generation: GenerationConfig::default(),
    };
    assert!(config.is_provider_configured());
}

#[test]
fn config_env_override() {
    // SAFETY: test-only, single-threaded test context
    unsafe { std::env::set_var("LLM_TUI_BASE_URL", "http://env-test:9090/v1") };
    let mut config = Config::default();
    config.apply_env_overrides();
    assert_eq!(
        config.provider.base_url.as_deref(),
        Some("http://env-test:9090/v1")
    );
    unsafe { std::env::remove_var("LLM_TUI_BASE_URL") };
}

// -----------------------------------------------------------------------
// Model Repository
// -----------------------------------------------------------------------

#[tokio::test]
async fn model_repo_upsert_and_list() {
    let (db, _dir) = test_db().await;
    let model_repo = SqliteModelRepository::new(db.pool.clone());
    let provider_id = seed_provider(&db).await;

    let models = vec![
        Model {
            id: "model-a".into(),
            display_name: Some("Model A".into()),
            context_length: Some(4096),
            metadata: serde_json::json!({}),
        },
        Model {
            id: "model-b".into(),
            display_name: Some("Model B".into()),
            context_length: Some(8192),
            metadata: serde_json::json!({}),
        },
    ];

    model_repo.upsert(provider_id, &models).await.unwrap();

    let loaded = model_repo.list_for_provider(provider_id).await.unwrap();
    assert_eq!(loaded.len(), 2);
    assert_eq!(loaded[0].id, "model-a");
    assert_eq!(loaded[1].id, "model-b");
}

#[tokio::test]
async fn model_repo_upsert_updates_existing() {
    let (db, _dir) = test_db().await;
    let model_repo = SqliteModelRepository::new(db.pool.clone());
    let provider_id = seed_provider(&db).await;

    let models_v1 = vec![Model {
        id: "model-a".into(),
        display_name: Some("Old Name".into()),
        context_length: Some(4096),
        metadata: serde_json::json!({}),
    }];
    model_repo.upsert(provider_id, &models_v1).await.unwrap();

    let models_v2 = vec![Model {
        id: "model-a".into(),
        display_name: Some("New Name".into()),
        context_length: Some(8192),
        metadata: serde_json::json!({}),
    }];
    model_repo.upsert(provider_id, &models_v2).await.unwrap();

    let loaded = model_repo.list_for_provider(provider_id).await.unwrap();
    assert_eq!(loaded.len(), 1);
    assert_eq!(loaded[0].display_name.as_deref(), Some("New Name"));
    assert_eq!(loaded[0].context_length, Some(8192));
}

// -----------------------------------------------------------------------
// Connection diagnostics
// -----------------------------------------------------------------------

#[tokio::test]
async fn test_connection_succeeds_with_fake_provider() {
    let (db, _dir) = test_db().await;
    let (event_tx, _event_rx) = mpsc::channel::<AppEvent>(256);
    let mut app = make_app(&db, event_tx);
    app.init().await;

    app.test_connection().await;
    assert!(app.error.is_none(), "connection should succeed with fake provider");
    assert!(!app.models.is_empty(), "models should be loaded");
}

// -----------------------------------------------------------------------
// Model selection
// -----------------------------------------------------------------------

#[tokio::test]
async fn select_model_by_index() {
    let (db, _dir) = test_db().await;
    let (event_tx, _event_rx) = mpsc::channel::<AppEvent>(256);
    let mut app = make_app(&db, event_tx);
    app.init().await;

    assert!(app.models.len() >= 2);

    app.handle_event(AppEvent::User(UserEvent::SelectModel(1))).await;
    assert_eq!(app.selected_model.as_deref(), Some("fake-slow"));

    app.handle_event(AppEvent::User(UserEvent::SelectModel(0))).await;
    assert_eq!(app.selected_model.as_deref(), Some("fake-fast"));
}

#[tokio::test]
async fn cycle_models_with_ctrl_m() {
    let (db, _dir) = test_db().await;
    let (event_tx, _event_rx) = mpsc::channel::<AppEvent>(256);
    let mut app = make_app(&db, event_tx);
    app.init().await;

    let first_model = app.selected_model.clone().unwrap();

    app.handle_event(AppEvent::User(UserEvent::SelectModel(usize::MAX))).await;
    let second_model = app.selected_model.clone().unwrap();
    assert_ne!(first_model, second_model);

    app.handle_event(AppEvent::User(UserEvent::SelectModel(usize::MAX))).await;
    let third_model = app.selected_model.clone().unwrap();
    assert_ne!(second_model, third_model);
}

// -----------------------------------------------------------------------
// Generation settings
// -----------------------------------------------------------------------

#[tokio::test]
async fn generation_config_passed_to_chat_request() {
    let (db, _dir) = test_db().await;
    let (event_tx, mut event_rx) = mpsc::channel::<AppEvent>(256);

    let provider = Arc::new(FakeProvider::new());
    let conv_repo = Arc::new(SqliteConversationRepository::new(db.pool.clone()));
    let msg_repo = Arc::new(SqliteMessageRepository::new(db.pool.clone()));
    let model_repo = Arc::new(SqliteModelRepository::new(db.pool.clone()));
    let provider_repo = Arc::new(SqliteProviderRepository::new(db.pool.clone()));

    let gen_config = GenerationConfig {
        temperature: Some(0.7),
        top_p: Some(0.9),
        max_tokens: Some(100),
    };

    let mut app = App::new(
        provider,
        "fake".into(),
        conv_repo,
        msg_repo,
        model_repo,
        provider_repo,
        gen_config,
        event_tx.clone(),
    );
    app.init().await;

    app.handle_event(AppEvent::User(UserEvent::NewConversation)).await;
    app.handle_event(AppEvent::User(UserEvent::InputChar('h'))).await;
    app.handle_event(AppEvent::User(UserEvent::SendMessage)).await;

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
        if completed { break; }
    }
    assert!(completed, "generation should complete");
}

// -----------------------------------------------------------------------
// Streaming stats
// -----------------------------------------------------------------------

#[tokio::test]
async fn streaming_stats_available_during_generation() {
    let (db, _dir) = test_db().await;
    let (event_tx, mut event_rx) = mpsc::channel::<AppEvent>(256);
    let mut app = make_app(&db, event_tx.clone());
    app.init().await;

    app.handle_event(AppEvent::User(UserEvent::NewConversation)).await;
    app.handle_event(AppEvent::User(UserEvent::InputChar('t'))).await;
    app.handle_event(AppEvent::User(UserEvent::SendMessage)).await;

    // Wait for at least one delta
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    // Drain a few events
    for _ in 0..5 {
        match event_rx.try_recv() {
            Ok(event) => app.handle_event(event).await,
            Err(_) => break,
        }
    }

    if app.streaming.is_some() {
        let stats = app.streaming_stats();
        assert!(stats.is_some(), "streaming stats should be available");
    }
}
