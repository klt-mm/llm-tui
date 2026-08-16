mod tui;

use std::sync::Arc;

use anyhow::Result;
use tokio::sync::mpsc;

use llm_tui::app::App;
use llm_tui::events::AppEvent;
use llm_tui::llm::FakeProvider;
use llm_tui::persistence::{
    SqliteConversationRepository, SqliteMessageRepository, SqliteProviderRepository, Database,
};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "llm_tui=debug".to_string()),
        )
        .init();

    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://llm-tui.db".to_string());

    let db = Database::connect(&database_url).await?;

    let conversation_repo = Arc::new(SqliteConversationRepository::new(db.pool.clone()));
    let message_repo = Arc::new(SqliteMessageRepository::new(db.pool.clone()));
    let provider_repo = Arc::new(SqliteProviderRepository::new(db.pool.clone()));

    let provider = Arc::new(FakeProvider::new());

    let (event_tx, _event_rx) = mpsc::channel::<AppEvent>(256);

    let mut app = App::new(provider, conversation_repo, message_repo, provider_repo, event_tx);

    tui::run(&mut app).await
}
