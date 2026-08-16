use std::sync::Arc;

use tokio::sync::mpsc;

use llm_tui::app::App;
use llm_tui::config::GenerationConfig;
use llm_tui::domain::Role;
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

#[tokio::test]
async fn e2e_new_conversation_send_and_persist() {
    let (db, _dir) = test_db().await;
    let (event_tx, mut event_rx) = mpsc::channel::<AppEvent>(256);

    let mut app = make_app(&db, event_tx.clone());
    app.init().await;

    assert!(
        app.error.is_none(),
        "init should not error: {:?}",
        app.error
    );
    assert!(app.models.len() >= 2, "FakeProvider should return models");

    app.handle_event(AppEvent::User(UserEvent::NewConversation))
        .await;
    assert!(
        app.error.is_none(),
        "new conversation should not error: {:?}",
        app.error
    );
    assert!(app.active_conversation.is_some());

    app.handle_event(AppEvent::User(UserEvent::InputChar('h')))
        .await;
    app.handle_event(AppEvent::User(UserEvent::InputChar('i')))
        .await;
    app.handle_event(AppEvent::User(UserEvent::SendMessage))
        .await;

    assert!(app.input.is_empty(), "input should be cleared after send");
    assert_eq!(app.messages.len(), 1);
    assert_eq!(app.messages[0].role, Role::User);
    assert_eq!(app.messages[0].content, "hi");

    assert!(app.streaming.is_some(), "generation should be in progress");

    let mut got_completed = false;
    for _ in 0..200 {
        tokio::select! {
            Some(event) = event_rx.recv() => {
                app.handle_event(event).await;
                if app.streaming.is_none() && !app.messages.is_empty()
                    && app.messages.last().unwrap().role == Role::Assistant {
                        got_completed = true;
                        break;
                    }
            }
            _ = tokio::time::sleep(std::time::Duration::from_millis(50)) => {}
        }
        if got_completed {
            break;
        }
    }

    assert!(
        got_completed,
        "should have received completed assistant message"
    );
    assert!(
        app.messages.len() >= 2,
        "should have user + assistant messages"
    );

    let assistant = app.messages.last().unwrap();
    assert_eq!(assistant.role, Role::Assistant);
    assert!(
        !assistant.content.is_empty(),
        "assistant content should not be empty"
    );

    let conv_repo = SqliteConversationRepository::new(db.pool.clone());
    let msg_repo = SqliteMessageRepository::new(db.pool.clone());

    let conversations = conv_repo.list().await.unwrap();
    assert_eq!(conversations.len(), 1);

    let messages = msg_repo
        .list_for_conversation(conversations[0].id)
        .await
        .unwrap();
    assert!(
        messages.len() >= 2,
        "persisted messages should include user + assistant"
    );
}

#[tokio::test]
async fn e2e_restart_restores_conversation() {
    let (db, _dir) = test_db().await;
    let (event_tx, _event_rx) = mpsc::channel::<AppEvent>(256);

    let mut app = make_app(&db, event_tx.clone());
    app.init().await;

    app.handle_event(AppEvent::User(UserEvent::NewConversation))
        .await;
    let conv_id = app.active_conversation.unwrap();

    let conv_repo = SqliteConversationRepository::new(db.pool.clone());

    let conversations = conv_repo.list().await.unwrap();
    assert_eq!(conversations.len(), 1);

    let (event_tx2, _event_rx2) = mpsc::channel::<AppEvent>(256);
    let mut app2 = make_app(&db, event_tx2);
    app2.init().await;

    assert_eq!(app2.conversations.len(), 1);
    assert_eq!(app2.conversations[0].id, conv_id);
}

#[tokio::test]
async fn e2e_cancel_generation() {
    let (db, _dir) = test_db().await;
    let (event_tx, mut event_rx) = mpsc::channel::<AppEvent>(256);

    let mut app = make_app(&db, event_tx.clone());
    app.init().await;

    app.handle_event(AppEvent::User(UserEvent::NewConversation))
        .await;
    app.handle_event(AppEvent::User(UserEvent::InputChar('t')))
        .await;
    app.handle_event(AppEvent::User(UserEvent::SendMessage))
        .await;

    assert!(app.streaming.is_some());

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    app.handle_event(AppEvent::User(UserEvent::CancelGeneration))
        .await;

    let mut done = false;
    for _ in 0..100 {
        tokio::select! {
            Some(event) = event_rx.recv() => {
                app.handle_event(event).await;
                if app.streaming.is_none() {
                    done = true;
                    break;
                }
            }
            _ = tokio::time::sleep(std::time::Duration::from_millis(50)) => {}
        }
        if done {
            break;
        }
    }

    assert!(done, "streaming should stop after cancellation");
}
