# First Sprint

The first sprint should prove the architecture, not maximize features.

## Goal

A fake provider can stream a response into the TUI and the resulting conversation can be persisted and restored.

## Tasks

- [ ] Implement `AppState`
- [ ] Implement `AppEvent` routing
- [ ] Implement `FakeProvider`
- [ ] Implement provider task spawning
- [ ] Implement stream event handling
- [ ] Add conversation repository
- [ ] Add message repository
- [ ] Add SQLite integration tests
- [ ] Add minimal chat rendering
- [ ] Add graceful cancellation
- [ ] Add one end-to-end application test

## Definition of done

The following flow works:

```text
launch
  ↓
new conversation
  ↓
type message
  ↓
send
  ↓
fake provider streams
  ↓
tokens appear incrementally
  ↓
generation completes
  ↓
assistant message persisted
  ↓
application restarts
  ↓
conversation remains available
```

Do not implement prompts, RAG, tools, or multiple providers during this sprint.
