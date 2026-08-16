# Domain Model

## Design objective

The domain model must not depend on OpenAI JSON, llama.cpp JSON, SQLite rows, or TUI widgets.

The domain represents the application's concepts.

## Core entities

### Provider

Represents a configured inference endpoint.

Key fields:

- `id`
- `name`
- `base_url`
- `protocol`
- `api_key_ref`
- `default_model`
- timestamps

The initial protocol is `OpenAiCompatible`.

### Model

Represents a model exposed by a provider.

The same model identifier may exist on multiple providers, so the database key is `(provider_id, model_id)`.

### Conversation

A durable chat workspace.

Important fields:

- provider
- model
- title
- system prompt
- timestamps
- archive state

### Message

A durable node in a conversation.

`parent_id` intentionally supports branching even if the first UI is linear.

Roles:

- system
- user
- assistant
- tool

### Prompt

A reusable prompt template.

Variables are stored separately from the prompt body so the UI can provide interpolation and validation later.

### GenerationRun

A record of one model generation attempt.

It stores performance and usage metadata independently from message content.

This is useful for llama.cpp because generation throughput and timing are valuable diagnostics.

## Domain invariants

1. A message belongs to exactly one conversation.
2. A message's parent, when present, belongs to the same conversation.
3. A conversation references one provider and one selected model.
4. Provider-specific metadata must not become required domain fields.
5. Secrets are referenced, not stored as ordinary application data.
6. A generation run belongs to the assistant message it produced.
7. Deleted conversations cascade to their messages and generation runs.
8. Branching must never mutate an already persisted assistant response.

## Context model

The domain should eventually distinguish:

```text
Conversation
  ↓
ContextPolicy
  ↓
ContextBuilder
  ↓
Provider ChatRequest
```

The provider receives a constructed request, not the entire database conversation by default.
