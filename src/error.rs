use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("provider error in {operation}: {message}")]
    Provider {
        operation: String,
        provider: String,
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("persistence error in {operation}: {message}")]
    Persistence {
        operation: String,
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    #[error("configuration error: {0}")]
    Config(String),

    #[error("generation error: {0}")]
    Generation(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl AppError {
    pub fn provider(
        operation: impl Into<String>,
        provider: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::Provider {
            operation: operation.into(),
            provider: provider.into(),
            message: message.into(),
            source: None,
        }
    }

    pub fn persistence(operation: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Persistence {
            operation: operation.into(),
            message: message.into(),
            source: None,
        }
    }

    pub fn safe_message(&self) -> String {
        match self {
            Self::Provider {
                operation, message, ..
            } => {
                format!("{operation} failed: {message}")
            }
            Self::Persistence {
                operation, message, ..
            } => {
                format!("{operation} failed: {message}")
            }
            Self::Config(msg) => msg.clone(),
            Self::Generation(msg) => msg.clone(),
            Self::Other(e) => format!("{e}"),
        }
    }
}
