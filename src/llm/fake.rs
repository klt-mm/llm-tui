use async_trait::async_trait;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::domain::{Capabilities, GenerationUsage, Model};
use crate::llm::provider::{ChatRequest, ChatResponse, LlmError, LlmProvider, StreamEvent};

pub struct FakeProvider;

impl Default for FakeProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl FakeProvider {
    pub fn new() -> Self {
        Self
    }
}

const FAKE_TOKENS: &[&str] = &[
    "The ",
    "quick ",
    "brown ",
    "fox ",
    "jumps ",
    "over ",
    "the ",
    "lazy ",
    "dog. ",
    "This ",
    "is ",
    "a ",
    "fake ",
    "response ",
    "from ",
    "the ",
    "test ",
    "provider. ",
    "It ",
    "streams ",
    "tokens ",
    "one ",
    "by ",
    "one ",
    "to ",
    "simulate ",
    "real ",
    "LLM ",
    "behavior. ",
    "You ",
    "can ",
    "use ",
    "this ",
    "for ",
    "testing ",
    "the ",
    "TUI ",
    "without ",
    "a ",
    "real ",
    "model. ",
    "Have ",
    "fun! ",
    "🦊",
];

#[async_trait]
impl LlmProvider for FakeProvider {
    async fn models(&self) -> Result<Vec<Model>, LlmError> {
        Ok(vec![
            Model {
                id: "fake-fast".into(),
                display_name: Some("Fake Fast".into()),
                context_length: Some(4096),
                metadata: serde_json::json!({}),
            },
            Model {
                id: "fake-slow".into(),
                display_name: Some("Fake Slow".into()),
                context_length: Some(8192),
                metadata: serde_json::json!({}),
            },
            Model {
                id: "fake-reasoning".into(),
                display_name: Some("Fake Reasoning".into()),
                context_length: Some(16384),
                metadata: serde_json::json!({}),
            },
        ])
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
        _request: ChatRequest,
        cancel: CancellationToken,
    ) -> Result<mpsc::Receiver<Result<StreamEvent, LlmError>>, LlmError> {
        let (tx, rx) = mpsc::channel(64);

        tokio::spawn(async move {
            let _ = tx.send(Ok(StreamEvent::Delta(String::new()))).await;

            for token in FAKE_TOKENS {
                tokio::select! {
                    _ = cancel.cancelled() => {
                        let _ = tx.send(Ok(StreamEvent::Completed)).await;
                        return;
                    }
                    _ = tokio::time::sleep(std::time::Duration::from_millis(50)) => {}
                }

                if tx
                    .send(Ok(StreamEvent::Delta(token.to_string())))
                    .await
                    .is_err()
                {
                    return;
                }
            }

            let _ = tx
                .send(Ok(StreamEvent::Usage(GenerationUsage {
                    prompt_tokens: Some(42),
                    completion_tokens: Some(FAKE_TOKENS.len() as u64),
                    total_tokens: Some(42 + FAKE_TOKENS.len() as u64),
                    prompt_ms: Some(10.0),
                    generation_ms: Some(FAKE_TOKENS.len() as f64 * 50.0),
                })))
                .await;

            let _ = tx.send(Ok(StreamEvent::Completed)).await;
        });

        Ok(rx)
    }
}
