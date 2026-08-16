use async_trait::async_trait;
use std::process::Command;
use tracing::debug;

use crate::domain::{ToolCall, ToolDefinition, ToolResult};
use crate::tools::executor::ToolExecutor;

pub struct ShellTool;

#[async_trait]
impl ToolExecutor for ShellTool {
    async fn execute(&self, call: &ToolCall) -> ToolResult {
        debug!(tool = "shell", args = ?call.arguments, "executing shell command");

        let args = &call.arguments;

        let command = match args.get("command").and_then(|v| v.as_str()) {
            Some(c) => c.to_string(),
            None => {
                return ToolResult {
                    call_id: call.id.clone(),
                    output: "Missing 'command' argument".to_string(),
                    is_error: true,
                };
            }
        };

        let output = tokio::task::spawn_blocking(move || {
            Command::new("sh").arg("-c").arg(&command).output()
        })
        .await;

        let output = match output {
            Ok(Ok(o)) => o,
            Ok(Err(e)) => {
                return ToolResult {
                    call_id: call.id.clone(),
                    output: format!("Failed to execute command: {}", e),
                    is_error: true,
                };
            }
            Err(e) => {
                return ToolResult {
                    call_id: call.id.clone(),
                    output: format!("Task failed: {}", e),
                    is_error: true,
                };
            }
        };

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        let result = if output.status.success() {
            if stderr.is_empty() {
                stdout
            } else {
                format!("{}\n{}", stdout, stderr)
            }
        } else {
            format!(
                "Command failed with exit code {:?}\n{}\n{}",
                output.status.code(),
                stdout,
                stderr
            )
        };

        ToolResult {
            call_id: call.id.clone(),
            output: result,
            is_error: !output.status.success(),
        }
    }

    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "shell".to_string(),
            description: "Execute a shell command and return the output".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "command": {
                        "type": "string",
                        "description": "The shell command to execute"
                    }
                },
                "required": ["command"]
            }),
        }
    }
}

pub struct ReadFileTool;

#[async_trait]
impl ToolExecutor for ReadFileTool {
    async fn execute(&self, call: &ToolCall) -> ToolResult {
        debug!(tool = "read_file", args = ?call.arguments, "reading file");

        let args = &call.arguments;

        let path = match args.get("path").and_then(|v| v.as_str()) {
            Some(p) => p,
            None => {
                return ToolResult {
                    call_id: call.id.clone(),
                    output: "Missing 'path' argument".to_string(),
                    is_error: true,
                };
            }
        };

        match tokio::fs::read_to_string(path).await {
            Ok(content) => ToolResult {
                call_id: call.id.clone(),
                output: content,
                is_error: false,
            },
            Err(e) => ToolResult {
                call_id: call.id.clone(),
                output: format!("Failed to read file: {}", e),
                is_error: true,
            },
        }
    }

    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "read_file".to_string(),
            description: "Read the contents of a file".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "The path to the file to read"
                    }
                },
                "required": ["path"]
            }),
        }
    }
}

pub struct WriteFileTool;

#[async_trait]
impl ToolExecutor for WriteFileTool {
    async fn execute(&self, call: &ToolCall) -> ToolResult {
        debug!(tool = "write_file", args = ?call.arguments, "writing file");

        let args = &call.arguments;

        let path = match args.get("path").and_then(|v| v.as_str()) {
            Some(p) => p,
            None => {
                return ToolResult {
                    call_id: call.id.clone(),
                    output: "Missing 'path' argument".to_string(),
                    is_error: true,
                };
            }
        };

        let content = match args.get("content").and_then(|v| v.as_str()) {
            Some(c) => c,
            None => {
                return ToolResult {
                    call_id: call.id.clone(),
                    output: "Missing 'content' argument".to_string(),
                    is_error: true,
                };
            }
        };

        match tokio::fs::write(path, content).await {
            Ok(_) => ToolResult {
                call_id: call.id.clone(),
                output: format!("Successfully wrote to {}", path),
                is_error: false,
            },
            Err(e) => ToolResult {
                call_id: call.id.clone(),
                output: format!("Failed to write file: {}", e),
                is_error: true,
            },
        }
    }

    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "write_file".to_string(),
            description: "Write content to a file".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "The path to the file to write"
                    },
                    "content": {
                        "type": "string",
                        "description": "The content to write to the file"
                    }
                },
                "required": ["path", "content"]
            }),
        }
    }
}

pub struct ListDirectoryTool;

#[async_trait]
impl ToolExecutor for ListDirectoryTool {
    async fn execute(&self, call: &ToolCall) -> ToolResult {
        debug!(tool = "list_directory", args = ?call.arguments, "listing directory");

        let args = &call.arguments;

        let path = match args.get("path").and_then(|v| v.as_str()) {
            Some(p) => p.to_string(),
            None => ".".to_string(),
        };

        match tokio::fs::read_dir(path).await {
            Ok(mut entries) => {
                let mut result = Vec::new();
                while let Ok(Some(entry)) = entries.next_entry().await {
                    let path = entry.path();
                    let name = path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    let is_dir = entry.file_type().await.map(|t| t.is_dir()).unwrap_or(false);
                    result.push(if is_dir { format!("{}/", name) } else { name });
                }
                result.sort();
                ToolResult {
                    call_id: call.id.clone(),
                    output: result.join("\n"),
                    is_error: false,
                }
            }
            Err(e) => ToolResult {
                call_id: call.id.clone(),
                output: format!("Failed to list directory: {}", e),
                is_error: true,
            },
        }
    }

    fn definition(&self) -> ToolDefinition {
        ToolDefinition {
            name: "list_directory".to_string(),
            description: "List the contents of a directory".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "The path to the directory to list (defaults to current directory)"
                    }
                },
                "required": []
            }),
        }
    }
}
