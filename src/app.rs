use std::sync::Arc;

use chrono::Utc;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing::{debug, warn};
use uuid::Uuid;

use crate::domain::{Conversation, GenerationParameters, Message, Model, Provider, ProviderProtocol, Role};
use crate::events::{AppEvent, ProviderEvent, UserEvent};
use crate::llm::provider::{ChatRequest, LlmProvider, StreamEvent};
use crate::persistence::repositories::{ConversationRepository, MessageRepository, ProviderRepository};

pub struct StreamingState {
    pub message_id: Uuid,
    pub buffer: String,
    pub cancel: CancellationToken,
}

pub struct App {
    pub provider: Arc<dyn LlmProvider>,
    pub models: Vec<Model>,
    pub selected_model: Option<String>,
    pub conversations: Vec<Conversation>,
    pub active_conversation: Option<Uuid>,
    pub messages: Vec<Message>,
    pub streaming: Option<StreamingState>,
    pub input: String,
    pub should_quit: bool,
    pub error: Option<String>,
    provider_id: Uuid,
    conversation_repo: Arc<dyn ConversationRepository>,
    message_repo: Arc<dyn MessageRepository>,
    provider_repo: Arc<dyn ProviderRepository>,
    event_tx: mpsc::Sender<AppEvent>,
}

impl App {
    pub fn new(
        provider: Arc<dyn LlmProvider>,
        conversation_repo: Arc<dyn ConversationRepository>,
        message_repo: Arc<dyn MessageRepository>,
        provider_repo: Arc<dyn ProviderRepository>,
        event_tx: mpsc::Sender<AppEvent>,
    ) -> Self {
        Self {
            provider,
            models: Vec::new(),
            selected_model: None,
            conversations: Vec::new(),
            active_conversation: None,
            messages: Vec::new(),
            streaming: None,
            input: String::new(),
            should_quit: false,
            error: None,
            provider_id: Uuid::nil(),
            conversation_repo,
            message_repo,
            provider_repo,
            event_tx,
        }
    }

    pub async fn set_event_tx(&mut self, tx: mpsc::Sender<AppEvent>) {
        self.event_tx = tx;
    }

    pub async fn init(&mut self) {
        let providers = self.provider_repo.list().await.unwrap_or_default();
        if let Some(p) = providers.first() {
            self.provider_id = p.id;
        } else {
            let provider = Provider {
                id: Uuid::new_v4(),
                name: "default".into(),
                base_url: "fake://local".into(),
                protocol: ProviderProtocol::OpenAiCompatible,
                api_key_ref: None,
                default_model: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };
            if let Err(e) = self.provider_repo.create(&provider).await {
                warn!(%e, "failed to seed default provider");
            }
            self.provider_id = provider.id;
        }

        match self.provider.models().await {
            Ok(models) => {
                debug!(count = models.len(), "loaded models");
                self.models = models;
                if let Some(first) = self.models.first() {
                    self.selected_model = Some(first.id.clone());
                }
            }
            Err(e) => {
                warn!(%e, "failed to load models");
                self.error = Some(format!("Failed to load models: {e}"));
            }
        }

        match self.conversation_repo.list().await {
            Ok(conversations) => self.conversations = conversations,
            Err(e) => {
                warn!(%e, "failed to list conversations");
                self.error = Some(format!("Failed to load conversations: {e}"));
            }
        }
    }

    pub async fn handle_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::User(user_event) => self.handle_user_event(user_event).await,
            AppEvent::Provider(provider_event) => self.handle_provider_event(provider_event).await,
        }
    }

    async fn handle_user_event(&mut self, event: UserEvent) {
        match event {
            UserEvent::Quit => {
                self.should_quit = true;
            }
            UserEvent::InputChar(c) => {
                self.input.push(c);
            }
            UserEvent::Backspace => {
                self.input.pop();
            }
            UserEvent::SendMessage => {
                self.send_message().await;
            }
            UserEvent::NewConversation => {
                self.new_conversation().await;
            }
            UserEvent::CancelGeneration => {
                self.cancel_generation();
            }
            UserEvent::Retry => {
                self.retry_generation().await;
            }
            UserEvent::OpenCommandPalette => {
                debug!("command palette requested (not yet implemented)");
            }
        }
    }

    async fn handle_provider_event(&mut self, event: ProviderEvent) {
        match event {
            ProviderEvent::StreamStarted { message_id } => {
                debug!(%message_id, "stream started");
                self.streaming = Some(StreamingState {
                    message_id,
                    buffer: String::new(),
                    cancel: CancellationToken::new(),
                });
            }
            ProviderEvent::Delta { message_id, text } => {
                if let Some(ref mut state) = self.streaming {
                    if state.message_id == message_id {
                        state.buffer.push_str(&text);
                    }
                }
            }
            ProviderEvent::ReasoningDelta { .. } => {}
            ProviderEvent::Usage { .. } => {}
            ProviderEvent::Completed { message } => {
                debug!(message_id = %message.id, "generation completed");
                if let Err(e) = self.message_repo.create(&message).await {
                    self.error = Some(format!("Failed to persist message: {e}"));
                }
                self.messages.push(message);
                self.streaming = None;
                self.touch_conversation().await;
            }
            ProviderEvent::Failed { message_id, error } => {
                warn!(%message_id, %error, "generation failed");
                self.error = Some(error);
                self.streaming = None;
            }
            ProviderEvent::ModelsLoaded(models) => {
                self.models = models;
            }
            ProviderEvent::CapabilitiesLoaded(_) => {}
        }
    }

    async fn send_message(&mut self) {
        let input = self.input.clone();
        if input.trim().is_empty() {
            return;
        }
        if self.streaming.is_some() {
            return;
        }

        let conversation_id = match self.active_conversation {
            Some(id) => id,
            None => {
                self.new_conversation().await;
                self.active_conversation.unwrap()
            }
        };

        let user_message = Message {
            id: Uuid::new_v4(),
            conversation_id,
            parent_id: self.messages.last().map(|m| m.id),
            role: Role::User,
            content: input,
            reasoning_content: None,
            metadata: serde_json::json!({}),
            created_at: Utc::now(),
        };

        if let Err(e) = self.message_repo.create(&user_message).await {
            self.error = Some(format!("Failed to save message: {e}"));
            return;
        }
        self.messages.push(user_message);
        self.input.clear();
        self.error = None;

        self.start_generation(conversation_id).await;
    }

    async fn start_generation(&mut self, conversation_id: Uuid) {
        let model = match &self.selected_model {
            Some(m) => m.clone(),
            None => {
                self.error = Some("No model selected".into());
                return;
            }
        };

        let request = ChatRequest {
            model,
            messages: self.messages.clone(),
            generation: GenerationParameters::default(),
        };

        let cancel = CancellationToken::new();
        let assistant_message_id = Uuid::new_v4();

        self.streaming = Some(StreamingState {
            message_id: assistant_message_id,
            buffer: String::new(),
            cancel: cancel.clone(),
        });

        let provider = Arc::clone(&self.provider);
        let event_tx = self.event_tx.clone();

        let _ = event_tx
            .send(AppEvent::Provider(ProviderEvent::StreamStarted {
                message_id: assistant_message_id,
            }))
            .await;

        tokio::spawn(async move {
            match provider.stream_chat(request, cancel).await {
                Ok(mut rx) => {
                    let mut content = String::new();
                    while let Some(event) = rx.recv().await {
                        match event {
                            Ok(StreamEvent::Delta(text)) => {
                                content.push_str(&text);
                                let _ = event_tx
                                    .send(AppEvent::Provider(ProviderEvent::Delta {
                                        message_id: assistant_message_id,
                                        text,
                                    }))
                                    .await;
                            }
                            Ok(StreamEvent::ReasoningDelta(text)) => {
                                let _ = event_tx
                                    .send(AppEvent::Provider(ProviderEvent::ReasoningDelta {
                                        message_id: assistant_message_id,
                                        text,
                                    }))
                                    .await;
                            }
                            Ok(StreamEvent::Usage(usage)) => {
                                let _ = event_tx
                                    .send(AppEvent::Provider(ProviderEvent::Usage {
                                        message_id: assistant_message_id,
                                        usage,
                                    }))
                                    .await;
                            }
                            Ok(StreamEvent::Completed) => {
                                let assistant = Message {
                                    id: assistant_message_id,
                                    conversation_id,
                                    parent_id: None,
                                    role: Role::Assistant,
                                    content,
                                    reasoning_content: None,
                                    metadata: serde_json::json!({}),
                                    created_at: Utc::now(),
                                };
                                let _ = event_tx
                                    .send(AppEvent::Provider(ProviderEvent::Completed {
                                        message: assistant,
                                    }))
                                    .await;
                                return;
                            }
                            Err(e) => {
                                let _ = event_tx
                                    .send(AppEvent::Provider(ProviderEvent::Failed {
                                        message_id: assistant_message_id,
                                        error: e.to_string(),
                                    }))
                                    .await;
                                return;
                            }
                        }
                    }
                    let assistant = Message {
                        id: assistant_message_id,
                        conversation_id,
                        parent_id: None,
                        role: Role::Assistant,
                        content,
                        reasoning_content: None,
                        metadata: serde_json::json!({}),
                        created_at: Utc::now(),
                    };
                    let _ = event_tx
                        .send(AppEvent::Provider(ProviderEvent::Completed {
                            message: assistant,
                        }))
                        .await;
                }
                Err(e) => {
                    let _ = event_tx
                        .send(AppEvent::Provider(ProviderEvent::Failed {
                            message_id: assistant_message_id,
                            error: e.to_string(),
                        }))
                        .await;
                }
            }
        });
    }

    async fn new_conversation(&mut self) {
        let model_id = self
            .selected_model
            .clone()
            .unwrap_or_else(|| "default".into());

        let conversation = Conversation {
            id: Uuid::new_v4(),
            title: "New Conversation".into(),
            provider_id: self.provider_id,
            model_id,
            system_prompt: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            archived_at: None,
        };

        if let Err(e) = self.conversation_repo.create(&conversation).await {
            self.error = Some(format!("Failed to create conversation: {e}"));
            return;
        }

        self.conversations.insert(0, conversation.clone());
        self.active_conversation = Some(conversation.id);
        self.messages.clear();
        self.error = None;
        debug!(id = %conversation.id, "created new conversation");
    }

    fn cancel_generation(&mut self) {
        if let Some(ref state) = self.streaming {
            state.cancel.cancel();
            debug!(message_id = %state.message_id, "cancellation requested");
        }
    }

    async fn retry_generation(&mut self) {
        self.cancel_generation();
        self.streaming = None;

        let conversation_id = match self.active_conversation {
            Some(id) => id,
            None => return,
        };

        if let Some(last) = self.messages.last() {
            if last.role == Role::Assistant {
                self.messages.pop();
            }
        }

        self.start_generation(conversation_id).await;
    }

    async fn touch_conversation(&mut self) {
        let Some(conversation_id) = self.active_conversation else {
            return;
        };
        if let Some(conv) = self.conversations.iter_mut().find(|c| c.id == conversation_id) {
            conv.updated_at = Utc::now();
            if self.messages.len() == 2 && conv.title == "New Conversation" {
                if let Some(user_msg) = self.messages.iter().find(|m| m.role == Role::User) {
                    conv.title = user_msg.content.chars().take(50).collect();
                }
            }
        }
    }

    pub fn streaming_content(&self) -> Option<&str> {
        self.streaming.as_ref().map(|s| s.buffer.as_str())
    }
}
