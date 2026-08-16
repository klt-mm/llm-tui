# llm-tui

A local-first terminal UI client for llama.cpp and OpenAI-compatible LLM endpoints.

## Project goals

- First-class llama.cpp experience.
- OpenAI-compatible provider support.
- Persistent local conversations.
- Prompt library.
- Full-text search.
- Streaming generation.
- Keyboard-first TUI.
- Provider capability detection.
- Architecture that remains maintainable for a solo developer.

## Non-goals for the first release

- Multi-user support.
- Cloud synchronization.
- Built-in RAG.
- Autonomous agents.
- Plugin marketplace.
- Complex server-side infrastructure.

## Current status

This repository is a **starter architecture**, not a production-ready client. The domain model, event model, database migration, provider boundary, and minimal TUI are present so implementation can proceed incrementally.

## Quick start

1. Install Rust.
2. Run:

```bash
cargo check
cargo test
cargo run
```

3. The application creates the SQLite database and runs migrations automatically.

You can point the future provider configuration at a local llama.cpp server such as:

```text
http://localhost:8080/v1
```

## Architecture

```text
TUI
  ↓
Application / Services
  ↓
Domain
  ├── LLM Provider
  └── Repositories
        ↓
      SQLite + FTS5
```

See `docs/` for the development plan and design contracts.
