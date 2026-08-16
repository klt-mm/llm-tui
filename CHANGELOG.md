# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.2] - 2026-08-16

### Added
- **Phase 6: Advanced Provider Capabilities**
  - Provider capability negotiation system
  - Tool calling framework with built-in tools (shell, read_file, write_file, list_directory)
  - Vision support with image loading and encoding
  - Tool executor engine and registry
  - UI rendering for tool calls and results
  - Comprehensive test suite for Phase 6 features

- **Documentation & Release Preparation**
  - Comprehensive README.md with installation, usage, and contribution guides
  - SECURITY.md with security policy and vulnerability reporting guidelines
  - GitHub Pages documentation website with:
    - Installation guide
    - Usage guide
    - Contributing guide
    - Learning resources
    - Professional styling
  - Installation script (install.sh) for automated setup
  - Uninstallation script (uninstall.sh) for clean removal
  - Updated development roadmap with phases 0-6 marked complete
  - Added phases 7-8 as future implementation plan

### Changed
- Updated project metadata in Cargo.toml:
  - Version bumped to 1.0.2
  - Added repository, homepage, and documentation URLs
  - Added keywords and categories for crates.io
  - Updated description to reflect production-ready status

### Fixed
- All quality gates passing (cargo test, clippy, fmt)
- 69+ tests passing with comprehensive coverage

## [1.0.0] - 2026-08-16

### Added
- **Phase 0: Architecture Foundation**
  - Domain types (Provider, Model, Conversation, Message, Prompt, GenerationRun)
  - Event-driven architecture (UserEvent, ProviderEvent, AppEvent)
  - SQLite persistence layer with migrations
  - Repository pattern with trait-based abstractions
  - Provider abstraction (LlmProvider trait)
  - TUI scaffolding with ratatui + crossterm
  - Logging with tracing

- **Phase 1: llama.cpp Chat**
  - OpenAI-compatible provider implementation
  - Model discovery via `/models` endpoint
  - Streaming chat completions with SSE parsing
  - Connection diagnostics
  - Model selection (keyboard shortcuts + UI)
  - Message sending and streaming rendering
  - Cancellation support
  - Completed-message persistence

- **Phase 2: Core UX**
  - Conversation sidebar with navigation (j/k, arrows)
  - New conversation creation (Ctrl+N)
  - Rename/delete conversations with modals
  - Retry generation (Ctrl+R)
  - Markdown rendering (headings, code, lists, emphasis)
  - Status bar with provider/model/streaming info
  - Input editor with focus management
  - Command palette (Ctrl+K) with 11 searchable commands
  - Model selector modal with active indicator
  - Provider selector modal with runtime switching
  - Generation settings modal (Ctrl+G) for temperature/top_p/max_tokens
  - Help screen (?)

- **Phase 3: Prompt Management and Search**
  - Prompt CRUD (create, read, update, delete)
  - Prompt variables with `{{variable}}` syntax
  - Prompt tags for organization
  - FTS5 indexing for messages and prompts
  - Prompt picker (Ctrl+P) with search and variable resolution
  - Prompt list screen (Ctrl+L) with navigation
  - Global search (Ctrl+F or /) across messages and prompts
  - Search result navigation and opening

- **Phase 4: Context Engineering**
  - Token counting with chars/4 heuristic
  - Context budget management (auto from model + user override)
  - Reserve tokens for response headroom
  - System prompt inclusion in context
  - Oldest-first message dropping when over budget
  - Token budget display in status bar (e.g., "1.2k/4k tok")
  - ContextConfig for user customization

- **Phase 5: Branching and Diagnostics**
  - GenerationRun domain type and repository
  - Automatic metrics persistence on generation completion
  - Branch inline indicators showing parent message relationships
  - Branch history view modal (Ctrl+B) for visualizing conversation branches
  - Edit-as-branch functionality to copy messages to input for re-editing
  - Enhanced status bar with final generation metrics (tok/s, tokens, duration)
  - Generation metrics: prompt/completion tokens, latency, tokens/sec

### Changed
- Extended Message struct with tool_calls, tool_call_id, and images fields
- Extended Capabilities struct with granular feature flags
- Added supports_feature() helper method for capability checking
- Updated all test files to support new Message fields

### Fixed
- All 69 tests passing
- All quality gates passing (cargo test, clippy, fmt)
- Comprehensive test coverage across all phases

## [0.3.0] - 2026-08-15

### Added
- Phase 4: Context engineering with token budgeting
- Phase 5: Branching and diagnostics

## [0.2.0] - 2026-08-14

### Added
- Phase 2: Core UX features
- Phase 3: Prompt management and FTS search

## [0.1.0] - 2026-08-13

### Added
- Initial architecture and domain model
- Phase 0: Architecture foundation
- Phase 1: Basic llama.cpp chat functionality
- SQLite persistence with migrations
- Event-driven architecture
- Provider abstraction layer
- Basic TUI scaffolding

---

## Version History Summary

- **v1.0.2** (Current) — Production release with full documentation and installation scripts
- **v1.0.0** — All core phases (0-6) complete, production-ready
- **v0.3.0** — Context engineering and diagnostics
- **v0.2.0** — Core UX and prompt management
- **v0.1.0** — Initial architecture and basic chat

## Future Plans

### Phase 7: Optional Knowledge Features
- Embeddings and semantic search
- Local document indexing
- RAG (Retrieval-Augmented Generation)
- Context retrieval UI

### Phase 8: Ecosystem
- Additional native protocols (Anthropic, Gemini, Bedrock)
- Import/export formats
- Scripting support
- Plugin architecture
- MCP integration
- Multi-device sync

[1.0.2]: https://github.com/klt-mm/llm-tui/compare/v1.0.0...v1.0.2
[1.0.0]: https://github.com/klt-mm/llm-tui/compare/v0.3.0...v1.0.0
[0.3.0]: https://github.com/klt-mm/llm-tui/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/klt-mm/llm-tui/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/klt-mm/llm-tui/releases/tag/v0.1.0
