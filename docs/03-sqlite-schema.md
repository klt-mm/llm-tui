# SQLite Schema

SQLite is the authoritative local store for application data.

## Tables

### providers

Configured LLM endpoints.

### models

Cached model metadata per provider.

### conversations

Durable conversation metadata.

### messages

Conversation messages and branch relationships.

### prompts

Reusable prompt templates.

### generation_runs

Performance and usage metrics for generations.

### FTS tables

`messages_fts` and `prompts_fts` provide local full-text search.

## Why FTS5

The application needs search over potentially thousands of messages, but does not need an external search service.

FTS5 provides:

- tokenized full-text search
- prefix search
- boolean queries
- ranking facilities
- local operation

## Migration rules

1. Every schema change gets a new migration.
2. Never edit an applied migration.
3. Migration filenames are monotonically ordered.
4. Destructive migrations require explicit review.
5. Test migrations against a fresh database and an upgraded database.
6. Foreign-key enforcement must remain enabled.

## Future schema additions

Possible later tables:

```text
conversation_tags
message_attachments
tools
tool_calls
embedding_documents
embedding_chunks
settings
```

Do not add them until the corresponding feature exists.

## Data retention

Conversation deletion should be explicit.

Exports are not the database's responsibility.

Secrets should live in the OS credential store when available.
