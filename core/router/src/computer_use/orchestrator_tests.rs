#[cfg(test)]
mod tests {
    use super::super::ComputerUseOrchestrator;
    use crate::computer_use::orchestrator::OrchestratorError;

    #[tokio::test]
    async fn mvp_runs_without_panic() {
        let mut orch = ComputerUseOrchestrator::new();
        let res = orch.execute("demo-mvp").await;
        // MVP path uses stubs; expect success with stub final_state
        assert!(res.is_ok(), "execute should succeed in MVP stub mode");
        let result = res.unwrap();
        assert_eq!(result.steps, 1);
        assert!(result.success);
    }
}
