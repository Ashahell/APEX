use apex_memory::tasks::TaskTier;
use once_cell::sync::Lazy;
use regex::Regex;

static INSTANT_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        // Very simple queries that don't need LLM
        Regex::new(r"(?i)^(what time is it|what's the time|tell me the time)[\s?]*$").unwrap(),
        Regex::new(r"(?i)^(date|what's the date|today's date)[\s?]*$").unwrap(),
    ]
});

pub struct TaskClassifier;

impl TaskClassifier {
    pub fn classify(content: &str) -> TaskTier {
        let content_lower = content.to_lowercase();

        // Check for very simple queries (Instant)
        for pattern in INSTANT_PATTERNS.iter() {
            if pattern.is_match(&content_lower) {
                return TaskTier::Instant;
            }
        }

        // Default to Deep - LLM handles most tasks including greetings and conversations
        TaskTier::Deep
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instant_classification() {
        assert_eq!(
            TaskClassifier::classify("what time is it?"),
            TaskTier::Instant
        );
        assert_eq!(
            TaskClassifier::classify("what's the date"),
            TaskTier::Instant
        );
    }

    #[test]
    fn test_deep_classification() {
        // Most inputs default to Deep
        assert_eq!(TaskClassifier::classify("hello"), TaskTier::Deep);
        assert_eq!(TaskClassifier::classify("hi"), TaskTier::Deep);
        assert_eq!(TaskClassifier::classify("build a website"), TaskTier::Deep);
        assert_eq!(TaskClassifier::classify("how are you"), TaskTier::Deep);
        assert_eq!(TaskClassifier::classify("morning"), TaskTier::Deep);
    }
}
