pub mod orchestrator;
pub mod screenshot;
pub mod actions;
pub mod vlm;
pub mod orchestrator_v2;

// Re-export commonly used types for easier imports from other modules
pub use orchestrator::{ComputerUseConfig, ExecutionContext, ComputerUseOrchestrator, ExecutionResult, OrchestratorError, OrchestratorState};
pub use screenshot::{ScreenshotConfig, ScreenshotManager, CapturedScreenshot, ScreenshotError};
pub use actions::{ComputerAction, MouseButton, KeyModifier, ActionExecutor, ActionError, ActionResult, ActionRecord};
pub use vlm::{VLMController, VLMConfig, VLMProvider, VLMResponse, VLMError};
