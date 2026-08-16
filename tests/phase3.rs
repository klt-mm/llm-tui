use std::sync::Arc;

use chrono::Utc;
use tokio::sync::mpsc;
use uuid::Uuid;

use llm_tui::app::{ActiveScreen, App, Modal};
use llm_tui::config::{ContextConfig, GenerationConfig};
use llm_tui::domain::*;
use llm_tui::events::{AppEvent, UserEvent};
use llm_tui::llm::FakeProvider;
use llm_tui::persistence::repositories::*;
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

fn make_prompt(name: &str, content: &str) -> Prompt {
    Prompt {
        id: Uuid::new_v4(),
        name: name.into(),
        description: Some(format!("{} description", name)),
        content: content.into(),
        system_prompt: None,
        tags: vec!["test".into()],
        variables: vec![],
        created_at: Utc::now(),
        updated_at: Utc::now(),
    }
}

// -----------------------------------------------------------------------
// Prompt Repository CRUD
// -----------------------------------------------------------------------

#[tokio::test]
async fn prompt_list_returns_all() {
    let (db, _dir) = test_db().await;
    let repo = SqlitePromptRepository::new(db.pool.clone());

    let p1 = make_prompt("alpha", "first prompt");
    let p2 = make_prompt("beta", "second prompt");
    repo.create(&p1).await.unwrap();
    repo.create(&p2).await.unwrap();

    let list = repo.list().await.unwrap();
    assert_eq!(list.len(), 2);
}

#[tokio::test]
async fn prompt_update_changes_content() {
    let (db, _dir) = test_db().await;
    let repo = SqlitePromptRepository::new(db.pool.clone());

    let mut prompt = make_prompt("original", "original content");
    repo.create(&prompt).await.unwrap();

    prompt.content = "updated content".into();
    prompt.name = "updated".into();
    prompt.tags = vec!["updated".into()];
    repo.update(&prompt).await.unwrap();

    let loaded = repo.get(prompt.id).await.unwrap().unwrap();
    assert_eq!(loaded.name, "updated");
    assert_eq!(loaded.content, "updated content");
    assert_eq!(loaded.tags, vec!["updated"]);
}

#[tokio::test]
async fn prompt_delete_removes_prompt() {
    let (db, _dir) = test_db().await;
    let repo = SqlitePromptRepository::new(db.pool.clone());

    let prompt = make_prompt("to-delete", "will be removed");
    repo.create(&prompt).await.unwrap();
    assert_eq!(repo.list().await.unwrap().len(), 1);

    repo.delete(prompt.id).await.unwrap();
    assert_eq!(repo.list().await.unwrap().len(), 0);
}

// -----------------------------------------------------------------------
// FTS Search
// -----------------------------------------------------------------------

#[tokio::test]
async fn prompt_fts_search_finds_by_name() {
    let (db, _dir) = test_db().await;
    let repo = SqlitePromptRepository::new(db.pool.clone());

    repo.create(&make_prompt("rust tutorial", "learn rust"))
        .await
        .unwrap();
    repo.create(&make_prompt("python guide", "learn python"))
        .await
        .unwrap();

    let results = repo.search("rust", 10).await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].prompt.name, "rust tutorial");
}

#[tokio::test]
async fn prompt_fts_search_finds_by_content() {
    let (db, _dir) = test_db().await;
    let repo = SqlitePromptRepository::new(db.pool.clone());

    repo.create(&make_prompt("p1", "the quick brown fox"))
        .await
        .unwrap();
    repo.create(&make_prompt("p2", "lazy dog sleeps"))
        .await
        .unwrap();

    let results = repo.search("fox", 10).await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].prompt.name, "p1");
}

#[tokio::test]
async fn prompt_fts_search_empty_query_returns_empty() {
    let (db, _dir) = test_db().await;
    let repo = SqlitePromptRepository::new(db.pool.clone());

    repo.create(&make_prompt("test", "content")).await.unwrap();

    let results = repo.search("", 10).await.unwrap();
    assert!(results.is_empty());
}

#[tokio::test]
async fn message_fts_search_finds_messages() {
    let (db, _dir) = test_db().await;
    let conv_repo = SqliteConversationRepository::new(db.pool.clone());
    let msg_repo = SqliteMessageRepository::new(db.pool.clone());

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
    SqliteProviderRepository::new(db.pool.clone())
        .create(&provider)
        .await
        .unwrap();

    let conv = Conversation {
        id: Uuid::new_v4(),
        title: "Search Test Conv".into(),
        provider_id: provider.id,
        model_id: "test".into(),
        system_prompt: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        archived_at: None,
    };
    conv_repo.create(&conv).await.unwrap();

    let msg = Message {
        id: Uuid::new_v4(),
        conversation_id: conv.id,
        parent_id: None,
        role: Role::User,
        content: "hello world search test".into(),
        reasoning_content: None,
        metadata: serde_json::json!({}),
        created_at: Utc::now(),
    };
    msg_repo.create(&msg).await.unwrap();

    let results = msg_repo.search("search", 10).await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].conversation_title, "Search Test Conv");
    assert_eq!(results[0].message.content, "hello world search test");
}

#[tokio::test]
async fn conversation_search_titles() {
    let (db, _dir) = test_db().await;
    let repo = SqliteConversationRepository::new(db.pool.clone());

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
    SqliteProviderRepository::new(db.pool.clone())
        .create(&provider)
        .await
        .unwrap();

    let c1 = Conversation {
        id: Uuid::new_v4(),
        title: "Rust Programming".into(),
        provider_id: provider.id,
        model_id: "test".into(),
        system_prompt: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        archived_at: None,
    };
    let c2 = Conversation {
        id: Uuid::new_v4(),
        title: "Python Scripts".into(),
        provider_id: provider.id,
        model_id: "test".into(),
        system_prompt: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        archived_at: None,
    };
    repo.create(&c1).await.unwrap();
    repo.create(&c2).await.unwrap();

    let results = repo.search_titles("rust").await.unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "Rust Programming");
}

// -----------------------------------------------------------------------
// App-level prompt flows
// -----------------------------------------------------------------------

#[tokio::test]
async fn prompt_picker_opens_and_selects() {
    let (db, _dir) = test_db().await;
    let (event_tx, _event_rx) = mpsc::channel::<AppEvent>(256);
    let mut app = make_app(&db, event_tx);
    app.init().await;

    // Seed a prompt
    let prompt_repo = SqlitePromptRepository::new(db.pool.clone());
    let prompt = make_prompt("greeting", "Hello there!");
    prompt_repo.create(&prompt).await.unwrap();
    app.prompts = prompt_repo.list().await.unwrap();

    // Open picker
    app.handle_event(AppEvent::User(UserEvent::OpenPromptPicker))
        .await;
    assert!(matches!(app.modal, Modal::PromptPicker { .. }));

    // Select the prompt (Enter)
    app.handle_event(AppEvent::User(UserEvent::SendMessage))
        .await;

    // Prompt content should be in the input buffer
    assert_eq!(app.input, "Hello there!");
    assert!(matches!(app.modal, Modal::None));
    assert!(matches!(app.active_screen, ActiveScreen::Chat));
}

#[tokio::test]
async fn prompt_with_variables_shows_input() {
    let (db, _dir) = test_db().await;
    let (event_tx, _event_rx) = mpsc::channel::<AppEvent>(256);
    let mut app = make_app(&db, event_tx);
    app.init().await;

    // Seed a prompt with variables
    let prompt_repo = SqlitePromptRepository::new(db.pool.clone());
    let mut prompt = make_prompt("template", "Hello {{name}}, welcome to {{place}}!");
    prompt.variables = vec!["name".into(), "place".into()];
    prompt_repo.create(&prompt).await.unwrap();
    app.prompts = prompt_repo.list().await.unwrap();

    // Open picker and select
    app.handle_event(AppEvent::User(UserEvent::OpenPromptPicker))
        .await;
    app.handle_event(AppEvent::User(UserEvent::SendMessage))
        .await;

    // Should show variable input modal
    assert!(matches!(app.modal, Modal::VariableInput { .. }));

    // Type values for variables
    app.handle_event(AppEvent::User(UserEvent::InputChar('A')))
        .await;
    app.handle_event(AppEvent::User(UserEvent::InputChar('l')))
        .await;
    app.handle_event(AppEvent::User(UserEvent::InputChar('i')))
        .await;
    app.handle_event(AppEvent::User(UserEvent::InputChar('c')))
        .await;
    app.handle_event(AppEvent::User(UserEvent::InputChar('e')))
        .await;

    // Move to next variable
    app.handle_event(AppEvent::User(UserEvent::PromptFieldNext))
        .await;

    // Type second value
    app.handle_event(AppEvent::User(UserEvent::InputChar('M')))
        .await;
    app.handle_event(AppEvent::User(UserEvent::InputChar('a')))
        .await;
    app.handle_event(AppEvent::User(UserEvent::InputChar('r')))
        .await;
    app.handle_event(AppEvent::User(UserEvent::InputChar('s')))
        .await;

    // Confirm
    app.handle_event(AppEvent::User(UserEvent::SendMessage))
        .await;

    assert_eq!(app.input, "Hello Alice, welcome to Mars!");
    assert!(matches!(app.active_screen, ActiveScreen::Chat));
}

#[tokio::test]
async fn search_screen_opens_and_searches() {
    let (db, _dir) = test_db().await;
    let (event_tx, _event_rx) = mpsc::channel::<AppEvent>(256);
    let mut app = make_app(&db, event_tx);
    app.init().await;

    // Seed a prompt
    let prompt_repo = SqlitePromptRepository::new(db.pool.clone());
    prompt_repo
        .create(&make_prompt("rust helper", "help with rust programming"))
        .await
        .unwrap();
    app.prompts = prompt_repo.list().await.unwrap();

    // Open search
    app.handle_event(AppEvent::User(UserEvent::OpenSearch))
        .await;
    assert!(matches!(app.active_screen, ActiveScreen::Search));

    // Type search query — each char triggers a search
    for c in "rust".chars() {
        app.handle_event(AppEvent::User(UserEvent::InputChar(c)))
            .await;
    }

    // Should have found the prompt
    assert!(!app.search_results.is_empty());
}

#[tokio::test]
async fn slash_opens_search_when_input_empty() {
    let (db, _dir) = test_db().await;
    let (event_tx, _event_rx) = mpsc::channel::<AppEvent>(256);
    let mut app = make_app(&db, event_tx);
    app.init().await;

    assert!(app.input.is_empty());
    assert!(matches!(app.active_screen, ActiveScreen::Chat));

    // Type / with empty input
    app.handle_event(AppEvent::User(UserEvent::InputChar('/')))
        .await;

    assert!(matches!(app.active_screen, ActiveScreen::Search));
}

#[tokio::test]
async fn slash_types_normally_when_input_not_empty() {
    let (db, _dir) = test_db().await;
    let (event_tx, _event_rx) = mpsc::channel::<AppEvent>(256);
    let mut app = make_app(&db, event_tx);
    app.init().await;

    app.input = "hello".into();

    app.handle_event(AppEvent::User(UserEvent::InputChar('/')))
        .await;

    // Should NOT open search, should append to input
    assert!(matches!(app.active_screen, ActiveScreen::Chat));
    assert_eq!(app.input, "hello/");
}

#[tokio::test]
async fn prompt_editor_creates_new_prompt() {
    let (db, _dir) = test_db().await;
    let (event_tx, _event_rx) = mpsc::channel::<AppEvent>(256);
    let mut app = make_app(&db, event_tx);
    app.init().await;

    // Open new prompt editor
    app.handle_event(AppEvent::User(UserEvent::PromptNew)).await;
    assert!(matches!(
        app.modal,
        Modal::PromptEditor {
            editing_id: None,
            ..
        }
    ));

    // Fill in name field (field 0)
    for c in "my-prompt".chars() {
        app.handle_event(AppEvent::User(UserEvent::InputChar(c)))
            .await;
    }

    // Move to content field (field 2)
    app.handle_event(AppEvent::User(UserEvent::PromptFieldNext))
        .await; // field 1
    app.handle_event(AppEvent::User(UserEvent::PromptFieldNext))
        .await; // field 2

    // Type content
    for c in "do the thing".chars() {
        app.handle_event(AppEvent::User(UserEvent::InputChar(c)))
            .await;
    }

    // Save
    app.handle_event(AppEvent::User(UserEvent::SendMessage))
        .await;

    assert!(matches!(app.modal, Modal::None));
    assert_eq!(app.prompts.len(), 1);
    assert_eq!(app.prompts[0].name, "my-prompt");
    assert_eq!(app.prompts[0].content, "do the thing");

    // Verify persisted
    let repo = SqlitePromptRepository::new(db.pool.clone());
    let stored = repo.list().await.unwrap();
    assert_eq!(stored.len(), 1);
    assert_eq!(stored[0].name, "my-prompt");
}

#[tokio::test]
async fn prompts_screen_navigation() {
    let (db, _dir) = test_db().await;
    let (event_tx, _event_rx) = mpsc::channel::<AppEvent>(256);
    let mut app = make_app(&db, event_tx);
    app.init().await;

    // Seed prompts
    let prompt_repo = SqlitePromptRepository::new(db.pool.clone());
    prompt_repo
        .create(&make_prompt("first", "content 1"))
        .await
        .unwrap();
    prompt_repo
        .create(&make_prompt("second", "content 2"))
        .await
        .unwrap();
    app.prompts = prompt_repo.list().await.unwrap();

    // Open prompts screen
    app.handle_event(AppEvent::User(UserEvent::OpenPromptList))
        .await;
    assert!(matches!(app.active_screen, ActiveScreen::Prompts));

    // Navigate with j/k
    app.handle_event(AppEvent::User(UserEvent::InputChar('j')))
        .await;
    assert_eq!(app.prompt_selection, 1);

    app.handle_event(AppEvent::User(UserEvent::InputChar('k')))
        .await;
    assert_eq!(app.prompt_selection, 0);

    // q returns to chat
    app.handle_event(AppEvent::User(UserEvent::InputChar('q')))
        .await;
    assert!(matches!(app.active_screen, ActiveScreen::Chat));
}
