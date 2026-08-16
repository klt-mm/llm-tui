# Development Guidelines

## Architecture

Use this dependency direction:

```text
TUI
 ↓
Application / Services
 ↓
Domain
 ↑
Infrastructure adapters
```

More concretely:

```text
TUI → services → domain
                 ↑
        provider / persistence
```

The domain must not import ratatui, sqlx, reqwest, or terminal APIs.

## Provider rules

- Use the application's domain request model.
- Translate to provider-specific wire formats at the boundary.
- Do not expose `reqwest::Response` to application code.
- Do not assume complete OpenAI compatibility.
- Capabilities determine UI availability.
- Provider-specific options belong in provider-specific configuration.

## Persistence rules

- Use repositories/services rather than SQL from widgets.
- Transactions should be used for multi-row logical operations.
- Persist completed messages atomically where possible.
- Do not write every streaming token to SQLite.
- Use migrations for schema evolution.

## UI rules

- Widgets render state.
- Widgets do not call HTTP directly.
- Keyboard input creates actions/events.
- Long operations are asynchronous.
- Modal state is explicit.
- Every important action should have a command-palette equivalent where practical.

## Error handling

Use typed errors at subsystem boundaries.

Errors should preserve:

- subsystem
- operation
- provider
- HTTP status where applicable
- safe diagnostic detail

Never display API keys or authorization headers.

## Logging

Use structured tracing.

Recommended levels:

- `error`: operation failed
- `warn`: recoverable unexpected condition
- `info`: lifecycle events
- `debug`: request/response metadata
- `trace`: low-level streaming diagnostics

Never log secrets or full prompts by default.

## Code quality

Prefer:

- small modules
- explicit types
- simple state transitions
- pure helper functions
- integration tests around boundaries

Avoid:

- global mutable state
- UI-owned business logic
- speculative abstractions
- generic frameworks created only for one use case
