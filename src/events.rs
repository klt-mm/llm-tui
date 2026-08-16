use crate::domain::{Capabilities, GenerationUsage, Message, Model};
use uuid::Uuid;

#[derive(Debug)]
pub enum UserEvent {
    Quit,
    InputChanged(String),
    SendMessage,
    NewConversation,
    CancelGeneration,
    Retry,
    OpenCommandPalette,
}

#[derive(Debug)]
pub enum ProviderEvent {
    ModelsLoaded(Vec<Model>),
    CapabilitiesLoaded(Capabilities),
    StreamStarted { message_id: Uuid },
    Delta { message_id: Uuid, text: String },
    ReasoningDelta { message_id: Uuid, text: String },
    Usage { message_id: Uuid, usage: GenerationUsage },
    Completed { message: Message },
    Failed { message_id: Uuid, error: String },
}

#[derive(Debug)]
pub enum AppEvent {
    User(UserEvent),
    Provider(ProviderEvent),
}
