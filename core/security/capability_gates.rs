// Minimal capability gates scaffold
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityTier {
    T0,
    T1,
    T2,
    T3,
}

pub struct GatePolicy {
    pub allowed_actions: Vec<String>,
    pub gate_tier: CapabilityTier,
}

impl GatePolicy {
    pub fn new(gate_tier: CapabilityTier, allowed_actions: Vec<String>) -> Self {
        Self {
            gate_tier,
            allowed_actions,
        }
    }
    pub fn can_perform(&self, action: &str) -> bool {
        self.allowed_actions.iter().any(|a| a.as_str() == action)
    }
}
