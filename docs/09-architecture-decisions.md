# Architecture Decisions

## ADR-001: Rust

Decision: Rust.

Reasoning:

- strong fit for TUI applications
- excellent async ecosystem
- predictable memory behavior
- easy distribution as a single native binary
- good terminal and systems integration

## ADR-002: SQLite

Decision: SQLite as the local source of truth.

Reasoning:

- single-user local workload
- zero server administration
- transactional
- mature
- FTS5
- easy backup/export

## ADR-003: OpenAI-compatible provider first

Decision: use OpenAI-compatible APIs as the first provider boundary.

Reasoning:

- llama.cpp already exposes compatible endpoints
- broad ecosystem compatibility
- minimizes provider-specific code
- allows the application to remain backend-neutral

## ADR-004: Event-driven TUI

Decision: state transitions are driven by events.

Reasoning:

- streaming is naturally event-oriented
- avoids blocking the terminal
- improves testability
- separates input, network, persistence, and rendering

## ADR-005: Branch-capable message graph

Decision: messages include `parent_id`.

Reasoning:

- retry and edit can create branches
- avoids destructive mutation
- enables future tree navigation

## ADR-006: No external database service

Decision: do not require PostgreSQL or another server database.

Reasoning:

The product is local-first and solo-user. An external database would add operational complexity without a corresponding product benefit.
