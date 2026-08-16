use std::sync::Arc;

use tokio::sync::mpsc;

use llm_tui::app::App;
use llm_tui::config::{ContextConfig, GenerationConfig};
use llm_tui::events::{AppEvent, UserEvent};
use llm_tui::llm::FakeProvider;
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
// Focus toggling
// -----------------------------------------------------------------------

#[tokio::test]
async fn toggle_focus_switches_between_sidebar_and_input() {
    let (db, _dir) = test_db().await;
    let (event_tx, _event_rx) = mpsc::channel::<AppEvent>(256);
    let mut app = make_app(&db, event_tx);
    app.init().await;

    assert!(!app.sidebar_focus, "should start with input focused");

    app.handle_event(AppEvent::User(UserEvent::ToggleFocus))
        .await;
    assert!(app.sidebar_focus, "should switch to sidebar focused");

    app.handle_event(AppEvent::User(UserEvent::ToggleFocus))
        .await;
    assert!(!app.sidebar_focus, "should switch back to input focused");
}

#[tokio::test]
async fn toggle_focus_syncs_selection_with_active_conversation() {
    let (db, _dir) = test_db().await;
    let (event_tx, _event_rx) = mpsc::channel::<AppEvent>(256);
    let mut app = make_app(&db, event_tx);
    app.init().await;

    // Create 3 conversations
    app.handle_event(AppEvent::User(UserEvent::NewConversation))
        .await;
    app.handle_event(AppEvent::User(UserEvent::NewConversation))
        .await;
    app.handle_event(AppEvent::User(UserEvent::NewConversation))
        .await;

    let active_id = app.active_conversation.unwrap();

    // Switch to sidebar — selection should sync to active conversation position
    app.handle_event(AppEvent::User(UserEvent::ToggleFocus))
        .await;
    assert!(app.sidebar_focus);

    // Active conversation should be findable in the list
    let expected_pos = app
        .conversations
        .iter()
        .position(|c| c.id == active_id)
        .unwrap();
    assert_eq!(app.sidebar_selection, expected_pos);
}

// -----------------------------------------------------------------------
// Navigation
// -----------------------------------------------------------------------

#[tokio::test]
async fn j_k_navigate_sidebar_selection() {
    let (db, _dir) = test_db().await;
    let (event_tx, _event_rx) = mpsc::channel::<AppEvent>(256);
    let mut app = make_app(&db, event_tx);
    app.init().await;

    // Create 3 conversations
    app.handle_event(AppEvent::User(UserEvent::NewConversation))
        .await;
    app.handle_event(AppEvent::User(UserEvent::NewConversation))
        .await;
    app.handle_event(AppEvent::User(UserEvent::NewConversation))
        .await;

    // Switch to sidebar
    app.handle_event(AppEvent::User(UserEvent::ToggleFocus))
        .await;
    assert_eq!(app.sidebar_selection, 0);

    // j moves down
    app.handle_event(AppEvent::User(UserEvent::InputChar('j')))
        .await;
    assert_eq!(app.sidebar_selection, 1);

    app.handle_event(AppEvent::User(UserEvent::InputChar('j')))
        .await;
    assert_eq!(app.sidebar_selection, 2);

    // j at bottom stays at bottom
    app.handle_event(AppEvent::User(UserEvent::InputChar('j')))
        .await;
    assert_eq!(app.sidebar_selection, 2);

    // k moves up
    app.handle_event(AppEvent::User(UserEvent::InputChar('k')))
        .await;
    assert_eq!(app.sidebar_selection, 1);

    // k at top stays at top
    app.handle_event(AppEvent::User(UserEvent::InputChar('k')))
        .await;
    assert_eq!(app.sidebar_selection, 0);
    app.handle_event(AppEvent::User(UserEvent::InputChar('k')))
        .await;
    assert_eq!(app.sidebar_selection, 0);
}

#[tokio::test]
async fn arrow_keys_navigate_sidebar() {
    let (db, _dir) = test_db().await;
    let (event_tx, _event_rx) = mpsc::channel::<AppEvent>(256);
    let mut app = make_app(&db, event_tx);
    app.init().await;

    app.handle_event(AppEvent::User(UserEvent::NewConversation))
        .await;
    app.handle_event(AppEvent::User(UserEvent::NewConversation))
        .await;
    app.handle_event(AppEvent::User(UserEvent::NewConversation))
        .await;

    app.handle_event(AppEvent::User(UserEvent::ToggleFocus))
        .await;

    app.handle_event(AppEvent::User(UserEvent::NavigateDown))
        .await;
    assert_eq!(app.sidebar_selection, 1);

    app.handle_event(AppEvent::User(UserEvent::NavigateUp))
        .await;
    assert_eq!(app.sidebar_selection, 0);
}

#[tokio::test]
async fn navigation_ignored_when_sidebar_not_focused() {
    let (db, _dir) = test_db().await;
    let (event_tx, _event_rx) = mpsc::channel::<AppEvent>(256);
    let mut app = make_app(&db, event_tx);
    app.init().await;

    app.handle_event(AppEvent::User(UserEvent::NewConversation))
        .await;
    app.handle_event(AppEvent::User(UserEvent::NewConversation))
        .await;

    assert!(!app.sidebar_focus);
    assert_eq!(app.sidebar_selection, 0);

    // NavigateDown should not change selection when sidebar not focused
    app.handle_event(AppEvent::User(UserEvent::NavigateDown))
        .await;
    assert_eq!(app.sidebar_selection, 0);

    // j should type into input, not navigate
    app.handle_event(AppEvent::User(UserEvent::InputChar('j')))
        .await;
    assert_eq!(app.input, "j");
    assert_eq!(app.sidebar_selection, 0);
}

// -----------------------------------------------------------------------
// Opening conversations
// -----------------------------------------------------------------------

#[tokio::test]
async fn enter_opens_selected_conversation() {
    let (db, _dir) = test_db().await;
    let (event_tx, _event_rx) = mpsc::channel::<AppEvent>(256);
    let mut app = make_app(&db, event_tx);
    app.init().await;

    // Create conversations
    app.handle_event(AppEvent::User(UserEvent::NewConversation))
        .await;
    let _first_id = app.active_conversation.unwrap();
    app.handle_event(AppEvent::User(UserEvent::NewConversation))
        .await;
    let _second_id = app.active_conversation.unwrap();

    // Switch to sidebar, navigate to first conversation
    app.handle_event(AppEvent::User(UserEvent::ToggleFocus))
        .await;
    app.handle_event(AppEvent::User(UserEvent::NavigateDown))
        .await;
    assert_eq!(app.sidebar_selection, 1);

    // Open it with Enter (SendMessage in sidebar mode)
    app.handle_event(AppEvent::User(UserEvent::SendMessage))
        .await;

    assert!(!app.sidebar_focus, "focus should return to input");
    assert_eq!(app.active_conversation.unwrap(), app.conversations[1].id);
}

#[tokio::test]
async fn opening_already_active_conversation_returns_to_input() {
    let (db, _dir) = test_db().await;
    let (event_tx, _event_rx) = mpsc::channel::<AppEvent>(256);
    let mut app = make_app(&db, event_tx);
    app.init().await;

    app.handle_event(AppEvent::User(UserEvent::NewConversation))
        .await;
    let conv_id = app.active_conversation.unwrap();

    // Switch to sidebar — selection syncs to active
    app.handle_event(AppEvent::User(UserEvent::ToggleFocus))
        .await;
    assert!(app.sidebar_focus);

    // Open the already-active conversation
    app.handle_event(AppEvent::User(UserEvent::SendMessage))
        .await;
    assert!(!app.sidebar_focus, "should return to input");
    assert_eq!(app.active_conversation, Some(conv_id));
}

// -----------------------------------------------------------------------
// Input guarding
// -----------------------------------------------------------------------

#[tokio::test]
async fn input_ignored_when_sidebar_focused() {
    let (db, _dir) = test_db().await;
    let (event_tx, _event_rx) = mpsc::channel::<AppEvent>(256);
    let mut app = make_app(&db, event_tx);
    app.init().await;

    app.handle_event(AppEvent::User(UserEvent::NewConversation))
        .await;
    app.handle_event(AppEvent::User(UserEvent::ToggleFocus))
        .await;

    // Typing should not modify input
    app.handle_event(AppEvent::User(UserEvent::InputChar('h')))
        .await;
    app.handle_event(AppEvent::User(UserEvent::InputChar('i')))
        .await;
    assert_eq!(
        app.input, "",
        "input should not change when sidebar focused"
    );

    // Backspace should not modify input
    app.handle_event(AppEvent::User(UserEvent::Backspace)).await;
    assert_eq!(app.input, "");
}

#[tokio::test]
async fn q_quits_when_sidebar_focused() {
    let (db, _dir) = test_db().await;
    let (event_tx, _event_rx) = mpsc::channel::<AppEvent>(256);
    let mut app = make_app(&db, event_tx);
    app.init().await;

    app.handle_event(AppEvent::User(UserEvent::NewConversation))
        .await;
    app.handle_event(AppEvent::User(UserEvent::ToggleFocus))
        .await;

    assert!(!app.should_quit);
    app.handle_event(AppEvent::User(UserEvent::InputChar('q')))
        .await;
    assert!(app.should_quit, "q should quit when sidebar focused");
}

// -----------------------------------------------------------------------
// Edge cases
// -----------------------------------------------------------------------

#[tokio::test]
async fn sidebar_navigation_with_empty_list() {
    let (db, _dir) = test_db().await;
    let (event_tx, _event_rx) = mpsc::channel::<AppEvent>(256);
    let mut app = make_app(&db, event_tx);
    app.init().await;

    // No conversations yet
    app.handle_event(AppEvent::User(UserEvent::ToggleFocus))
        .await;
    assert!(app.sidebar_focus);
    assert_eq!(app.sidebar_selection, 0);

    // Navigation should not panic or change selection
    app.handle_event(AppEvent::User(UserEvent::NavigateDown))
        .await;
    assert_eq!(app.sidebar_selection, 0);

    app.handle_event(AppEvent::User(UserEvent::NavigateUp))
        .await;
    assert_eq!(app.sidebar_selection, 0);

    // Opening with empty list should not panic
    app.handle_event(AppEvent::User(UserEvent::SendMessage))
        .await;
    assert!(app.active_conversation.is_none());
}

#[tokio::test]
async fn new_conversation_while_sidebar_focused() {
    let (db, _dir) = test_db().await;
    let (event_tx, _event_rx) = mpsc::channel::<AppEvent>(256);
    let mut app = make_app(&db, event_tx);
    app.init().await;

    app.handle_event(AppEvent::User(UserEvent::NewConversation))
        .await;
    app.handle_event(AppEvent::User(UserEvent::ToggleFocus))
        .await;
    assert!(app.sidebar_focus);

    // Create new conversation while sidebar is focused
    app.handle_event(AppEvent::User(UserEvent::NewConversation))
        .await;

    // New conversation should be created and active
    assert_eq!(app.conversations.len(), 2);
    assert!(app.active_conversation.is_some());
    // New conversation inserted at 0
    assert_eq!(app.conversations[0].id, app.active_conversation.unwrap());
}
