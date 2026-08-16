mod tui;

use std::sync::Arc;

use anyhow::Result;
use tokio::sync::mpsc;

use llm_tui::app::App;
use llm_tui::config::Config;
use llm_tui::events::AppEvent;
use llm_tui::llm::{FakeProvider, OpenAiCompatibleProvider};
use llm_tui::persistence::{
    Database, SqliteConversationRepository, SqliteMessageRepository, SqliteModelRepository,
    SqliteProviderRepository,
};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(std::env::var("RUST_LOG").unwrap_or_else(|_| "llm_tui=debug".to_string()))
        .init();

    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://llm-tui.db".to_string());

    let db = Database::connect(&database_url).await?;
    let config = Config::load();

    let conversation_repo = Arc::new(SqliteConversationRepository::new(db.pool.clone()));
    let message_repo = Arc::new(SqliteMessageRepository::new(db.pool.clone()));
    let model_repo = Arc::new(SqliteModelRepository::new(db.pool.clone()));
    let provider_repo = Arc::new(SqliteProviderRepository::new(db.pool.clone()));

    let (provider, provider_name): (Arc<dyn llm_tui::llm::LlmProvider>, String) =
        if config.is_provider_configured() {
            let base_url = config.provider.base_url.clone().unwrap();
            let name = config
                .provider
                .name
                .clone()
                .unwrap_or_else(|| base_url.clone());

            let domain_provider = llm_tui::domain::Provider {
                id: uuid::Uuid::new_v4(),
                name: name.clone(),
                base_url: base_url.clone(),
                protocol: llm_tui::domain::ProviderProtocol::OpenAiCompatible,
                api_key_ref: config.provider.api_key.clone(),
                default_model: config.provider.default_model.clone(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            };

            match OpenAiCompatibleProvider::new(domain_provider) {
                Ok(p) => {
                    tracing::info!(url = %base_url, "using OpenAI-compatible provider");
                    (Arc::new(p), name)
                }
                Err(e) => {
                    tracing::warn!(%e, "failed to create provider, falling back to fake");
                    (Arc::new(FakeProvider::new()), "fake".into())
                }
            }
        } else {
            tracing::info!("no provider configured, using fake provider");
            (Arc::new(FakeProvider::new()), "fake".into())
        };

    let (event_tx, _event_rx) = mpsc::channel::<AppEvent>(256);

    let mut app = App::new(
        provider,
        provider_name,
        conversation_repo,
        message_repo,
        model_repo,
        provider_repo,
        config.generation,
        event_tx,
    );

    tui::run(&mut app).await
}
