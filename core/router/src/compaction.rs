//! Chat Compaction Service
//!
//! Reduces context window usage by summarizing older messages.
//! Inspired by Hermes chat compaction plugin.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactionConfig {
    pub enabled: bool,
    pub threshold_percent: u8,
    pub preserve_recent: usize,
    pub max_summary_length: usize,
}

impl Default for CompactionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            threshold_percent: 50,
            preserve_recent: 10,
            max_summary_length: 500,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub role: String,
    pub content: String,
    pub timestamp: Option<String>,
    pub tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub input: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactionResult {
    pub original_count: usize,
    pub compacted_count: usize,
    pub summary: String,
    pub removed_message_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactionSummary {
    pub message_count: usize,
    pub token_estimate: usize,
    pub should_compact: bool,
}

pub struct ChatCompaction {
    config: CompactionConfig,
}

impl ChatCompaction {
    pub fn new(config: CompactionConfig) -> Self {
        Self { config }
    }

    pub fn should_compact(&self, messages: &[ChatMessage]) -> bool {
        if !self.config.enabled {
            return false;
        }

        let total = messages.len();
        if total < self.config.preserve_recent * 2 {
            return false;
        }

        let token_estimate = Self::estimate_tokens(messages);
        let threshold_tokens = 128000 * self.config.threshold_percent as usize / 100;
        
        token_estimate > threshold_tokens
    }

    pub fn compact(&self, messages: &[ChatMessage]) -> CompactionResult {
        let original_count = messages.len();
        
        if messages.len() <= self.config.preserve_recent {
            return CompactionResult {
                original_count,
                compacted_count: original_count,
                summary: String::new(),
                removed_message_ids: vec![],
            };
        }

        let preserve_count = self.config.preserve_recent;
        let to_compact = &messages[..messages.len() - preserve_count];
        let to_preserve = &messages[messages.len() - preserve_count..];

        let mut summary = String::new();
        let mut removed_ids = Vec::new();

        if !to_compact.is_empty() {
            summary = Self::generate_summary(to_compact);
            if summary.len() > self.config.max_summary_length {
                summary.truncate(self.config.max_summary_length);
                summary.push_str("...");
            }

            for msg in to_compact {
                removed_ids.push(msg.id.clone());
            }
        }

        let compacted_count = to_preserve.len() + 1;

        CompactionResult {
            original_count,
            compacted_count,
            summary,
            removed_message_ids: removed_ids,
        }
    }

    pub fn estimate_compaction_summary(&self, messages: &[ChatMessage]) -> CompactionSummary {
        let message_count = messages.len();
        let token_estimate = Self::estimate_tokens(messages);
        let should_compact = self.should_compact(messages);

        CompactionSummary {
            message_count,
            token_estimate,
            should_compact,
        }
    }

    fn estimate_tokens(messages: &[ChatMessage]) -> usize {
        messages
            .iter()
            .map(|m| {
                let content_tokens = m.content.len() / 4;
                let tool_tokens = m.tool_calls.as_ref().map(|tc| {
                    tc.iter()
                        .map(|t| t.name.len() + t.input.to_string().len())
                        .sum::<usize>() / 4
                }).unwrap_or(0);
                content_tokens + tool_tokens + 10
            })
            .sum()
    }

    fn generate_summary(messages: &[ChatMessage]) -> String {
        let mut summary_parts = Vec::new();
        
        for msg in messages.iter().take(20) {
            let role_prefix = match msg.role.as_str() {
                "user" => "User",
                "assistant" => "Assistant",
                "system" => "System",
                _ => "Unknown",
            };
            
            let content_preview = if msg.content.len() > 100 {
                format!("{}...", &msg.content[..100])
            } else {
                msg.content.clone()
            };
            
            summary_parts.push(format!("{}: {}", role_prefix, content_preview));
        }

        let combined = summary_parts.join("\n");
        
        if combined.len() > 500 {
            format!("[Summary of {} messages]\n{}", messages.len(), &combined[..500])
        } else {
            format!("[Summary of {} messages]\n{}", messages.len(), combined)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_messages(count: usize) -> Vec<ChatMessage> {
        (0..count)
            .map(|i| ChatMessage {
                id: format!("msg-{}", i),
                role: if i % 2 == 0 { "user".to_string() } else { "assistant".to_string() },
                content: format!("This is test message number {} with some additional content to increase token count.", i),
                timestamp: None,
                tool_calls: None,
            })
            .collect()
    }

    #[test]
    fn test_compaction_disabled() {
        let config = CompactionConfig {
            enabled: false,
            ..Default::default()
        };
        let compaction = ChatCompaction::new(config);
        
        let messages = create_test_messages(100);
        assert!(!compaction.should_compact(&messages));
    }

    #[test]
    fn test_compaction_preserves_recent() {
        let config = CompactionConfig {
            enabled: true,
            preserve_recent: 5,
            ..Default::default()
        };
        let compaction = ChatCompaction::new(config);
        
        let messages = create_test_messages(20);
        let result = compaction.compact(&messages);
        
        assert_eq!(result.compacted_count, 6);
    }

    #[test]
    fn test_estimate_tokens() {
        let messages = create_test_messages(10);
        let tokens = ChatCompaction::estimate_tokens(&messages);
        
        assert!(tokens > 0);
    }

    #[test]
    fn test_should_compact_below_threshold() {
        let config = CompactionConfig {
            enabled: true,
            threshold_percent: 1,
            ..Default::default()
        };
        let compaction = ChatCompaction::new(config);
        
        let messages = create_test_messages(5);
        assert!(!compaction.should_compact(&messages));
    }

    #[test]
    fn test_summary_generation() {
        let messages = vec![
            ChatMessage {
                id: "1".to_string(),
                role: "user".to_string(),
                content: "Hello".to_string(),
                timestamp: None,
                tool_calls: None,
            },
            ChatMessage {
                id: "2".to_string(),
                role: "assistant".to_string(),
                content: "Hi there!".to_string(),
                timestamp: None,
                tool_calls: None,
            },
        ];
        
        let summary = ChatCompaction::generate_summary(&messages);
        assert!(summary.contains("Summary of 2 messages"));
    }
}
