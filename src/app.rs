use std::sync::Arc;

use chrono::Utc;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tracing::{debug, warn};
use uuid::Uuid;

use crate::config::GenerationConfig;
use crate::domain::{
    Conversation, GenerationParameters, GenerationUsage, Message, Model, Prompt, Provider,
    ProviderProtocol, Role,
};
use crate::events::{AppEvent, ProviderEvent, UserEvent};
use crate::llm::provider::{ChatRequest, LlmProvider, StreamEvent};
use crate::persistence::repositories::{
    ConversationRepository, MessageRepository, ModelRepository, PromptRepository,
    ProviderRepository,
};

pub struct StreamingState {
    pub message_id: Uuid,
    pub buffer: String,
    pub cancel: CancellationToken,
    pub started_at: std::time::Instant,
    pub usage: Option<GenerationUsage>,
}

pub enum Modal {
    None,
    Rename {
        buffer: String,
    },
    DeleteConfirm {
        conversation_id: Uuid,
        title: String,
    },
    Help,
    CommandPalette {
        query: String,
        selected: usize,
    },
    PromptPicker {
        query: String,
        selected: usize,
        filtered: Vec<usize>,
    },
    PromptEditor {
        editing_id: Option<Uuid>,
        name: String,
        description: String,
        content: String,
        system_prompt: String,
        tags: String,
        variables: String,
        field: usize,
    },
    VariableInput {
        prompt_content: String,
        variables: Vec<String>,
        values: Vec<String>,
        current: usize,
        send_immediately: bool,
    },
    PromptDeleteConfirm {
        prompt_id: Uuid,
        name: String,
    },
}

pub enum ActiveScreen {
    Chat,
    Search,
    Prompts,
}

pub struct App {
    pub provider: Arc<dyn LlmProvider>,
    pub provider_name: String,
    pub models: Vec<Model>,
    pub selected_model: Option<String>,
    pub conversations: Vec<Conversation>,
    pub active_conversation: Option<Uuid>,
    pub messages: Vec<Message>,
    pub streaming: Option<StreamingState>,
    pub input: String,
    pub should_quit: bool,
    pub error: Option<String>,
    pub generation: GenerationConfig,
    pub sidebar_focus: bool,
    pub sidebar_selection: usize,
    pub modal: Modal,
    pub active_screen: ActiveScreen,
    pub prompts: Vec<Prompt>,
    pub prompt_selection: usize,
    pub search_query: String,
    pub search_results: Vec<SearchResultEntry>,
    pub search_selection: usize,
    provider_id: Uuid,
    conversation_repo: Arc<dyn ConversationRepository>,
    message_repo: Arc<dyn MessageRepository>,
    model_repo: Arc<dyn ModelRepository>,
    provider_repo: Arc<dyn ProviderRepository>,
    prompt_repo: Arc<dyn PromptRepository>,
    event_tx: mpsc::Sender<AppEvent>,
}

pub enum SearchResultEntry {
    Message {
        message: Message,
        conversation_id: Uuid,
        conversation_title: String,
    },
    Prompt {
        prompt: Prompt,
    },
}

impl App {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        provider: Arc<dyn LlmProvider>,
        provider_name: String,
        conversation_repo: Arc<dyn ConversationRepository>,
        message_repo: Arc<dyn MessageRepository>,
        model_repo: Arc<dyn ModelRepository>,
        provider_repo: Arc<dyn ProviderRepository>,
        prompt_repo: Arc<dyn PromptRepository>,
        generation: GenerationConfig,
        event_tx: mpsc::Sender<AppEvent>,
    ) -> Self {
        Self {
            provider,
            provider_name,
            models: Vec::new(),
            selected_model: None,
            conversations: Vec::new(),
            active_conversation: None,
            messages: Vec::new(),
            streaming: None,
            input: String::new(),
            should_quit: false,
            error: None,
            generation,
            sidebar_focus: false,
            sidebar_selection: 0,
            modal: Modal::None,
            active_screen: ActiveScreen::Chat,
            prompts: Vec::new(),
            prompt_selection: 0,
            search_query: String::new(),
            search_results: Vec::new(),
            search_selection: 0,
            provider_id: Uuid::nil(),
            conversation_repo,
            message_repo,
            model_repo,
            provider_repo,
            prompt_repo,
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
                name: self.provider_name.clone(),
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

        self.refresh_models().await;

        match self.conversation_repo.list().await {
            Ok(conversations) => self.conversations = conversations,
            Err(e) => {
                warn!(%e, "failed to list conversations");
                self.error = Some(format!("Failed to load conversations: {e}"));
            }
        }

        match self.prompt_repo.list().await {
            Ok(prompts) => self.prompts = prompts,
            Err(e) => {
                warn!(%e, "failed to load prompts");
            }
        }
    }

    pub async fn refresh_models(&mut self) {
        match self.provider.models().await {
            Ok(models) => {
                debug!(count = models.len(), "loaded models");
                let _ = self.model_repo.upsert(self.provider_id, &models).await;
                self.models = models;
                if let Some(first) = self.models.first() {
                    self.selected_model = Some(first.id.clone());
                }
            }
            Err(e) => {
                warn!(%e, "failed to load models");
                if let Ok(cached) = self.model_repo.list_for_provider(self.provider_id).await
                    && !cached.is_empty()
                {
                    self.models = cached;
                    if let Some(first) = self.models.first() {
                        self.selected_model = Some(first.id.clone());
                    }
                    return;
                }
                self.error = Some(format!("Failed to load models: {e}"));
            }
        }
    }

    pub async fn test_connection(&mut self) {
        self.error = None;
        match self.provider.models().await {
            Ok(models) => {
                debug!(count = models.len(), "connection test succeeded");
                let _ = self.model_repo.upsert(self.provider_id, &models).await;
                self.models = models;
            }
            Err(e) => {
                self.error = Some(format!("Connection failed: {e}"));
            }
        }
    }

    pub fn select_model(&mut self, model_id: String) {
        if self.models.iter().any(|m| m.id == model_id) {
            self.selected_model = Some(model_id);
            debug!(model = %self.selected_model.as_ref().unwrap(), "model selected");
        }
    }

    pub async fn handle_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::User(user_event) => self.handle_user_event(user_event).await,
            AppEvent::Provider(provider_event) => self.handle_provider_event(provider_event).await,
        }
    }

    async fn handle_user_event(&mut self, event: UserEvent) {
        // Handle modal input first
        if !matches!(self.modal, Modal::None) {
            match event {
                UserEvent::InputChar(c) => match &mut self.modal {
                    Modal::Rename { buffer } => buffer.push(c),
                    Modal::CommandPalette { query, .. } => query.push(c),
                    Modal::PromptPicker { query, .. } => {
                        query.push(c);
                        self.refresh_prompt_picker_filter();
                    }
                    Modal::PromptEditor {
                        name,
                        description,
                        content,
                        system_prompt,
                        tags,
                        variables,
                        field,
                        ..
                    } => match field {
                        0 => name.push(c),
                        1 => description.push(c),
                        2 => content.push(c),
                        3 => system_prompt.push(c),
                        4 => tags.push(c),
                        5 => variables.push(c),
                        _ => {}
                    },
                    Modal::VariableInput {
                        values, current, ..
                    } => {
                        if let Some(val) = values.get_mut(*current) {
                            val.push(c);
                        }
                    }
                    _ => {}
                },
                UserEvent::Backspace => match &mut self.modal {
                    Modal::Rename { buffer } => {
                        buffer.pop();
                    }
                    Modal::CommandPalette { query, .. } => {
                        query.pop();
                    }
                    Modal::PromptPicker { query, .. } => {
                        query.pop();
                        self.refresh_prompt_picker_filter();
                    }
                    Modal::PromptEditor {
                        name,
                        description,
                        content,
                        system_prompt,
                        tags,
                        variables,
                        field,
                        ..
                    } => match field {
                        0 => {
                            name.pop();
                        }
                        1 => {
                            description.pop();
                        }
                        2 => {
                            content.pop();
                        }
                        3 => {
                            system_prompt.pop();
                        }
                        4 => {
                            tags.pop();
                        }
                        5 => {
                            variables.pop();
                        }
                        _ => {}
                    },
                    Modal::VariableInput {
                        values, current, ..
                    } => {
                        if let Some(val) = values.get_mut(*current) {
                            val.pop();
                        }
                    }
                    _ => {}
                },
                UserEvent::SendMessage => {
                    self.confirm_modal().await;
                }
                UserEvent::Quit => {
                    self.modal = Modal::None;
                    if matches!(self.active_screen, ActiveScreen::Search) {
                        self.active_screen = ActiveScreen::Chat;
                    }
                }
                UserEvent::NavigateUp | UserEvent::PromptFieldPrev => match &mut self.modal {
                    Modal::PromptPicker {
                        selected, filtered, ..
                    } => {
                        *selected = selected.saturating_sub(1);
                        let _ = filtered;
                    }
                    Modal::PromptEditor { field, .. } => {
                        *field = field.saturating_sub(1);
                    }
                    Modal::VariableInput { current, .. } => {
                        *current = current.saturating_sub(1);
                    }
                    _ => {}
                },
                UserEvent::NavigateDown | UserEvent::PromptFieldNext => match &mut self.modal {
                    Modal::PromptPicker {
                        selected, filtered, ..
                    } => {
                        if !filtered.is_empty() {
                            let max = filtered.len().saturating_sub(1);
                            *selected = (*selected + 1).min(max);
                        }
                    }
                    Modal::PromptEditor { field, .. } => {
                        *field = (*field + 1).min(5);
                    }
                    Modal::VariableInput {
                        current, variables, ..
                    } => {
                        let max = variables.len().saturating_sub(1);
                        *current = (*current + 1).min(max);
                    }
                    _ => {}
                },
                UserEvent::PromptNew => {
                    if matches!(self.modal, Modal::PromptPicker { .. }) {
                        self.modal = Modal::PromptEditor {
                            editing_id: None,
                            name: String::new(),
                            description: String::new(),
                            content: String::new(),
                            system_prompt: String::new(),
                            tags: String::new(),
                            variables: String::new(),
                            field: 0,
                        };
                    }
                }
                UserEvent::PromptEditSelected(idx) => {
                    if let Modal::PromptPicker { filtered, .. } = &self.modal
                        && let Some(&real_idx) = filtered.get(idx)
                        && let Some(prompt) = self.prompts.get(real_idx)
                    {
                        self.modal = Modal::PromptEditor {
                            editing_id: Some(prompt.id),
                            name: prompt.name.clone(),
                            description: prompt.description.clone().unwrap_or_default(),
                            content: prompt.content.clone(),
                            system_prompt: prompt.system_prompt.clone().unwrap_or_default(),
                            tags: prompt.tags.join(", "),
                            variables: prompt.variables.join(", "),
                            field: 0,
                        };
                    }
                }
                UserEvent::PromptDeleteConfirm => {
                    if let Modal::PromptPicker { filtered, .. } = &self.modal
                        && let Some(&real_idx) = filtered.first()
                        && let Some(prompt) = self.prompts.get(real_idx)
                    {
                        self.modal = Modal::PromptDeleteConfirm {
                            prompt_id: prompt.id,
                            name: prompt.name.clone(),
                        };
                    }
                }
                _ => {}
            }
            return;
        }

        match event {
            UserEvent::Quit => {
                self.should_quit = true;
            }
            UserEvent::InputChar(c) => {
                if c == '/'
                    && self.input.is_empty()
                    && !self.sidebar_focus
                    && matches!(self.active_screen, ActiveScreen::Chat)
                {
                    self.active_screen = ActiveScreen::Search;
                    self.search_query.clear();
                    self.search_results.clear();
                    self.search_selection = 0;
                } else if matches!(self.active_screen, ActiveScreen::Search) {
                    self.search_query.push(c);
                    self.run_search().await;
                } else if self.sidebar_focus {
                    match c {
                        'j' => {
                            if !self.conversations.is_empty() {
                                let max = self.conversations.len().saturating_sub(1);
                                self.sidebar_selection = (self.sidebar_selection + 1).min(max);
                            }
                        }
                        'k' => {
                            self.sidebar_selection = self.sidebar_selection.saturating_sub(1);
                        }
                        'r' => {
                            self.start_rename();
                        }
                        'd' => {
                            self.start_delete();
                        }
                        'q' => {
                            self.should_quit = true;
                        }
                        'e' => {
                            self.start_prompt_edit();
                        }
                        'n' => {
                            self.active_screen = ActiveScreen::Prompts;
                        }
                        _ => {}
                    }
                } else if matches!(self.active_screen, ActiveScreen::Prompts) {
                    match c {
                        'j' => {
                            if !self.prompts.is_empty() {
                                let max = self.prompts.len().saturating_sub(1);
                                self.prompt_selection = (self.prompt_selection + 1).min(max);
                            }
                        }
                        'k' => {
                            self.prompt_selection = self.prompt_selection.saturating_sub(1);
                        }
                        'n' => {
                            self.modal = Modal::PromptEditor {
                                editing_id: None,
                                name: String::new(),
                                description: String::new(),
                                content: String::new(),
                                system_prompt: String::new(),
                                tags: String::new(),
                                variables: String::new(),
                                field: 0,
                            };
                        }
                        'e' => {
                            self.start_prompt_edit();
                        }
                        'd' => {
                            if let Some(prompt) = self.prompts.get(self.prompt_selection) {
                                self.modal = Modal::PromptDeleteConfirm {
                                    prompt_id: prompt.id,
                                    name: prompt.name.clone(),
                                };
                            }
                        }
                        'q' => {
                            self.active_screen = ActiveScreen::Chat;
                        }
                        _ => {}
                    }
                } else {
                    self.input.push(c);
                }
            }
            UserEvent::Backspace => {
                if matches!(self.active_screen, ActiveScreen::Search) {
                    self.search_query.pop();
                    self.run_search().await;
                } else if !self.sidebar_focus
                    && !matches!(self.active_screen, ActiveScreen::Prompts)
                {
                    self.input.pop();
                }
            }
            UserEvent::SendMessage => {
                if matches!(self.active_screen, ActiveScreen::Search) {
                    self.open_search_result().await;
                } else if self.sidebar_focus {
                    self.open_selected_conversation().await;
                } else if matches!(self.active_screen, ActiveScreen::Prompts) {
                    self.start_prompt_edit();
                } else {
                    self.send_message().await;
                }
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
            UserEvent::TestConnection => {
                self.test_connection().await;
            }
            UserEvent::SelectModel(idx) => {
                if idx == usize::MAX {
                    // Cycle to next model
                    if !self.models.is_empty() {
                        let current = self.selected_model.as_deref().unwrap_or("");
                        let pos = self
                            .models
                            .iter()
                            .position(|m| m.id == current)
                            .unwrap_or(0);
                        let next = (pos + 1) % self.models.len();
                        let model_id = self.models[next].id.clone();
                        self.select_model(model_id);
                    }
                } else if let Some(model) = self.models.get(idx) {
                    let model_id = model.id.clone();
                    self.select_model(model_id);
                }
            }
            UserEvent::OpenCommandPalette => {
                debug!("command palette requested (not yet implemented)");
            }
            UserEvent::NavigateUp => {
                if matches!(self.active_screen, ActiveScreen::Search) {
                    self.search_selection = self.search_selection.saturating_sub(1);
                } else if self.sidebar_focus && !self.conversations.is_empty() {
                    self.sidebar_selection = self.sidebar_selection.saturating_sub(1);
                }
            }
            UserEvent::NavigateDown => {
                if matches!(self.active_screen, ActiveScreen::Search) {
                    let max = self.search_results.len().saturating_sub(1);
                    self.search_selection = (self.search_selection + 1).min(max);
                } else if self.sidebar_focus && !self.conversations.is_empty() {
                    let max = self.conversations.len().saturating_sub(1);
                    self.sidebar_selection = (self.sidebar_selection + 1).min(max);
                }
            }
            UserEvent::OpenSelected => {
                if self.sidebar_focus {
                    self.open_selected_conversation().await;
                }
            }
            UserEvent::ToggleFocus => {
                self.sidebar_focus = !self.sidebar_focus;
                if self.sidebar_focus
                    && !self.conversations.is_empty()
                    && let Some(active_id) = self.active_conversation
                    && let Some(pos) = self.conversations.iter().position(|c| c.id == active_id)
                {
                    self.sidebar_selection = pos;
                }
            }
            UserEvent::StartRename => {
                if self.sidebar_focus {
                    self.start_rename();
                }
            }
            UserEvent::StartDelete => {
                if self.sidebar_focus {
                    self.start_delete();
                }
            }
            UserEvent::ConfirmAction => {
                self.confirm_modal().await;
            }
            UserEvent::CancelModal => {
                self.modal = Modal::None;
            }
            UserEvent::OpenHelp => {
                self.modal = Modal::Help;
            }
            UserEvent::OpenPromptPicker => {
                self.open_prompt_picker();
            }
            UserEvent::OpenPromptList => {
                self.active_screen = ActiveScreen::Prompts;
            }
            UserEvent::OpenSearch => {
                self.active_screen = ActiveScreen::Search;
                self.search_query.clear();
                self.search_results.clear();
                self.search_selection = 0;
            }
            UserEvent::SearchNavigateUp => {
                self.search_selection = self.search_selection.saturating_sub(1);
            }
            UserEvent::SearchNavigateDown => {
                let max = self.search_results.len().saturating_sub(1);
                self.search_selection = (self.search_selection + 1).min(max);
            }
            UserEvent::SearchOpenResult => {
                self.open_search_result().await;
            }
            UserEvent::PromptNew => {
                self.modal = Modal::PromptEditor {
                    editing_id: None,
                    name: String::new(),
                    description: String::new(),
                    content: String::new(),
                    system_prompt: String::new(),
                    tags: String::new(),
                    variables: String::new(),
                    field: 0,
                };
            }
            UserEvent::PromptEditSelected(idx) => {
                if let Some(prompt) = self.prompts.get(idx) {
                    self.modal = Modal::PromptEditor {
                        editing_id: Some(prompt.id),
                        name: prompt.name.clone(),
                        description: prompt.description.clone().unwrap_or_default(),
                        content: prompt.content.clone(),
                        system_prompt: prompt.system_prompt.clone().unwrap_or_default(),
                        tags: prompt.tags.join(", "),
                        variables: prompt.variables.join(", "),
                        field: 0,
                    };
                }
            }
            UserEvent::PromptDeleteConfirm => {
                if let Some(prompt) = self.prompts.get(self.prompt_selection) {
                    self.modal = Modal::PromptDeleteConfirm {
                        prompt_id: prompt.id,
                        name: prompt.name.clone(),
                    };
                }
            }
            UserEvent::PromptFieldNext | UserEvent::PromptFieldPrev => {}
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
                    started_at: std::time::Instant::now(),
                    usage: None,
                });
            }
            ProviderEvent::Delta { message_id, text } => {
                if let Some(ref mut state) = self.streaming
                    && state.message_id == message_id
                {
                    state.buffer.push_str(&text);
                }
            }
            ProviderEvent::ReasoningDelta { .. } => {}
            ProviderEvent::Usage { message_id, usage } => {
                if let Some(ref mut state) = self.streaming
                    && state.message_id == message_id
                {
                    state.usage = Some(usage);
                }
            }
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

    fn build_generation_params(&self) -> GenerationParameters {
        GenerationParameters {
            temperature: self.generation.temperature,
            top_p: self.generation.top_p,
            max_tokens: self.generation.max_tokens,
            stop: Vec::new(),
        }
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
            generation: self.build_generation_params(),
        };

        let cancel = CancellationToken::new();
        let assistant_message_id = Uuid::new_v4();

        self.streaming = Some(StreamingState {
            message_id: assistant_message_id,
            buffer: String::new(),
            cancel: cancel.clone(),
            started_at: std::time::Instant::now(),
            usage: None,
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

        if let Some(last) = self.messages.last()
            && last.role == Role::Assistant
        {
            self.messages.pop();
        }

        self.start_generation(conversation_id).await;
    }

    async fn touch_conversation(&mut self) {
        let Some(conversation_id) = self.active_conversation else {
            return;
        };
        if let Some(conv) = self
            .conversations
            .iter_mut()
            .find(|c| c.id == conversation_id)
        {
            conv.updated_at = Utc::now();
            if self.messages.len() == 2
                && conv.title == "New Conversation"
                && let Some(user_msg) = self.messages.iter().find(|m| m.role == Role::User)
            {
                conv.title = user_msg.content.chars().take(50).collect();
            }
        }
    }

    pub fn streaming_content(&self) -> Option<&str> {
        self.streaming.as_ref().map(|s| s.buffer.as_str())
    }

    pub fn streaming_stats(&self) -> Option<(usize, f64)> {
        self.streaming.as_ref().map(|s| {
            let elapsed = s.started_at.elapsed().as_secs_f64();
            let tokens = s
                .usage
                .as_ref()
                .and_then(|u| u.completion_tokens)
                .unwrap_or(0) as usize;
            (tokens, elapsed)
        })
    }

    async fn open_selected_conversation(&mut self) {
        if let Some(conv) = self.conversations.get(self.sidebar_selection) {
            let conv_id = conv.id;
            if self.active_conversation == Some(conv_id) {
                self.sidebar_focus = false;
                return;
            }
            self.active_conversation = Some(conv_id);
            match self.message_repo.list_for_conversation(conv_id).await {
                Ok(messages) => {
                    self.messages = messages;
                    self.sidebar_focus = false;
                    self.error = None;
                    debug!(id = %conv_id, "opened conversation");
                }
                Err(e) => {
                    self.error = Some(format!("Failed to load messages: {e}"));
                }
            }
        }
    }

    fn start_rename(&mut self) {
        if let Some(conv) = self.conversations.get(self.sidebar_selection) {
            self.modal = Modal::Rename {
                buffer: conv.title.clone(),
            };
        }
    }

    fn start_delete(&mut self) {
        if let Some(conv) = self.conversations.get(self.sidebar_selection) {
            self.modal = Modal::DeleteConfirm {
                conversation_id: conv.id,
                title: conv.title.clone(),
            };
        }
    }

    fn start_prompt_edit(&mut self) {
        let idx = if matches!(self.active_screen, ActiveScreen::Prompts) {
            self.prompt_selection
        } else {
            self.sidebar_selection
        };
        if let Some(prompt) = self.prompts.get(idx) {
            self.modal = Modal::PromptEditor {
                editing_id: Some(prompt.id),
                name: prompt.name.clone(),
                description: prompt.description.clone().unwrap_or_default(),
                content: prompt.content.clone(),
                system_prompt: prompt.system_prompt.clone().unwrap_or_default(),
                tags: prompt.tags.join(", "),
                variables: prompt.variables.join(", "),
                field: 0,
            };
        }
    }

    async fn confirm_modal(&mut self) {
        match &self.modal {
            Modal::Rename { buffer } => {
                let new_title = buffer.clone();
                if let Some(conv) = self.conversations.get_mut(self.sidebar_selection) {
                    conv.title = new_title;
                    conv.updated_at = Utc::now();
                    let _ = self.conversation_repo.update(conv).await;
                }
                self.modal = Modal::None;
            }
            Modal::DeleteConfirm {
                conversation_id, ..
            } => {
                let id = *conversation_id;
                let _ = self.conversation_repo.delete(id).await;
                self.conversations.retain(|c| c.id != id);
                if self.active_conversation == Some(id) {
                    self.active_conversation = None;
                    self.messages.clear();
                }
                if !self.conversations.is_empty()
                    && self.sidebar_selection >= self.conversations.len()
                {
                    self.sidebar_selection = self.conversations.len() - 1;
                }
                self.modal = Modal::None;
            }
            Modal::PromptPicker {
                filtered, selected, ..
            } => {
                let idx = *selected;
                let filtered = filtered.clone();
                if let Some(&real_idx) = filtered.get(idx)
                    && let Some(prompt) = self.prompts.get(real_idx).cloned()
                {
                    self.modal = Modal::None;
                    self.use_prompt(prompt, false).await;
                    return;
                }
                self.modal = Modal::None;
            }
            Modal::PromptEditor { .. } => {
                self.save_prompt_editor().await;
            }
            Modal::VariableInput { .. } => {
                self.confirm_variable_input().await;
            }
            Modal::PromptDeleteConfirm { prompt_id, .. } => {
                let id = *prompt_id;
                let _ = self.prompt_repo.delete(id).await;
                self.prompts.retain(|p| p.id != id);
                if !self.prompts.is_empty() && self.prompt_selection >= self.prompts.len() {
                    self.prompt_selection = self.prompts.len() - 1;
                }
                self.modal = Modal::None;
            }
            _ => {
                self.modal = Modal::None;
            }
        }
    }

    fn open_prompt_picker(&mut self) {
        let filtered: Vec<usize> = (0..self.prompts.len()).collect();
        self.modal = Modal::PromptPicker {
            query: String::new(),
            selected: 0,
            filtered,
        };
    }

    fn refresh_prompt_picker_filter(&mut self) {
        if let Modal::PromptPicker {
            query,
            selected,
            filtered,
        } = &mut self.modal
        {
            let q = query.to_lowercase();
            *filtered = self
                .prompts
                .iter()
                .enumerate()
                .filter(|(_, p)| {
                    q.is_empty()
                        || p.name.to_lowercase().contains(&q)
                        || p.content.to_lowercase().contains(&q)
                        || p.tags.iter().any(|t| t.to_lowercase().contains(&q))
                })
                .map(|(i, _)| i)
                .collect();
            if *selected >= filtered.len() {
                *selected = filtered.len().saturating_sub(1);
            }
        }
    }

    async fn use_prompt(&mut self, prompt: Prompt, send_immediately: bool) {
        let vars = extract_variables(&prompt.content, &prompt.variables);
        if vars.is_empty() {
            let resolved = prompt.content.clone();
            if send_immediately {
                self.input = resolved;
                self.send_message().await;
            } else {
                self.input = resolved;
            }
            self.active_screen = ActiveScreen::Chat;
        } else {
            self.modal = Modal::VariableInput {
                prompt_content: prompt.content.clone(),
                variables: vars,
                values: Vec::new(),
                current: 0,
                send_immediately,
            };
            if let Modal::VariableInput {
                values, variables, ..
            } = &mut self.modal
            {
                *values = vec![String::new(); variables.len()];
            }
        }
    }

    async fn confirm_variable_input(&mut self) {
        let (content, _values, send_immediately) = match &self.modal {
            Modal::VariableInput {
                prompt_content,
                variables,
                values,
                send_immediately,
                ..
            } => {
                let mut resolved = prompt_content.clone();
                for (var, val) in variables.iter().zip(values.iter()) {
                    resolved = resolved.replace(&format!("{{{{{var}}}}}"), val);
                }
                (resolved, values.clone(), *send_immediately)
            }
            _ => return,
        };
        self.modal = Modal::None;
        self.input = content;
        self.active_screen = ActiveScreen::Chat;
        if send_immediately {
            self.send_message().await;
        }
    }

    async fn save_prompt_editor(&mut self) {
        let (editing_id, name, description, content, system_prompt, tags_str, variables_str) =
            match &self.modal {
                Modal::PromptEditor {
                    editing_id,
                    name,
                    description,
                    content,
                    system_prompt,
                    tags,
                    variables,
                    ..
                } => (
                    *editing_id,
                    name.clone(),
                    description.clone(),
                    content.clone(),
                    system_prompt.clone(),
                    tags.clone(),
                    variables.clone(),
                ),
                _ => return,
            };

        if name.trim().is_empty() {
            self.error = Some("Prompt name cannot be empty".into());
            return;
        }

        let tags: Vec<String> = tags_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let mut variables: Vec<String> = variables_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let detected = extract_variables(&content, &[]);
        for v in detected {
            if !variables.contains(&v) {
                variables.push(v);
            }
        }

        let now = Utc::now();
        let prompt = if let Some(id) = editing_id {
            Prompt {
                id,
                name,
                description: if description.is_empty() {
                    None
                } else {
                    Some(description)
                },
                content,
                system_prompt: if system_prompt.is_empty() {
                    None
                } else {
                    Some(system_prompt)
                },
                tags,
                variables,
                created_at: self
                    .prompts
                    .iter()
                    .find(|p| p.id == id)
                    .map(|p| p.created_at)
                    .unwrap_or(now),
                updated_at: now,
            }
        } else {
            Prompt {
                id: Uuid::new_v4(),
                name,
                description: if description.is_empty() {
                    None
                } else {
                    Some(description)
                },
                content,
                system_prompt: if system_prompt.is_empty() {
                    None
                } else {
                    Some(system_prompt)
                },
                tags,
                variables,
                created_at: now,
                updated_at: now,
            }
        };

        if editing_id.is_some() {
            if let Err(e) = self.prompt_repo.update(&prompt).await {
                self.error = Some(format!("Failed to update prompt: {e}"));
                return;
            }
            if let Some(existing) = self.prompts.iter_mut().find(|p| p.id == prompt.id) {
                *existing = prompt.clone();
            }
        } else {
            if let Err(e) = self.prompt_repo.create(&prompt).await {
                self.error = Some(format!("Failed to create prompt: {e}"));
                return;
            }
            self.prompts.insert(0, prompt);
        }

        self.modal = Modal::None;
        self.error = None;
    }

    async fn open_search_result(&mut self) {
        if let Some(result) = self.search_results.get(self.search_selection) {
            match result {
                SearchResultEntry::Message {
                    conversation_id, ..
                } => {
                    let conv_id = *conversation_id;
                    self.active_conversation = Some(conv_id);
                    match self.message_repo.list_for_conversation(conv_id).await {
                        Ok(messages) => self.messages = messages,
                        Err(e) => self.error = Some(format!("Failed to load messages: {e}")),
                    }
                    self.active_screen = ActiveScreen::Chat;
                    self.sidebar_focus = false;
                }
                SearchResultEntry::Prompt { prompt } => {
                    let prompt = prompt.clone();
                    self.active_screen = ActiveScreen::Chat;
                    self.use_prompt(prompt, false).await;
                }
            }
        }
    }

    pub async fn run_search(&mut self) {
        let query = self.search_query.clone();
        if query.trim().is_empty() {
            self.search_results.clear();
            return;
        }

        let mut results = Vec::new();

        match self.message_repo.search(&query, 20).await {
            Ok(msg_results) => {
                for r in msg_results {
                    let conv_id = r.message.conversation_id;
                    results.push(SearchResultEntry::Message {
                        message: r.message,
                        conversation_id: conv_id,
                        conversation_title: r.conversation_title,
                    });
                }
            }
            Err(e) => {
                warn!(%e, "message search failed");
            }
        }

        match self.prompt_repo.search(&query, 10).await {
            Ok(prompt_results) => {
                for r in prompt_results {
                    results.push(SearchResultEntry::Prompt { prompt: r.prompt });
                }
            }
            Err(e) => {
                warn!(%e, "prompt search failed");
            }
        }

        self.search_results = results;
        self.search_selection = 0;
    }
}

fn extract_variables(content: &str, declared: &[String]) -> Vec<String> {
    let mut vars: Vec<String> = declared.to_vec();
    let mut remaining = content;
    while let Some(start) = remaining.find("{{") {
        if let Some(end) = remaining[start..].find("}}") {
            let var_name = remaining[start + 2..start + end].trim().to_string();
            if !var_name.is_empty() && !vars.contains(&var_name) {
                vars.push(var_name);
            }
            remaining = &remaining[start + end + 2..];
        } else {
            break;
        }
    }
    vars
}
