use std::sync::Arc;

use tokio::sync::mpsc;

use llm_tui::app::{App, Command, Modal};
use llm_tui::config::{ContextConfig, GenerationConfig};
use llm_tui::events::{AppEvent, UserEvent};
use llm_tui::llm::FakeProvider;
use llm_tui::persistence::{
    Database, SqliteConversationRepository, SqliteMessageRepository, SqliteModelRepository,
    SqlitePromptRepository, SqliteProviderRepository,
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
    App::new(
        provider,
        "fake".into(),
        conv_repo,
        msg_repo,
        model_repo,
        provider_repo,
        prompt_repo,
        GenerationConfig::default(),
        ContextConfig::default(),
        event_tx,
    )
}

// -----------------------------------------------------------------------
// Command palette
// -----------------------------------------------------------------------

#[tokio::test]
async fn command_palette_opens_with_all_commands() {
    let (db, _dir) = test_db().await;
    let (event_tx, _event_rx) = mpsc::channel::<AppEvent>(256);
    let mut app = make_app(&db, event_tx);
    app.init().await;

    app.handle_event(AppEvent::User(UserEvent::OpenCommandPalette))
        .await;

    match &app.modal {
        Modal::CommandPalette { filtered, .. } => {
            assert_eq!(filtered.len(), Command::ALL.len());
        }
        _ => panic!("expected CommandPalette modal"),
    }
}

#[tokio::test]
async fn command_palette_filters_by_query() {
    let (db, _dir) = test_db().await;
    let (event_tx, _event_rx) = mpsc::channel::<AppEvent>(256);
    let mut app = make_app(&db, event_tx);
    app.init().await;

    app.handle_event(AppEvent::User(UserEvent::OpenCommandPalette))
        .await;

    // Type "search"
    for c in "search".chars() {
        app.handle_event(AppEvent::User(UserEvent::InputChar(c)))
            .await;
    }

    match &app.modal {
        Modal::CommandPalette { filtered, .. } => {
            assert!(
                filtered.contains(&Command::Search),
                "filtered should contain Search"
            );
            assert!(
                !filtered.contains(&Command::Quit),
                "filtered should not contain Quit"
            );
        }
        _ => panic!("expected CommandPalette modal"),
    }
}

#[tokio::test]
async fn command_palette_executes_new_conversation() {
    let (db, _dir) = test_db().await;
    let (event_tx, _event_rx) = mpsc::channel::<AppEvent>(256);
    let mut app = make_app(&db, event_tx);
    app.init().await;

    let initial_count = app.conversations.len();

    app.handle_event(AppEvent::User(UserEvent::OpenCommandPalette))
        .await;

    // Type "new" to filter
    for c in "new".chars() {
        app.handle_event(AppEvent::User(UserEvent::InputChar(c)))
            .await;
    }

    // Select first match (should be New Conversation) and confirm
    app.handle_event(AppEvent::User(UserEvent::SendMessage))
        .await;

    assert_eq!(
        app.conversations.len(),
        initial_count + 1,
        "new conversation should be created"
    );
}

#[tokio::test]
async fn command_palette_executes_quit() {
    let (db, _dir) = test_db().await;
    let (event_tx, _event_rx) = mpsc::channel::<AppEvent>(256);
    let mut app = make_app(&db, event_tx);
    app.init().await;

    app.handle_event(AppEvent::User(UserEvent::OpenCommandPalette))
        .await;

    // Type "quit"
    for c in "quit".chars() {
        app.handle_event(AppEvent::User(UserEvent::InputChar(c)))
            .await;
    }

    // Confirm
    app.handle_event(AppEvent::User(UserEvent::SendMessage))
        .await;

    assert!(app.should_quit, "quit command should set should_quit");
}

#[tokio::test]
async fn command_palette_navigate_and_select() {
    let (db, _dir) = test_db().await;
    let (event_tx, _event_rx) = mpsc::channel::<AppEvent>(256);
    let mut app = make_app(&db, event_tx);
    app.init().await;

    app.handle_event(AppEvent::User(UserEvent::OpenCommandPalette))
        .await;

    // Navigate down
    app.handle_event(AppEvent::User(UserEvent::NavigateDown))
        .await;

    match &app.modal {
        Modal::CommandPalette { selected, .. } => {
            assert_eq!(*selected, 1, "selection should move to index 1");
        }
        _ => panic!("expected CommandPalette modal"),
    }

    // Navigate up
    app.handle_event(AppEvent::User(UserEvent::NavigateUp))
        .await;

    match &app.modal {
        Modal::CommandPalette { selected, .. } => {
            assert_eq!(*selected, 0, "selection should move back to 0");
        }
        _ => panic!("expected CommandPalette modal"),
    }
}

// -----------------------------------------------------------------------
// Model selector
// -----------------------------------------------------------------------

#[tokio::test]
async fn model_selector_opens_and_selects() {
    let (db, _dir) = test_db().await;
    let (event_tx, _event_rx) = mpsc::channel::<AppEvent>(256);
    let mut app = make_app(&db, event_tx);
    app.init().await;

    assert!(app.models.len() >= 2, "FakeProvider should provide models");

    app.handle_event(AppEvent::User(UserEvent::OpenModelSelector))
        .await;

    assert!(matches!(app.modal, Modal::ModelSelector { selected: 0 }));

    // Navigate to second model
    app.handle_event(AppEvent::User(UserEvent::NavigateDown))
        .await;

    // Select it
    app.handle_event(AppEvent::User(UserEvent::SendMessage))
        .await;

    assert!(matches!(app.modal, Modal::None));
    assert_eq!(
        app.selected_model.as_deref(),
        Some("fake-slow"),
        "should select second model"
    );
}

// -----------------------------------------------------------------------
// Generation settings
// -----------------------------------------------------------------------

#[tokio::test]
async fn generation_settings_opens_with_current_values() {
    let (db, _dir) = test_db().await;
    let (event_tx, _event_rx) = mpsc::channel::<AppEvent>(256);

    let gen_config = GenerationConfig {
        temperature: Some(0.7),
        top_p: Some(0.9),
        max_tokens: Some(100),
    };

    let provider = Arc::new(FakeProvider::new());
    let conv_repo = Arc::new(SqliteConversationRepository::new(db.pool.clone()));
    let msg_repo = Arc::new(SqliteMessageRepository::new(db.pool.clone()));
    let model_repo = Arc::new(SqliteModelRepository::new(db.pool.clone()));
    let provider_repo = Arc::new(SqliteProviderRepository::new(db.pool.clone()));
    let prompt_repo = Arc::new(SqlitePromptRepository::new(db.pool.clone()));

    let mut app = App::new(
        provider,
        "fake".into(),
        conv_repo,
        msg_repo,
        model_repo,
        provider_repo,
        prompt_repo,
        gen_config,
        ContextConfig::default(),
        event_tx,
    );
    app.init().await;

    app.handle_event(AppEvent::User(UserEvent::OpenGenerationSettings))
        .await;

    match &app.modal {
        Modal::GenerationSettings {
            temperature,
            top_p,
            max_tokens,
            field,
        } => {
            assert_eq!(temperature, "0.7");
            assert_eq!(top_p, "0.9");
            assert_eq!(max_tokens, "100");
            assert_eq!(*field, 0);
        }
        _ => panic!("expected GenerationSettings modal"),
    }
}

#[tokio::test]
async fn generation_settings_saves_on_confirm() {
    let (db, _dir) = test_db().await;
    let (event_tx, _event_rx) = mpsc::channel::<AppEvent>(256);
    let mut app = make_app(&db, event_tx);
    app.init().await;

    app.handle_event(AppEvent::User(UserEvent::OpenGenerationSettings))
        .await;

    // Clear temperature field and type new value
    // First clear existing (it's empty by default)
    for c in "0.5".chars() {
        app.handle_event(AppEvent::User(UserEvent::InputChar(c)))
            .await;
    }

    // Move to top_p field
    app.handle_event(AppEvent::User(UserEvent::NavigateDown))
        .await;
    for c in "0.8".chars() {
        app.handle_event(AppEvent::User(UserEvent::InputChar(c)))
            .await;
    }

    // Move to max_tokens field
    app.handle_event(AppEvent::User(UserEvent::NavigateDown))
        .await;
    for c in "200".chars() {
        app.handle_event(AppEvent::User(UserEvent::InputChar(c)))
            .await;
    }

    // Save
    app.handle_event(AppEvent::User(UserEvent::SendMessage))
        .await;

    assert!(matches!(app.modal, Modal::None));
    assert_eq!(app.generation.temperature, Some(0.5));
    assert_eq!(app.generation.top_p, Some(0.8));
    assert_eq!(app.generation.max_tokens, Some(200));
}

#[tokio::test]
async fn generation_settings_empty_fields_become_none() {
    let (db, _dir) = test_db().await;
    let (event_tx, _event_rx) = mpsc::channel::<AppEvent>(256);

    let gen_config = GenerationConfig {
        temperature: Some(0.7),
        top_p: Some(0.9),
        max_tokens: Some(100),
    };

    let provider = Arc::new(FakeProvider::new());
    let conv_repo = Arc::new(SqliteConversationRepository::new(db.pool.clone()));
    let msg_repo = Arc::new(SqliteMessageRepository::new(db.pool.clone()));
    let model_repo = Arc::new(SqliteModelRepository::new(db.pool.clone()));
    let provider_repo = Arc::new(SqliteProviderRepository::new(db.pool.clone()));
    let prompt_repo = Arc::new(SqlitePromptRepository::new(db.pool.clone()));

    let mut app = App::new(
        provider,
        "fake".into(),
        conv_repo,
        msg_repo,
        model_repo,
        provider_repo,
        prompt_repo,
        gen_config,
        ContextConfig::default(),
        event_tx,
    );
    app.init().await;

    app.handle_event(AppEvent::User(UserEvent::OpenGenerationSettings))
        .await;

    // Clear all fields by backspacing
    for _ in 0..10 {
        app.handle_event(AppEvent::User(UserEvent::Backspace)).await;
    }
    app.handle_event(AppEvent::User(UserEvent::NavigateDown))
        .await;
    for _ in 0..10 {
        app.handle_event(AppEvent::User(UserEvent::Backspace)).await;
    }
    app.handle_event(AppEvent::User(UserEvent::NavigateDown))
        .await;
    for _ in 0..10 {
        app.handle_event(AppEvent::User(UserEvent::Backspace)).await;
    }

    // Save
    app.handle_event(AppEvent::User(UserEvent::SendMessage))
        .await;

    assert_eq!(app.generation.temperature, None);
    assert_eq!(app.generation.top_p, None);
    assert_eq!(app.generation.max_tokens, None);
}

// -----------------------------------------------------------------------
// Command enum
// -----------------------------------------------------------------------

#[test]
fn command_labels_are_unique() {
    let labels: Vec<&str> = Command::ALL.iter().map(|c| c.label()).collect();
    let mut deduped = labels.clone();
    deduped.sort();
    deduped.dedup();
    assert_eq!(
        labels.len(),
        deduped.len(),
        "command labels should be unique"
    );
}

#[test]
fn command_matches_is_case_insensitive() {
    assert!(Command::Search.matches("search"));
    assert!(Command::Search.matches("SEARCH"));
    assert!(Command::Search.matches("Search"));
    assert!(!Command::Search.matches("xyz"));
}
