// Minimal Hand runner skeleton for MVP with lifecycle state
pub enum LifecycleState {
    Idle,
    Running,
    Completed,
}

pub struct HandRunner {
    pub name: String,
    pub active: bool,
    pub state: LifecycleState,
}

impl HandRunner {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            active: false,
            state: LifecycleState::Idle,
        }
    }
    pub fn start(&mut self) {
        self.active = true;
        self.state = LifecycleState::Running;
    }
    pub fn stop(&mut self) {
        self.active = false;
        self.state = LifecycleState::Completed;
    }
    pub fn status(&self) -> bool {
        self.active
    }
}

impl HandRunner {
    // Simple one-shot run to demonstrate lifecycle without external deps
    pub fn run_once(&mut self) {
        self.start();
        // simulate a tiny amount of work without blocking
        self.stop();
    }
    // Lightweight in-process lifecycle simulation
    pub fn run(&mut self, steps: usize) {
        self.start();
        // pretend to do 'steps' of work
        for _ in 0..steps { /* no-op */ }
        self.stop();
    }
    // Convenience constructor with initial state
    pub fn new_with_state(name: &str, state: LifecycleState) -> Self {
        HandRunner {
            name: name.to_string(),
            active: state == LifecycleState::Running,
            state,
        }
    }
}
