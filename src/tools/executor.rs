use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::Mutex;
use tracing::{debug, warn};

use crate::domain::{ToolCall, ToolDefinition, ToolResult};

#[async_trait]
pub trait ToolExecutor: Send + Sync {
    async fn execute(&self, call: &ToolCall) -> ToolResult;
    fn definition(&self) -> ToolDefinition;
}

pub struct ToolRegistry {
    executors: Arc<Mutex<HashMap<String, Box<dyn ToolExecutor>>>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            executors: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn register(&self, executor: Box<dyn ToolExecutor>) {
        let def = executor.definition();
        let mut executors = self.executors.lock().await;
        executors.insert(def.name, executor);
    }

    pub async fn execute(&self, call: &ToolCall) -> ToolResult {
        let executors = self.executors.lock().await;
        if let Some(executor) = executors.get(&call.name) {
            debug!(tool = %call.name, "executing tool");
            executor.execute(call).await
        } else {
            warn!(tool = %call.name, "tool not found");
            ToolResult {
                call_id: call.id.clone(),
                output: format!("Error: Tool '{}' not found", call.name),
                is_error: true,
            }
        }
    }

    pub async fn definitions(&self) -> Vec<ToolDefinition> {
        let executors = self.executors.lock().await;
        executors.values().map(|e| e.definition()).collect()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}
