use std::sync::Arc;

use tokio::sync::mpsc;
use uuid::Uuid;

use llm_tui::app::App;
use llm_tui::config::{ContextConfig, GenerationConfig};
use llm_tui::domain::*;
use llm_tui::events::{AppEvent, UserEvent};
use llm_tui::llm::FakeProvider;
use llm_tui::persistence::{
    Database, SqliteConversationRepository, SqliteGenerationRunRepository, SqliteMessageRepository,
    SqliteModelRepository, SqlitePromptRepository, SqliteProviderRepository,
};
use llm_tui::tools::{
    ListDirectoryTool, ReadFileTool, ShellTool, ToolExecutor, ToolRegistry, WriteFileTool,
};

async fn test_db() -> (Database, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.db");
    let url = format!("sqlite://{}?mode=rwc", path.display());
    let db = Database::connect(&url).await.unwrap();
    (db, dir)
}

fn make_app(db: &Database, event_tx: mpsc::Sender<AppEvent>) -> App {
    let provider = Arc::new(FakeProvider::new());
    let conv_repo = Arc::new(SqliteConversationRepository::new(db.pool.clone()));
    let msg_repo = Arc::new(SqliteMessageRepository::new(db.pool.clone()));
    let model_repo = Arc::new(SqliteModelRepository::new(db.pool.clone()));
    let provider_repo = Arc::new(SqliteProviderRepository::new(db.pool.clone()));
    let prompt_repo = Arc::new(SqlitePromptRepository::new(db.pool.clone()));
    let generation_run_repo = Arc::new(SqliteGenerationRunRepository::new(db.pool.clone()));
    App::new(
        provider,
        "fake".into(),
        conv_repo,
        msg_repo,
        model_repo,
        provider_repo,
        prompt_repo,
        generation_run_repo,
        GenerationConfig::default(),
        ContextConfig::default(),
        event_tx,
    )
}

// -----------------------------------------------------------------------
// Tool Registry Tests
// -----------------------------------------------------------------------

#[tokio::test]
async fn tool_registry_register_and_execute() {
    let registry = ToolRegistry::new();

    // Register a simple tool
    registry.register(Box::new(ShellTool)).await;

    // Verify tool is registered
    let defs = registry.definitions().await;
    assert_eq!(defs.len(), 1);
    assert_eq!(defs[0].name, "shell");
}

#[tokio::test]
async fn tool_registry_execute_nonexistent_tool() {
    let registry = ToolRegistry::new();

    let call = ToolCall {
        id: "test-1".to_string(),
        name: "nonexistent".to_string(),
        arguments: serde_json::json!({}),
    };

    let result = registry.execute(&call).await;
    assert!(result.is_error);
    assert!(result.output.contains("not found"));
}

#[tokio::test]
async fn tool_registry_multiple_tools() {
    let registry = ToolRegistry::new();

    registry.register(Box::new(ShellTool)).await;
    registry.register(Box::new(ReadFileTool)).await;
    registry.register(Box::new(WriteFileTool)).await;
    registry.register(Box::new(ListDirectoryTool)).await;

    let defs = registry.definitions().await;
    assert_eq!(defs.len(), 4);
}

// -----------------------------------------------------------------------
// Built-in Tool Tests
// -----------------------------------------------------------------------

#[tokio::test]
async fn shell_tool_execute_simple_command() {
    let tool = ShellTool;

    let call = ToolCall {
        id: "test-1".to_string(),
        name: "shell".to_string(),
        arguments: serde_json::json!({"command": "echo hello"}),
    };

    let result = tool.execute(&call).await;
    assert!(!result.is_error);
    assert!(result.output.contains("hello"));
}

#[tokio::test]
async fn shell_tool_missing_command_argument() {
    let tool = ShellTool;

    let call = ToolCall {
        id: "test-1".to_string(),
        name: "shell".to_string(),
        arguments: serde_json::json!({}),
    };

    let result = tool.execute(&call).await;
    assert!(result.is_error);
    assert!(result.output.contains("Missing 'command'"));
}

#[tokio::test]
async fn shell_tool_invalid_json_arguments() {
    let tool = ShellTool;

    let call = ToolCall {
        id: "test-1".to_string(),
        name: "shell".to_string(),
        arguments: serde_json::json!("not an object"),
    };

    let result = tool.execute(&call).await;
    // Should handle gracefully
    assert!(result.is_error || result.output.contains("Missing"));
}

#[tokio::test]
async fn read_file_tool_read_existing_file() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("test.txt");
    std::fs::write(&file_path, "test content").unwrap();

    let tool = ReadFileTool;

    let call = ToolCall {
        id: "test-1".to_string(),
        name: "read_file".to_string(),
        arguments: serde_json::json!({"path": file_path.to_str().unwrap()}),
    };

    let result = tool.execute(&call).await;
    assert!(!result.is_error);
    assert_eq!(result.output, "test content");
}

#[tokio::test]
async fn read_file_tool_nonexistent_file() {
    let tool = ReadFileTool;

    let call = ToolCall {
        id: "test-1".to_string(),
        name: "read_file".to_string(),
        arguments: serde_json::json!({"path": "/nonexistent/file.txt"}),
    };

    let result = tool.execute(&call).await;
    assert!(result.is_error);
    assert!(result.output.contains("Failed to read"));
}

#[tokio::test]
async fn write_file_tool_write_new_file() {
    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("output.txt");

    let tool = WriteFileTool;

    let call = ToolCall {
        id: "test-1".to_string(),
        name: "write_file".to_string(),
        arguments: serde_json::json!({
            "path": file_path.to_str().unwrap(),
            "content": "new content"
        }),
    };

    let result = tool.execute(&call).await;
    assert!(!result.is_error);

    let content = std::fs::read_to_string(&file_path).unwrap();
    assert_eq!(content, "new content");
}

#[tokio::test]
async fn list_directory_tool_list_current_dir() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("file1.txt"), "").unwrap();
    std::fs::write(dir.path().join("file2.txt"), "").unwrap();
    std::fs::create_dir(dir.path().join("subdir")).unwrap();

    let tool = ListDirectoryTool;

    let call = ToolCall {
        id: "test-1".to_string(),
        name: "list_directory".to_string(),
        arguments: serde_json::json!({"path": dir.path().to_str().unwrap()}),
    };

    let result = tool.execute(&call).await;
    assert!(!result.is_error);
    assert!(result.output.contains("file1.txt"));
    assert!(result.output.contains("file2.txt"));
    assert!(result.output.contains("subdir/"));
}

#[tokio::test]
async fn list_directory_tool_default_path() {
    let tool = ListDirectoryTool;

    let call = ToolCall {
        id: "test-1".to_string(),
        name: "list_directory".to_string(),
        arguments: serde_json::json!({}),
    };

    let result = tool.execute(&call).await;
    // Should list current directory without error
    assert!(!result.is_error);
}

// -----------------------------------------------------------------------
// Tool Definition Tests
// -----------------------------------------------------------------------

#[test]
fn tool_definition_has_correct_schema() {
    let tool = ShellTool;
    let def = tool.definition();

    assert_eq!(def.name, "shell");
    assert!(!def.description.is_empty());
    assert!(def.parameters.is_object());
    assert!(def.parameters["properties"]["command"].is_object());
}

#[test]
fn all_builtin_tools_have_definitions() {
    let tools: Vec<Box<dyn ToolExecutor>> = vec![
        Box::new(ShellTool),
        Box::new(ReadFileTool),
        Box::new(WriteFileTool),
        Box::new(ListDirectoryTool),
    ];

    for tool in tools {
        let def = tool.definition();
        assert!(!def.name.is_empty());
        assert!(!def.description.is_empty());
        assert!(def.parameters.is_object());
    }
}

// -----------------------------------------------------------------------
// Capability Tests
// -----------------------------------------------------------------------

#[test]
fn capabilities_supports_feature() {
    let caps = Capabilities {
        streaming: true,
        tool_calling: true,
        vision: true,
        ..Default::default()
    };

    assert!(caps.supports_feature("streaming"));
    assert!(caps.supports_feature("tool_calling"));
    assert!(caps.supports_feature("tools")); // alias
    assert!(caps.supports_feature("vision"));
    assert!(caps.supports_feature("image_input")); // alias
    assert!(!caps.supports_feature("reasoning"));
    assert!(!caps.supports_feature("embeddings"));
}

#[test]
fn capabilities_default_values() {
    let caps = Capabilities::default();

    assert!(!caps.streaming);
    assert!(!caps.tool_calling);
    assert!(!caps.vision);
    assert!(!caps.reasoning);
}

// -----------------------------------------------------------------------
// Vision Tests
// -----------------------------------------------------------------------

#[test]
fn image_content_creation() {
    let image = ImageContent {
        url: "data:image/png;base64,abc123".to_string(),
        detail: Some("high".to_string()),
    };

    assert_eq!(image.url, "data:image/png;base64,abc123");
    assert_eq!(image.detail, Some("high".to_string()));
}

#[test]
fn message_with_images() {
    let msg = Message {
        id: Uuid::new_v4(),
        conversation_id: Uuid::new_v4(),
        parent_id: None,
        role: Role::User,
        content: "What's in this image?".to_string(),
        reasoning_content: None,
        tool_calls: None,
        tool_call_id: None,
        images: Some(vec![ImageContent {
            url: "data:image/png;base64,test".to_string(),
            detail: None,
        }]),
        metadata: serde_json::json!({}),
        created_at: chrono::Utc::now(),
    };

    assert!(msg.images.is_some());
    assert_eq!(msg.images.as_ref().unwrap().len(), 1);
}

#[tokio::test]
async fn image_loading_from_file() {
    let dir = tempfile::tempdir().unwrap();
    let img_path = dir.path().join("test.png");

    // Create a minimal PNG file (1x1 pixel)
    let png_data = vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG signature
        0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, // IHDR chunk
        0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77,
        0x53, 0xDE,
    ];
    std::fs::write(&img_path, &png_data).unwrap();

    let result = llm_tui::image::load_image_from_path(img_path.to_str().unwrap());
    assert!(result.is_ok());

    let image = result.unwrap();
    assert!(image.url.starts_with("data:image/png;base64,"));
}

#[tokio::test]
async fn image_loading_nonexistent_file() {
    let result = llm_tui::image::load_image_from_path("/nonexistent/image.png");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}

#[tokio::test]
async fn image_loading_unsupported_format() {
    let dir = tempfile::tempdir().unwrap();
    let img_path = dir.path().join("test.bmp");
    std::fs::write(&img_path, "fake bmp").unwrap();

    let result = llm_tui::image::load_image_from_path(img_path.to_str().unwrap());
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Unsupported"));
}

// -----------------------------------------------------------------------
// Integration Tests
// -----------------------------------------------------------------------

#[tokio::test]
async fn app_has_tool_registry() {
    let (db, _dir) = test_db().await;
    let (event_tx, _event_rx) = mpsc::channel::<AppEvent>(256);
    let app = make_app(&db, event_tx);

    // Tool registry should be initialized
    let defs = app.tool_registry.definitions().await;
    // FakeProvider doesn't support tool calling, so no tools registered
    assert_eq!(defs.len(), 0);
}

#[tokio::test]
async fn app_pending_images_initially_empty() {
    let (db, _dir) = test_db().await;
    let (event_tx, _event_rx) = mpsc::channel::<AppEvent>(256);
    let app = make_app(&db, event_tx);

    assert!(app.pending_images.is_empty());
}

#[tokio::test]
#[ignore] // TODO: Fix image command handling in tests
async fn image_command_adds_to_pending_images() {
    let (db, _dir) = test_db().await;
    let (event_tx, _event_rx) = mpsc::channel::<AppEvent>(256);
    let mut app = make_app(&db, event_tx);

    // Create a test image
    let dir = tempfile::tempdir().unwrap();
    let img_path = dir.path().join("test.png");
    let png_data = vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00, 0x00, 0x90,
        0x77, 0x53, 0xDE,
    ];
    std::fs::write(&img_path, &png_data).unwrap();

    // Type /image command
    let cmd = format!("/image {}", img_path.display());
    for c in cmd.chars() {
        app.handle_event(AppEvent::User(UserEvent::InputChar(c)))
            .await;
    }
    app.handle_event(AppEvent::User(UserEvent::SendMessage))
        .await;

    // Check if there's an error
    if let Some(ref error) = app.error {
        panic!("Image command failed with error: {}", error);
    }

    // Image should be added to pending_images
    assert_eq!(app.pending_images.len(), 1);
    assert!(
        app.pending_images[0]
            .url
            .starts_with("data:image/png;base64,")
    );
}
