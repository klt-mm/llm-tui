# QWEN.md — llm-tui

## Project Overview

**llm-tui** is a local-first, keyboard-driven terminal UI client for llama.cpp and OpenAI-compatible LLM endpoints. Written in Rust, it provides persistent local conversations (SQLite + FTS5), a prompt library, streaming generation, and provider capability detection. The project is currently in starter-architecture stage — domain model, event model, database migration, provider boundary, and minimal TUI scaffolding are in place; implementation proceeds incrementally.

## Tech Stack

- **Language:** Rust (edition 2024)
- **TUI:** ratatui + crossterm
- **Async runtime:** tokio
- **Database:** SQLite via sqlx (with FTS5 full-text search, migrations)
- **HTTP / Streaming:** reqwest (rustls-tls, streaming SSE)
- **Serialization:** serde / serde_json
- **Error handling:** thiserror + anyhow
- **Logging:** tracing + tracing-subscriber (env-filter)
- **IDs:** uuid v4

## Build & Run

```bash
cargo check          # type-check
cargo test           # run all tests
cargo run            # build and launch (creates sqlite DB + runs migrations automatically)
```

The database defaults to `./llm-tui.db`; override with `DATABASE_URL`.
Log level defaults to `llm_tui=debug`; override with `RUST_LOG`.

## Architecture

```
TUI (ratatui)
  ↓
Application / Services (app.rs, events.rs)
  ↓
Domain (src/domain/)          ← must not import ratatui, sqlx, reqwest, or terminal APIs
  ├── LLM Provider (src/llm/)
  └── Repositories (src/persistence/)
        ↓
      SQLite + FTS5
```

Key modules:
- `src/domain/` — pure domain types: Provider, Model, Conversation, Message (branch-capable via `parent_id`), Prompt, GenerationRun
- `src/llm/` — provider adapters (OpenAI-compatible wire format)
- `src/persistence/` — Database connection, repositories, migrations (`migrations/001_initial.sql`)
- `src/events.rs` — event-driven state transitions (AppEvent / UserEvent)
- `src/tui.rs` — terminal UI rendering and input handling
- `src/app.rs` — application state and event dispatch

## Development Conventions

### Dependency direction (strict)
```
TUI → services → domain ← provider / persistence
```
The domain layer must never depend on infrastructure crates.

### Provider rules
- Translate domain ↔ wire format at the provider boundary; never expose `reqwest::Response` to application code.
- Do not assume full OpenAI compatibility; use capability detection.

### Persistence rules
- Access the DB through repositories/services, never directly from widgets.
- Persist completed messages atomically; do not write every streaming token.
- Use migrations for schema evolution.

### Error handling
- Typed errors (`thiserror`) at subsystem boundaries.
- Never log/display API keys or auth headers.

### Logging
- Structured `tracing`. Levels: error / warn / info / debug / trace.
- Never log secrets or full prompts by default.

### Code style
- Small modules, explicit types, simple state transitions, pure helpers.
- Avoid global mutable state, UI-owned business logic, speculative abstractions.

## Testing

- **Unit tests** — pure behavior (conversions, parsing, budgeting, interpolation).
- **Provider contract tests** — same behavioral contract for every provider impl.
- **Persistence integration tests** — use `tempfile` for a temporary SQLite DB; test CRUD, branching, cascade deletes, FTS.
- **Application integration tests** — test flows end-to-end (new conversation → stream → reopen → restore).
- **TUI snapshot tests** — layout-focused, ignore dynamic timestamps.
- **Failure tests** — server unavailable, timeout, malformed SSE/JSON, cancellation, SQLite locked.

Run tests: `cargo test`

## Key Docs

See `docs/` for detailed design contracts:
- `01-domain-model.md` — entities and invariants
- `02-event-model.md` — event taxonomy
- `03-sqlite-schema.md` — schema design
- `04-development-roadmap.md` — release plan
- `06-task-board.md` — current work
- `10-first-sprint.md` — sprint scope
- `11-data-flow.md` — runtime data flow
