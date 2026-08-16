use std::time::Duration;

use futures_util::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::domain::{Capabilities, GenerationUsage, Model, Provider, Role};
use crate::llm::provider::{ChatRequest, ChatResponse, LlmError, LlmProvider, StreamEvent};

#[derive(Clone)]
pub struct OpenAiCompatibleProvider {
    client: Client,
    config: Provider,
}

impl OpenAiCompatibleProvider {
    pub fn new(config: Provider) -> Result<Self, LlmError> {
        let client = Client::builder()
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(300))
            .build()
            .map_err(|e| LlmError::Network(e.to_string()))?;

        Ok(Self { client, config })
    }

    fn endpoint(&self, path: &str) -> String {
        format!(
            "{}/{}",
            self.config.base_url.trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    }

    fn auth(&self, request: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        match &self.config.api_key_ref {
            Some(key) if !key.is_empty() => request.bearer_auth(key),
            _ => request,
        }
    }

    fn role(role: &Role) -> &'static str {
        match role {
            Role::System => "system",
            Role::User => "user",
            Role::Assistant => "assistant",
            Role::Tool => "tool",
        }
    }
}

#[derive(Deserialize)]
struct ModelsResponse { data: Vec<ModelItem> }

#[derive(Deserialize)]
struct ModelItem { id: String }

#[derive(Serialize)]
struct ChatBody<'a> {
    model: &'a str,
    messages: Vec<ChatMessage<'a>>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
}

#[derive(Serialize)]
struct ChatMessage<'a> {
    role: &'static str,
    content: &'a str,
}

impl<'a> ChatBody<'a> {
    fn from_request(req: &'a ChatRequest) -> Self {
        Self {
            model: &req.model,
            messages: req.messages.iter().map(|m| ChatMessage {
                role: OpenAiCompatibleProvider::role(&m.role),
                content: &m.content,
            }).collect(),
            stream: true,
            temperature: req.generation.temperature,
            top_p: req.generation.top_p,
            max_tokens: req.generation.max_tokens,
        }
    }
}

#[async_trait::async_trait]
impl LlmProvider for OpenAiCompatibleProvider {
    async fn models(&self) -> Result<Vec<Model>, LlmError> {
        let req = self.client.get(self.endpoint("/models"));
        let response = self.auth(req).send().await
            .map_err(|e| LlmError::Network(e.to_string()))?
            .error_for_status()
            .map_err(|e| LlmError::Provider(e.to_string()))?;

        let body: ModelsResponse = response.json().await
            .map_err(|e| LlmError::Provider(e.to_string()))?;

        Ok(body.data.into_iter().map(|m| Model {
            id: m.id,
            display_name: None,
            context_length: None,
            metadata: serde_json::json!({}),
        }).collect())
    }

    async fn capabilities(&self) -> Result<Capabilities, LlmError> {
        Ok(Capabilities {
            streaming: true,
            ..Default::default()
        })
    }

    async fn chat(
        &self,
        request: ChatRequest,
        cancel: CancellationToken,
    ) -> Result<ChatResponse, LlmError> {
        let mut rx = self.stream_chat(request, cancel).await?;
        let mut content = String::new();
        let mut usage = GenerationUsage::default();

        while let Some(event) = rx.recv().await {
            match event? {
                StreamEvent::Delta(s) => content.push_str(&s),
                StreamEvent::Usage(u) => usage = u,
                StreamEvent::ReasoningDelta(_) => {}
                StreamEvent::Completed => break,
            }
        }

        Ok(ChatResponse { content, usage })
    }

    async fn stream_chat(
        &self,
        request: ChatRequest,
        cancel: CancellationToken,
    ) -> Result<mpsc::Receiver<Result<StreamEvent, LlmError>>, LlmError> {
        let body = ChatBody::from_request(&request);
        let req = self.client.post(self.endpoint("/chat/completions")).json(&body);
        let response = self.auth(req).send().await
            .map_err(|e| LlmError::Network(e.to_string()))?
            .error_for_status()
            .map_err(|e| LlmError::Provider(e.to_string()))?;

        let mut stream = response.bytes_stream();
        let (tx, rx) = mpsc::channel(64);

        tokio::spawn(async move {
            let mut buffer = String::new();

            loop {
                tokio::select! {
                    _ = cancel.cancelled() => break,
                    item = stream.next() => {
                        match item {
                            Some(Ok(bytes)) => {
                                buffer.push_str(&String::from_utf8_lossy(&bytes));

                                while let Some(pos) = buffer.find("\n\n") {
                                    let frame = buffer[..pos].to_string();
                                    buffer.drain(..pos + 2);

                                    for line in frame.lines() {
                                        let Some(data) = line.strip_prefix("data: ") else { continue };
                                        if data == "[DONE]" {
                                            let _ = tx.send(Ok(StreamEvent::Completed)).await;
                                            return;
                                        }

                                        let parsed: Result<serde_json::Value, _> = serde_json::from_str(data);
                                        let Ok(value) = parsed else { continue };

                                        if let Some(text) = value["choices"][0]["delta"]["content"].as_str() {
                                            if tx.send(Ok(StreamEvent::Delta(text.to_string()))).await.is_err() {
                                                return;
                                            }
                                        }
                                    }
                                }
                            }
                            Some(Err(e)) => {
                                let _ = tx.send(Err(LlmError::Stream(e.to_string()))).await;
                                return;
                            }
                            None => {
                                let _ = tx.send(Ok(StreamEvent::Completed)).await;
                                return;
                            }
                        }
                    }
                }
            }
        });

        Ok(rx)
    }
}
