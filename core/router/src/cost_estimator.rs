use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostEstimate {
    pub estimated_cost_usd: f64,
    pub confidence: f64,
    pub factors: Vec<CostFactor>,
    pub breakdown: CostBreakdown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostFactor {
    pub name: String,
    pub impact: f64,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostBreakdown {
    pub llm_cost: f64,
    pub compute_cost: f64,
    pub storage_cost: f64,
    pub network_cost: f64,
}

pub struct CostEstimator;

impl CostEstimator {
    const BASE_LLM_COST_PER_1K_TOKENS: f64 = 0.001;
    const COMPUTE_COST_PER_SECOND: f64 = 0.0001;
    const STORAGE_COST_PER_GB_DAY: f64 = 0.023;
    const NETWORK_COST_PER_MB: f64 = 0.0001;

    pub fn estimate(content: &str, max_steps: u32) -> CostEstimate {
        let token_count = Self::estimate_tokens(content);
        let estimated_steps = max_steps.min(10);

        let input_tokens = token_count;
        let output_tokens = token_count * 2;
        let total_tokens = input_tokens + output_tokens * estimated_steps as usize;

        let llm_cost = (total_tokens as f64 / 1000.0) * Self::BASE_LLM_COST_PER_1K_TOKENS;

        let compute_seconds = estimated_steps as f64 * 30.0;
        let compute_cost = compute_seconds * Self::COMPUTE_COST_PER_SECOND;

        let storage_gb = 0.01;
        let storage_cost = storage_gb * Self::STORAGE_COST_PER_GB_DAY;

        let network_mb = (total_tokens as f64 / 1000.0) * 4.0;
        let network_cost = network_mb * Self::NETWORK_COST_PER_MB;

        let total_cost = llm_cost + compute_cost + storage_cost + network_cost;

        let complexity = Self::analyze_complexity(content);
        let confidence = (1.0 - (complexity / 10.0)).max(0.5);

        let factors = vec![
            CostFactor {
                name: "Prompt Length".to_string(),
                impact: token_count as f64 / 100.0,
                description: format!("{} tokens estimated", token_count),
            },
            CostFactor {
                name: "Step Count".to_string(),
                impact: estimated_steps as f64 * 0.1,
                description: format!("{} steps requested", estimated_steps),
            },
            CostFactor {
                name: "Complexity".to_string(),
                impact: complexity * 0.05,
                description: format!("Complexity score: {}", complexity),
            },
        ];

        CostEstimate {
            estimated_cost_usd: (total_cost * 100.0).round() / 100.0,
            confidence,
            factors,
            breakdown: CostBreakdown {
                llm_cost: (llm_cost * 100.0).round() / 100.0,
                compute_cost: (compute_cost * 100.0).round() / 100.0,
                storage_cost: (storage_cost * 100.0).round() / 100.0,
                network_cost: (network_cost * 100.0).round() / 100.0,
            },
        }
    }

    fn estimate_tokens(content: &str) -> usize {
        content.split_whitespace().count() * 4 / 3
    }

    fn analyze_complexity(content: &str) -> f64 {
        let mut score = 0.0;

        let keywords = [
            "analyze",
            "implement",
            "create",
            "build",
            "fix",
            "debug",
            "refactor",
            "optimize",
        ];
        for kw in keywords {
            if content.to_lowercase().contains(kw) {
                score += 1.0;
            }
        }

        if content.contains("```") {
            score += 2.0;
        }

        if content.len() > 500 {
            score += 1.0;
        }

        if content.to_lowercase().contains("test") {
            score += 0.5;
        }

        score
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cost_estimation() {
        let estimate = CostEstimator::estimate("Implement a new feature", 5);
        assert!(estimate.estimated_cost_usd > 0.0);
        assert!(estimate.confidence > 0.0 && estimate.confidence <= 1.0);
    }

    #[test]
    fn test_token_estimation() {
        let tokens = CostEstimator::estimate_tokens("Hello world this is a test");
        assert!(tokens > 0);
    }
}
