# Task Board

This is the working backlog. Move completed items to the changelog rather than deleting them.

## Foundation

- [ ] Create application state model
- [ ] Define action/event reducers
- [ ] Add fake LLM provider
- [ ] Add provider configuration
- [ ] Add SQLite repository traits
- [ ] Add repository integration tests
- [ ] Add application error type
- [ ] Add tracing initialization

## Provider

- [ ] Implement model discovery
- [ ] Implement chat request conversion
- [ ] Implement SSE parser
- [ ] Handle `[DONE]`
- [ ] Handle malformed frames
- [ ] Handle HTTP errors
- [ ] Handle cancellation
- [ ] Detect server capabilities
- [ ] Add mock OpenAI-compatible HTTP server tests

## Chat

- [ ] Create conversation
- [ ] Persist user message
- [ ] Start generation
- [ ] Render deltas
- [ ] Persist assistant message
- [ ] Persist generation metrics
- [ ] Stop generation
- [ ] Retry generation
- [ ] Branch generation

## TUI

- [ ] Sidebar
- [ ] Chat viewport
- [ ] Input editor
- [ ] Status bar
- [ ] Command palette
- [ ] Provider selector
- [ ] Model selector
- [ ] Generation settings modal
- [ ] Prompt picker
- [ ] Search screen
- [ ] Help screen

## Persistence

- [ ] Provider CRUD
- [ ] Model cache
- [ ] Conversation CRUD
- [ ] Message CRUD
- [ ] Prompt CRUD
- [ ] Generation run persistence
- [ ] FTS message indexing
- [ ] FTS prompt indexing

## Quality

- [ ] Unit tests
- [ ] Integration tests
- [ ] Snapshot tests
- [ ] Migration tests
- [ ] Cancellation tests
- [ ] Error-path tests
- [ ] Large-history performance test
- [ ] Streaming stress test
