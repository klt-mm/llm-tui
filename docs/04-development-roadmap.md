# Development Roadmap

## Phase 0 — Architecture foundation

Goal: establish stable boundaries before feature growth.

- [ ] Domain types
- [ ] Event model
- [ ] Provider trait
- [ ] SQLite migration
- [ ] Repository interfaces
- [ ] Fake provider
- [ ] TUI event loop
- [ ] Logging

Exit criteria:

- `cargo check`
- `cargo test`
- TUI launches and exits
- SQLite migration succeeds
- Fake provider can emit a stream

## Phase 1 — llama.cpp chat

- [ ] OpenAI-compatible provider
- [ ] `/models`
- [ ] streaming `/chat/completions`
- [ ] connection diagnostics
- [ ] model selection
- [ ] send message
- [ ] streaming rendering
- [ ] cancellation
- [ ] completed-message persistence

Exit criteria:

A user can start llama-server, connect, select a model, stream a response, stop it, and reopen the conversation.

## Phase 2 — Core UX

- [ ] Conversation sidebar
- [ ] New conversation
- [ ] Rename
- [ ] Delete/archive
- [ ] Retry
- [ ] Markdown rendering
- [ ] Syntax highlighting
- [ ] Command palette
- [ ] Keyboard help
- [ ] Generation settings

## Phase 3 — Prompt and search

- [ ] Prompt CRUD
- [ ] Prompt variables
- [ ] Prompt tags
- [ ] FTS5 indexing
- [ ] Conversation search
- [ ] Prompt search
- [ ] Search result navigation

## Phase 4 — Context engineering

- [ ] Context policy
- [ ] Token budget
- [ ] Provider-aware token counting where available
- [ ] Recent-history strategy
- [ ] Summarization strategy
- [ ] Context preview/debug view

## Phase 5 — Branching and diagnostics

- [ ] Parent/branch UI
- [ ] Retry as branch
- [ ] Edit as branch
- [ ] Generation run metrics
- [ ] Tokens/sec
- [ ] Prompt/generation latency
- [ ] Request diagnostics

## Phase 6 — Advanced provider capabilities

- [ ] Tool calling
- [ ] Structured output
- [ ] Vision
- [ ] Reasoning display
- [ ] Provider capability negotiation
- [ ] llama.cpp-specific options

## Phase 7 — Optional knowledge features

Only after the core client is stable:

- [ ] Embeddings
- [ ] Semantic search
- [ ] Local document indexing
- [ ] RAG
- [ ] Context retrieval UI

## Phase 8 — Ecosystem

Only if there is a real need:

- [ ] Additional native protocols
- [ ] Import/export formats
- [ ] Scripting
- [ ] Plugins
- [ ] MCP
- [ ] Sync
