use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskOutcome {
    pub task_id: String,
    pub content: String,
    pub tier: String,
    pub status: String,
    pub steps_taken: u32,
    pub tools_used: Vec<String>,
    pub success: bool,
    pub execution_time_ms: u64,
    pub cost_cents: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyInsight {
    pub pattern: String,
    pub success_rate: f64,
    pub avg_steps: f64,
    pub avg_time_ms: u64,
    pub recommendation: String,
}

pub struct CurriculumAgent {
    outcomes: Mutex<Vec<TaskOutcome>>,
}

impl CurriculumAgent {
    pub fn new() -> Self {
        Self {
            outcomes: Mutex::new(Vec::new()),
        }
    }

    pub fn record_outcome(&self, outcome: TaskOutcome) {
        if let Ok(mut outcomes) = self.outcomes.lock() {
            outcomes.push(outcome);
        }
    }

    pub fn get_insights(&self) -> Vec<StrategyInsight> {
        let outcomes = match self.outcomes.lock() {
            Ok(o) => o,
            Err(_) => return Vec::new(),
        };

        if outcomes.is_empty() {
            return Vec::new();
        }

        let total = outcomes.len() as f64;
        let successful = outcomes.iter().filter(|o| o.success).count() as f64;
        let success_rate = successful / total;
        let avg_steps: f64 = outcomes.iter().map(|o| o.steps_taken as f64).sum::<f64>() / total;
        let avg_time: u64 =
            outcomes.iter().map(|o| o.execution_time_ms).sum::<u64>() / outcomes.len() as u64;

        let recommendation = if success_rate > 0.8 {
            format!(
                "High success rate ({:.0}%) - current strategy effective",
                success_rate * 100.0
            )
        } else if success_rate < 0.5 {
            "Low success rate - consider breaking into smaller subtasks".to_string()
        } else {
            "Moderate success rate - room for optimization".to_string()
        };

        vec![StrategyInsight {
            pattern: "overall".to_string(),
            success_rate,
            avg_steps,
            avg_time_ms: avg_time,
            recommendation,
        }]
    }

    pub fn suggest_strategy(&self, _task_content: &str) -> String {
        let insights = self.get_insights();

        if insights.is_empty() {
            return "No historical data - using default strategy".to_string();
        }

        "Based on historical performance, consider adjusting strategy based on task complexity."
            .to_string()
    }
}

impl Default for CurriculumAgent {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_and_get_insights() {
        let agent = CurriculumAgent::new();

        agent.record_outcome(TaskOutcome {
            task_id: "1".to_string(),
            content: "test".to_string(),
            tier: "deep".to_string(),
            status: "completed".to_string(),
            steps_taken: 5,
            tools_used: vec!["shell".to_string()],
            success: true,
            execution_time_ms: 1000,
            cost_cents: 50,
        });

        let insights = agent.get_insights();
        assert!(!insights.is_empty());
        assert_eq!(insights[0].pattern, "overall");
        assert_eq!(insights[0].success_rate, 1.0);
    }

    #[test]
    fn test_suggest_strategy() {
        let agent = CurriculumAgent::new();

        let suggestion = agent.suggest_strategy("complex task");
        assert!(suggestion.contains("default strategy"));

        agent.record_outcome(TaskOutcome {
            task_id: "1".to_string(),
            content: "test".to_string(),
            tier: "deep".to_string(),
            status: "completed".to_string(),
            steps_taken: 5,
            tools_used: vec![],
            success: true,
            execution_time_ms: 1000,
            cost_cents: 50,
        });

        let suggestion = agent.suggest_strategy("another task");
        assert!(suggestion.contains("historical performance") || suggestion.contains("adjusting"));
    }
}
