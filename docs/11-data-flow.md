# Data Flow

## Send message

```text
User
 ↓
TUI input
 ↓
UserEvent::SendMessage
 ↓
ChatService
 ↓
ContextBuilder
 ↓
LlmProvider::stream_chat
 ↓
ProviderEvent::Delta*
 ↓
AppState
 ↓
TUI render
```

## Persistence

```text
User message
 ↓
MessageRepository
 ↓
SQLite

Assistant stream
 ↓
in-memory buffer
 ↓
generation complete
 ↓
MessageRepository
 ↓
SQLite
```

## Search

```text
Search input
 ↓
SearchService
 ↓
SQLite FTS5
 ↓
SearchResult[]
 ↓
TUI search view
```

## Provider abstraction

```text
ChatRequest
 ↓
OpenAiCompatibleProvider
 ↓
HTTP JSON
 ↓
SSE
 ↓
StreamEvent
 ↓
ChatService
```

## Context

```text
Conversation
 + system prompt
 + generation settings
 + context policy
        ↓
ContextBuilder
        ↓
bounded ChatRequest
```
