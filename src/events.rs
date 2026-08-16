use crate::domain::{Capabilities, GenerationUsage, Message, Model};
use uuid::Uuid;

#[derive(Debug)]
pub enum UserEvent {
    Quit,
    InputChar(char),
    Backspace,
    SendMessage,
    NewConversation,
    CancelGeneration,
    Retry,
    TestConnection,
    SelectModel(usize),
    OpenCommandPalette,
    NavigateUp,
    NavigateDown,
    OpenSelected,
    ToggleFocus,
    StartRename,
    StartDelete,
    ConfirmAction,
    CancelModal,
    OpenHelp,
    OpenPromptPicker,
    OpenPromptList,
    OpenSearch,
    SearchNavigateUp,
    SearchNavigateDown,
    SearchOpenResult,
    PromptNew,
    PromptEditSelected(usize),
    PromptDeleteConfirm,
    PromptFieldNext,
    PromptFieldPrev,
}

#[derive(Debug)]
pub enum ProviderEvent {
    ModelsLoaded(Vec<Model>),
    CapabilitiesLoaded(Capabilities),
    StreamStarted {
        message_id: Uuid,
    },
    Delta {
        message_id: Uuid,
        text: String,
    },
    ReasoningDelta {
        message_id: Uuid,
        text: String,
    },
    Usage {
        message_id: Uuid,
        usage: GenerationUsage,
    },
    Completed {
        message: Message,
    },
    Failed {
        message_id: Uuid,
        error: String,
    },
}

#[derive(Debug)]
pub enum AppEvent {
    User(UserEvent),
    Provider(ProviderEvent),
}
