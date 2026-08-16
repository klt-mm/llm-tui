# Event Model

The application is event-driven.

## Event categories

### UserEvent

Produced by keyboard/UI interaction.

Examples:

- `Quit`
- `InputChanged`
- `SendMessage`
- `NewConversation`
- `CancelGeneration`
- `Retry`
- `OpenCommandPalette`

### ProviderEvent

Produced by asynchronous LLM operations.

Examples:

- models loaded
- capabilities loaded
- stream started
- text delta
- reasoning delta
- usage
- completed
- failed

### Internal events

As the project grows, add events for:

- database completion
- search completion
- context building
- export completion
- notifications

## Event flow

```text
Terminal / Network / DB
        ↓
      Event
        ↓
  App event handler
        ↓
    AppState mutation
        ↓
      Render
```

## Rules

1. Never block the TUI event loop on network I/O.
2. Long-running operations must be cancellable.
3. Streaming deltas should update volatile UI state first.
4. Persist the completed assistant message as one logical write.
5. Errors become explicit events rather than hidden logs.
6. UI widgets should react to state; they should not own business logic.

## Streaming lifecycle

```text
SendMessage
  ↓
StreamStarted
  ↓
Delta*
  ↓
Usage?
  ↓
Completed
```

Error path:

```text
StreamStarted
  ↓
Delta*
  ↓
Failed
```

Cancellation path:

```text
StreamStarted
  ↓
Delta*
  ↓
CancelGeneration
  ↓
provider cancellation
  ↓
application records partial result according to policy
```

The cancellation policy should be decided before implementing retries and branching.
