# Extension Guidelines

This document governs future development so the project does not become an abstraction-heavy monolith.

## Adding a provider

First ask:

> Can this provider be represented by the existing OpenAI-compatible provider?

If yes, add configuration only.

If no, implement a new `LlmProvider`.

A provider must translate between:

```text
application domain
      ↕
provider wire protocol
```

Do not leak provider-specific structures upward.

## Adding a feature

A feature should normally have:

```text
domain concept
→ application service/action
→ persistence if required
→ UI
→ tests
→ documentation
```

Avoid implementing UI-first features that have no domain model.

## Adding provider-specific options

Prefer:

```text
ProviderConfig
  generic options
  provider_options JSON
```

over adding many optional fields to the generic `ChatRequest`.

Promote an option to the generic model only when multiple providers share the same semantic meaning.

## Adding tools

Introduce domain concepts such as:

```text
ToolDefinition
ToolCall
ToolResult
```

Keep execution behind a service boundary.

Do not let the TUI execute arbitrary tools directly.

## Adding vision

Represent attachments in the domain:

```text
MessagePart
  ├── Text
  └── Image
```

Do not force multimodal content into a single plain string.

## Adding RAG

Do not couple retrieval directly to the provider.

Use:

```text
Conversation
  ↓
ContextBuilder
  ↓
Retriever
  ↓
Context items
  ↓
Provider
```

The model should not know where context came from.

## Adding MCP

Treat MCP as an integration boundary, not as the application's core abstraction.

A future architecture can be:

```text
ToolRegistry
  ├── Built-in tools
  ├── MCP tools
  └── Provider-native tools
```

The application works with a common `ToolDefinition` / `ToolCall` model.

## Adding synchronization

Do not add sync logic to repositories.

Introduce a synchronization layer above persistence:

```text
local database
      ↕
sync service
      ↕
remote store
```

Local-first behavior should remain possible without the remote service.

## Backward compatibility

For persisted data:

- never silently reinterpret old records
- migrate schema explicitly
- preserve export compatibility
- version exported formats
- test upgrades from previous versions

## Complexity rule

Before introducing a new abstraction, identify at least two concrete consumers.

If there is only one consumer, prefer the simplest implementation that satisfies the current requirement.
