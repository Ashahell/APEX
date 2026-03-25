# Capability Gates (Final MVP)

- Goal: enforce basic permission gating for critical actions in MVP flows.
- MVP: define a small gate model and a function gate_action(action: &str, required_tier: Tier) -> bool.
- Tier enum: T0, T1, T2, T3
- Basic usage: before performing any risky action in Hands or LLM tasks, call gate_action with the required tier.
- Future: tie to a policy store and sign-off workflow.
