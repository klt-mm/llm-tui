# Testing Strategy

Testing should protect architecture boundaries rather than maximize line coverage.

## Unit tests

Test pure behavior:

- role conversion
- request conversion
- generation parameter mapping
- context budgeting
- prompt interpolation
- command parsing
- branch calculations
- FTS query construction
- SSE frame parsing

## Provider contract tests

Every provider implementation should satisfy the same behavioral contract:

1. model discovery works
2. chat request maps correctly
3. streaming produces ordered deltas
4. `[DONE]` ends the stream
5. HTTP errors become `LlmError`
6. cancellation stops work
7. malformed provider data does not panic

## Persistence integration tests

Use a temporary SQLite database.

Test:

- migration from empty DB
- provider CRUD
- conversation CRUD
- message ordering
- parent/branch relationships
- cascade deletion
- FTS indexing/search
- generation-run persistence

## Application integration tests

Test flows rather than widgets:

```text
new conversation
→ user message
→ provider stream
→ assistant message
→ reopen conversation
→ messages restored
```

Also:

```text
user message
→ provider failure
→ error state
→ retry
→ new assistant branch
```

## TUI snapshot tests

Snapshot:

- empty application
- chat with messages
- streaming state
- error state
- command palette
- search results
- generation settings

Snapshots should focus on layout, not dynamic timestamps.

## Performance tests

Measure:

- startup time
- database open/migration time
- search latency
- rendering latency with large messages
- memory usage with long conversations
- streaming throughput
- cancellation latency

## Failure tests

Explicitly test:

- server unavailable
- timeout
- malformed JSON
- malformed SSE
- connection reset
- model unavailable
- context too large
- SQLite locked
- disk full where practical
- cancellation during streaming
