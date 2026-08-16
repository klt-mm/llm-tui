use std::time::Duration;

use futures_util::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::domain::{
    Capabilities, GenerationUsage, Model, Provider, Role, ToolCall, ToolDefinition,
};
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
struct ModelsResponse {
    data: Vec<ModelItem>,
}

#[derive(Deserialize)]
struct ModelItem {
    id: String,
}

#[derive(Serialize)]
struct ChatBody {
    model: String,
    messages: Vec<ChatMessage>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<ToolDefinition>>,
}

#[derive(Serialize)]
struct ChatMessage {
    role: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_call_id: Option<String>,
}

impl ChatBody {
    fn from_request(req: &ChatRequest) -> Self {
        Self {
            model: req.model.clone(),
            messages: req
                .messages
                .iter()
                .map(|m| {
                    // Build content based on whether images are present
                    let content = if let Some(ref images) = m.images {
                        if !images.is_empty() {
                            // Multimodal content: array of text and image parts
                            let mut parts = Vec::new();

                            // Add text part if content is not empty
                            if !m.content.is_empty() {
                                parts.push(serde_json::json!({
                                    "type": "text",
                                    "text": m.content
                                }));
                            }

                            // Add image parts
                            for image in images {
                                let mut image_part = serde_json::json!({
                                    "type": "image_url",
                                    "image_url": {
                                        "url": image.url
                                    }
                                });

                                if let Some(ref detail) = image.detail {
                                    image_part["image_url"]["detail"] = serde_json::json!(detail);
                                }

                                parts.push(image_part);
                            }

                            Some(serde_json::Value::Array(parts))
                        } else if !m.content.is_empty() {
                            Some(serde_json::Value::String(m.content.clone()))
                        } else {
                            None
                        }
                    } else if !m.content.is_empty() {
                        Some(serde_json::Value::String(m.content.clone()))
                    } else {
                        None
                    };

                    ChatMessage {
                        role: OpenAiCompatibleProvider::role(&m.role),
                        content,
                        tool_calls: m.tool_calls.clone(),
                        tool_call_id: m.tool_call_id.clone(),
                    }
                })
                .collect(),
            stream: true,
            temperature: req.generation.temperature,
            top_p: req.generation.top_p,
            max_tokens: req.generation.max_tokens,
            tools: req.tools.clone(),
        }
    }
}

#[async_trait::async_trait]
impl LlmProvider for OpenAiCompatibleProvider {
    async fn models(&self) -> Result<Vec<Model>, LlmError> {
        let req = self.client.get(self.endpoint("/models"));
        let response = self
            .auth(req)
            .send()
            .await
            .map_err(|e| LlmError::Network(e.to_string()))?
            .error_for_status()
            .map_err(|e| LlmError::Provider(e.to_string()))?;

        let body: ModelsResponse = response
            .json()
            .await
            .map_err(|e| LlmError::Provider(e.to_string()))?;

        Ok(body
            .data
            .into_iter()
            .map(|m| Model {
                id: m.id,
                display_name: None,
                context_length: None,
                metadata: serde_json::json!({}),
            })
            .collect())
    }

    async fn capabilities(&self) -> Result<Capabilities, LlmError> {
        // For OpenAI-compatible providers, we assume standard capabilities
        // In a real implementation, we'd probe endpoints or check model metadata
        Ok(Capabilities {
            streaming: true,
            tool_calling: true, // Most OpenAI-compatible providers support this
            tools: true,
            parallel_tool_calls: true,
            vision: true, // GPT-4V and compatible models
            image_input: true,
            image_formats: vec!["png".into(), "jpeg".into(), "webp".into()],
            structured_output: true, // JSON mode support
            json_mode: true,
            reasoning: false,              // Depends on model (o1, etc.)
            embeddings: false,             // Separate endpoint
            responses_api: false,          // Newer API
            max_output_tokens: Some(4096), // Typical default
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
                StreamEvent::ToolCall { .. } => {}
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
        let req = self
            .client
            .post(self.endpoint("/chat/completions"))
            .json(&body);
        let response = self
            .auth(req)
            .send()
            .await
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

                                        // Handle content deltas
                                        if let Some(text) = value["choices"][0]["delta"]["content"].as_str()
                                            && tx.send(Ok(StreamEvent::Delta(text.to_string()))).await.is_err() {
                                                return;
                                            }

                                        // Handle tool calls
                                        if let Some(tool_calls) = value["choices"][0]["delta"]["tool_calls"].as_array() {
                                            for tool_call in tool_calls {
                                                if let (Some(id), Some(name), Some(args)) = (
                                                    tool_call["id"].as_str(),
                                                    tool_call["function"]["name"].as_str(),
                                                    tool_call["function"]["arguments"].as_str(),
                                                ) && tx.send(Ok(StreamEvent::ToolCall {
                                                    id: id.to_string(),
                                                    name: name.to_string(),
                                                    arguments: args.to_string(),
                                                })).await.is_err() {
                                                    return;
                                                }
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
