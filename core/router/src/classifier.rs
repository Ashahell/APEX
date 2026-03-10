use apex_memory::tasks::TaskTier;
use once_cell::sync::Lazy;
use regex::Regex;

static INSTANT_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        // Very simple queries that don't need LLM
        Regex::new(r"(?i)^(what time is it|what's the time|tell me the time)[\s?]*$").unwrap(),
        Regex::new(r"(?i)^(date|what's the date|today's date)[\s?]*$").unwrap(),
        // Simple math
        Regex::new(r"(?i)^(what is \d+ [\+\-\*\/] \d+)[\s?]*$").unwrap(),
        // Simple factual
        Regex::new(r"(?i)^(who is |what is )[\w\s]+[\s?]*$").unwrap(),
    ]
});

static SHALLOW_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| {
    vec![
        // Skill execution patterns
        Regex::new(r"(?i)(generate|create|build|make)\s+(code|file|function|class|component)")
            .unwrap(),
        Regex::new(r"(?i)(commit|push|pull|branch)\s+(git|github)").unwrap(),
        Regex::new(r"(?i)(run|execute|start|stop)\s+(docker|container|pod)").unwrap(),
        Regex::new(r"(?i)(deploy|install|setup|configure)\s+").unwrap(),
        Regex::new(r"(?i)(format|lint|test)\s+(code|file)").unwrap(),
        Regex::new(r"(?i)(delete|remove|drop)\s+(file|table|database)").unwrap(),
        // Short commands
        Regex::new(r"^/[a-z]+\s+").unwrap(),
    ]
});

pub struct TaskClassifier;

impl TaskClassifier {
    /// Classify task content into appropriate tier
    ///
    /// Rules:
    /// - Instant: Simple queries that can be answered without LLM or skills
    /// - Shallow: Tasks that can be handled by a single skill execution
    /// - Deep: Complex tasks requiring LLM reasoning, multiple steps, or planning
    pub fn classify(content: &str) -> TaskTier {
        let content_lower = content.to_lowercase();

        // Check for instant patterns (simple queries)
        for pattern in INSTANT_PATTERNS.iter() {
            if pattern.is_match(&content_lower) {
                return TaskTier::Instant;
            }
        }

        // Check for shallow patterns (skill execution)
        for pattern in SHALLOW_PATTERNS.iter() {
            if pattern.is_match(&content_lower) {
                return TaskTier::Shallow;
            }
        }

        // Check for explicit skill mentions - these are Shallow
        let skill_keywords = [
            "git ", "docker ", "kubectl ", "npm ", "pnpm ", "cargo ", "file.", "delete ",
            "deploy ", "build ", "test ", "lint ",
        ];
        for keyword in skill_keywords {
            if content_lower.contains(keyword) {
                return TaskTier::Shallow;
            }
        }

        // Check for explicit LLM requests - these are Deep
        let deep_keywords = [
            "explain",
            "why",
            "how does",
            "analyze",
            "compare",
            "design",
            "architect",
            "plan",
            "strategy",
            "review",
            "refactor",
            "write code for",
            "implement",
            "create a system",
        ];
        for keyword in deep_keywords {
            if content_lower.contains(keyword) {
                return TaskTier::Deep;
            }
        }

        // Default to Shallow for short actionable commands, Deep for longer content
        if content.len() < 100 {
            TaskTier::Shallow
        } else {
            TaskTier::Deep
        }
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
        assert_eq!(TaskClassifier::classify("what is 2 + 2"), TaskTier::Instant);
        assert_eq!(
            TaskClassifier::classify("who is Elon Musk"),
            TaskTier::Instant
        );
    }

    #[test]
    fn test_shallow_classification() {
        // Skill execution patterns
        assert_eq!(
            TaskClassifier::classify("generate a function"),
            TaskTier::Shallow
        );
        assert_eq!(
            TaskClassifier::classify("git commit -m 'fix bug'"),
            TaskTier::Shallow
        );
        assert_eq!(
            TaskClassifier::classify("docker build -t myapp ."),
            TaskTier::Shallow
        );
        assert_eq!(
            TaskClassifier::classify("delete the file"),
            TaskTier::Shallow
        );
        // Short commands default to shallow
        assert_eq!(TaskClassifier::classify("run tests"), TaskTier::Shallow);
    }

    #[test]
    fn test_deep_classification() {
        // Complex tasks requiring LLM
        assert_eq!(
            TaskClassifier::classify("explain how this code works"),
            TaskTier::Deep
        );
        assert_eq!(TaskClassifier::classify("why is this slow"), TaskTier::Deep);
        assert_eq!(
            TaskClassifier::classify("design a system architecture"),
            TaskTier::Deep
        );
        // Long content with reasoning keywords defaults to deep
        assert_eq!(
            TaskClassifier::classify(
                "I need help analyzing why the system is slow and how to optimize it"
            ),
            TaskTier::Deep
        );
    }
}
