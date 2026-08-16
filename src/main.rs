mod app;
mod domain;
mod events;
mod llm;
mod persistence;
mod tui;

use anyhow::Result;

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

    let db = persistence::Database::connect(&database_url).await?;
    let mut app = app::App::new(db);

    tui::run(&mut app).await
}
