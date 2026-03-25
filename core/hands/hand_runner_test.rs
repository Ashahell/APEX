#[cfg(test)]
mod tests {
    use super::super::HandRunner;

    #[test]
    fn run_once_should_change_state_to_completed() {
        let mut hr = HandRunner::new("test");
        hr.run_once();
        // After run_once, the hand should be non-active
        assert!(!hr.status());
    }

    #[test]
    fn run_with_multiple_steps_should_not_panic() {
        let mut hr = HandRunner::new("multi");
        hr.run(3);
        assert!(!hr.status());
    }
}
