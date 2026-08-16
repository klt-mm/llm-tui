PRAGMA foreign_keys = ON;

CREATE TABLE providers (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    base_url TEXT NOT NULL,
    protocol TEXT NOT NULL DEFAULT 'openai_compatible',
    api_key_ref TEXT,
    default_model TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE conversations (
    id TEXT PRIMARY KEY NOT NULL,
    title TEXT NOT NULL,
    provider_id TEXT NOT NULL,
    model_id TEXT NOT NULL,
    system_prompt TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    archived_at TEXT,
    FOREIGN KEY (provider_id) REFERENCES providers(id) ON DELETE RESTRICT
);

CREATE TABLE messages (
    id TEXT PRIMARY KEY NOT NULL,
    conversation_id TEXT NOT NULL,
    parent_id TEXT,
    role TEXT NOT NULL CHECK (role IN ('system', 'user', 'assistant', 'tool')),
    content TEXT NOT NULL,
    reasoning_content TEXT,
    metadata_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE CASCADE,
    FOREIGN KEY (parent_id) REFERENCES messages(id) ON DELETE SET NULL
);

CREATE INDEX idx_conversations_updated_at
    ON conversations(updated_at DESC);

CREATE INDEX idx_messages_conversation_created_at
    ON messages(conversation_id, created_at);

CREATE INDEX idx_messages_parent_id
    ON messages(parent_id);

CREATE TABLE prompts (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    content TEXT NOT NULL,
    system_prompt TEXT,
    tags_json TEXT NOT NULL DEFAULT '[]',
    variables_json TEXT NOT NULL DEFAULT '[]',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE models (
    provider_id TEXT NOT NULL,
    model_id TEXT NOT NULL,
    display_name TEXT,
    context_length INTEGER,
    metadata_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    PRIMARY KEY (provider_id, model_id),
    FOREIGN KEY (provider_id) REFERENCES providers(id) ON DELETE CASCADE
);

CREATE TABLE generation_runs (
    id TEXT PRIMARY KEY NOT NULL,
    message_id TEXT NOT NULL,
    provider_id TEXT NOT NULL,
    model_id TEXT NOT NULL,
    started_at TEXT NOT NULL,
    completed_at TEXT,
    prompt_tokens INTEGER,
    completion_tokens INTEGER,
    total_tokens INTEGER,
    prompt_ms REAL,
    generation_ms REAL,
    metadata_json TEXT NOT NULL DEFAULT '{}',
    FOREIGN KEY (message_id) REFERENCES messages(id) ON DELETE CASCADE,
    FOREIGN KEY (provider_id) REFERENCES providers(id) ON DELETE RESTRICT
);

CREATE VIRTUAL TABLE messages_fts USING fts5(
    message_id UNINDEXED,
    conversation_id UNINDEXED,
    content
);

CREATE VIRTUAL TABLE prompts_fts USING fts5(
    prompt_id UNINDEXED,
    name,
    description,
    content
);
