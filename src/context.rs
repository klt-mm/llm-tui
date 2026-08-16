use crate::domain::Message;

const CHARS_PER_TOKEN: usize = 4;
const MESSAGE_OVERHEAD_TOKENS: usize = 4;
const DEFAULT_RESERVE_TOKENS: usize = 1024;

pub fn token_estimate(text: &str) -> usize {
    text.len().div_ceil(CHARS_PER_TOKEN)
}

pub fn message_tokens(msg: &Message) -> usize {
    token_estimate(&msg.content) + MESSAGE_OVERHEAD_TOKENS
}

#[derive(Debug, Clone)]
pub struct ContextPolicy {
    pub max_tokens: Option<usize>,
    pub reserve_for_response: usize,
}

impl Default for ContextPolicy {
    fn default() -> Self {
        Self {
            max_tokens: None,
            reserve_for_response: DEFAULT_RESERVE_TOKENS,
        }
    }
}

pub fn build_context(
    system_prompt: Option<&str>,
    messages: &[Message],
    model_context_length: Option<u64>,
    policy: &ContextPolicy,
) -> Vec<Message> {
    let budget = policy
        .max_tokens
        .or_else(|| model_context_length.map(|c| c as usize))
        .unwrap_or(8192);

    let available = budget.saturating_sub(policy.reserve_for_response);

    let system_tokens = system_prompt
        .map(|s| token_estimate(s) + MESSAGE_OVERHEAD_TOKENS)
        .unwrap_or(0);

    let mut remaining = available.saturating_sub(system_tokens);
    let mut selected: Vec<usize> = Vec::new();

    for (i, msg) in messages.iter().enumerate().rev() {
        let cost = message_tokens(msg);
        if cost > remaining {
            break;
        }
        remaining -= cost;
        selected.push(i);
    }

    selected.reverse();

    let mut result = Vec::with_capacity(selected.len() + system_prompt.map_or(0, |_| 1));

    if let Some(prompt) = system_prompt {
        let sys_msg = Message {
            id: uuid::Uuid::nil(),
            conversation_id: uuid::Uuid::nil(),
            parent_id: None,
            role: crate::domain::Role::System,
            content: prompt.to_string(),
            reasoning_content: None,
            metadata: serde_json::json!({}),
            created_at: chrono::Utc::now(),
        };
        result.push(sys_msg);
    }

    for idx in selected {
        result.push(messages[idx].clone());
    }

    result
}

pub fn total_tokens(system_prompt: Option<&str>, messages: &[Message]) -> usize {
    let sys = system_prompt
        .map(|s| token_estimate(s) + MESSAGE_OVERHEAD_TOKENS)
        .unwrap_or(0);
    let msgs: usize = messages.iter().map(message_tokens).sum();
    sys + msgs
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Message, Role};
    use chrono::Utc;
    use uuid::Uuid;

    fn msg(role: Role, content: &str) -> Message {
        Message {
            id: Uuid::new_v4(),
            conversation_id: Uuid::new_v4(),
            parent_id: None,
            role,
            content: content.to_string(),
            reasoning_content: None,
            metadata: serde_json::json!({}),
            created_at: Utc::now(),
        }
    }

    #[test]
    fn token_estimate_basic() {
        assert_eq!(token_estimate(""), 0);
        assert_eq!(token_estimate("hi"), 1);
        assert_eq!(token_estimate("hello world"), 3);
        assert_eq!(token_estimate("a".repeat(100).as_str()), 25);
    }

    #[test]
    fn message_tokens_includes_overhead() {
        let m = msg(Role::User, "hello");
        let tokens = message_tokens(&m);
        assert_eq!(tokens, token_estimate("hello") + MESSAGE_OVERHEAD_TOKENS);
    }

    #[test]
    fn build_context_fits_all_messages() {
        let messages = vec![msg(Role::User, "hello"), msg(Role::Assistant, "hi there")];
        let policy = ContextPolicy {
            max_tokens: Some(8192),
            reserve_for_response: 1024,
        };
        let result = build_context(None, &messages, None, &policy);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn build_context_drops_oldest_when_over_budget() {
        let messages = vec![
            msg(Role::User, &"a".repeat(4000)),
            msg(Role::Assistant, &"b".repeat(4000)),
            msg(Role::User, "recent question"),
        ];
        let policy = ContextPolicy {
            max_tokens: Some(1100),
            reserve_for_response: 500,
        };
        let result = build_context(None, &messages, None, &policy);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].content, "recent question");
    }

    #[test]
    fn build_context_includes_system_prompt() {
        let messages = vec![msg(Role::User, "hello")];
        let policy = ContextPolicy {
            max_tokens: Some(8192),
            reserve_for_response: 1024,
        };
        let result = build_context(Some("You are helpful"), &messages, None, &policy);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].role, Role::System);
        assert_eq!(result[0].content, "You are helpful");
    }

    #[test]
    fn build_context_system_prompt_takes_priority() {
        let messages = vec![msg(Role::User, &"a".repeat(8000))];
        let policy = ContextPolicy {
            max_tokens: Some(2000),
            reserve_for_response: 500,
        };
        let result = build_context(Some("system prompt here"), &messages, None, &policy);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].role, Role::System);
    }

    #[test]
    fn build_context_uses_model_context_length() {
        let messages = vec![msg(Role::User, "hello")];
        let policy = ContextPolicy::default();
        let result = build_context(None, &messages, Some(4096), &policy);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn total_tokens_sums_all() {
        let messages = vec![msg(Role::User, "hello"), msg(Role::Assistant, "world")];
        let total = total_tokens(Some("system"), &messages);
        assert!(total > 0);
    }
}
