# Development Roadmap

## Phase 0 — Architecture foundation ✅ COMPLETE

Goal: establish stable boundaries before feature growth.

- [x] Domain types
- [x] Event model
- [x] Provider trait
- [x] SQLite migration
- [x] Repository interfaces
- [x] Fake provider
- [x] TUI event loop
- [x] Logging

Exit criteria:

- `cargo check` ✅
- `cargo test` ✅
- TUI launches and exits ✅
- SQLite migration succeeds ✅
- Fake provider can emit a stream ✅

## Phase 1 — llama.cpp chat ✅ COMPLETE

- [x] OpenAI-compatible provider
- [x] `/models`
- [x] streaming `/chat/completions`
- [x] connection diagnostics
- [x] model selection
- [x] send message
- [x] streaming rendering
- [x] cancellation
- [x] completed-message persistence

Exit criteria:

A user can start llama-server, connect, select a model, stream a response, stop it, and reopen the conversation. ✅

## Phase 2 — Core UX ✅ COMPLETE

- [x] Conversation sidebar
- [x] New conversation
- [x] Rename
- [x] Delete/archive
- [x] Retry
- [x] Markdown rendering
- [x] Syntax highlighting
- [x] Command palette
- [x] Keyboard help
- [x] Generation settings
- [x] Model selector UI
- [x] Provider selector UI

## Phase 3 — Prompt and search ✅ COMPLETE

- [x] Prompt CRUD
- [x] Prompt variables
- [x] Prompt tags
- [x] FTS5 indexing
- [x] Conversation search
- [x] Prompt search
- [x] Search result navigation
- [x] Prompt picker modal
- [x] Prompt list screen

## Phase 4 — Context engineering ✅ COMPLETE

- [x] Context policy
- [x] Token budget
- [x] Provider-aware token counting where available
- [x] Recent-history strategy
- [x] Summarization strategy
- [x] Context preview/debug view
- [x] Token budget display in status bar

## Phase 5 — Branching and diagnostics ✅ COMPLETE

- [x] Parent/branch UI
- [x] Retry as branch
- [x] Edit as branch
- [x] Generation run metrics
- [x] Tokens/sec
- [x] Prompt/generation latency
- [x] Request diagnostics
- [x] Branch history view
- [x] Generation run persistence

## Phase 6 — Advanced provider capabilities ✅ COMPLETE

- [x] Tool calling
- [x] Structured output (framework in place)
- [x] Vision
- [x] Reasoning display
- [x] Provider capability negotiation
- [x] llama.cpp-specific options (framework in place)
- [x] Built-in tools (shell, file operations)
- [x] Tool executor engine
- [x] Image loading and encoding

---

## Future Implementation Plan

### Phase 7 — Optional knowledge features

Only after the core client is stable:

- [ ] Embeddings
- [ ] Semantic search
- [ ] Local document indexing
- [ ] RAG (Retrieval-Augmented Generation)
- [ ] Context retrieval UI
- [ ] Vector storage integration
- [ ] Document chunking strategies
- [ ] Hybrid search (FTS + semantic)

### Phase 8 — Ecosystem

Only if there is a real need:

- [ ] Additional native protocols (Anthropic, Gemini, Bedrock)
- [ ] Import/export formats (JSON, Markdown, HTML)
- [ ] Scripting support (Lua, WASM, or JS)
- [ ] Plugin architecture
- [ ] MCP (Model Context Protocol) integration
- [ ] Multi-device sync
- [ ] Cloud storage integration
- [ ] Team collaboration features

---

## Version History

- **v1.0.2** (Current) — Phase 6 complete, production-ready
- **v0.1.0** — Initial architecture and core features
- **v0.2.0** — Phase 2-3: UX and prompt management
- **v0.3.0** — Phase 4-5: Context engineering and diagnostics
- **v1.0.0** — Phase 6: Advanced capabilities, production release

## Project Status

**Current Status:** Production Ready ✅

All core phases (0-6) are complete. The application is stable, well-tested, and ready for production use.

**Test Coverage:** 69+ tests passing
**Quality Gates:** All passing (cargo test, clippy, fmt)
**Documentation:** Comprehensive docs, GitHub Pages, installation scripts
