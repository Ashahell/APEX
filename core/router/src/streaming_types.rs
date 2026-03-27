use crate::execution_stream::ExecutionEvent;
use serde::Serialize;
use std::sync::atomic::{AtomicU64, Ordering};

/// Thread-safe streaming metrics — atomic counters for observability.
#[derive(Debug, Default)]
pub struct StreamingMetrics {
    pub active_connections: AtomicU64,
    pub total_connections: AtomicU64,
    pub events_thought: AtomicU64,
    pub events_tool_call: AtomicU64,
    pub events_tool_progress: AtomicU64,
    pub events_tool_result: AtomicU64,
    pub events_approval: AtomicU64,
    pub events_error: AtomicU64,
    pub events_complete: AtomicU64,
    pub errors_auth: AtomicU64,
    pub errors_replay: AtomicU64,
    pub errors_internal: AtomicU64,
}

impl StreamingMetrics {
    /// Increment active connection counter (called on stream connect).
    pub fn on_connect(&self) {
        self.active_connections.fetch_add(1, Ordering::Relaxed);
        self.total_connections.fetch_add(1, Ordering::Relaxed);
    }

    /// Decrement active connection counter (called on stream disconnect).
    pub fn on_disconnect(&self) {
        self.active_connections.fetch_sub(1, Ordering::Relaxed);
    }

    /// Increment event counter by type.
    pub fn on_event(&self, event: &ExecutionEvent) {
        match event {
            ExecutionEvent::Thought { .. } => {
                let _ = self.events_thought.fetch_add(1, Ordering::Relaxed);
            }
            ExecutionEvent::ToolCall { .. } => {
                let _ = self.events_tool_call.fetch_add(1, Ordering::Relaxed);
            }
            ExecutionEvent::ToolProgress { .. } => {
                let _ = self.events_tool_progress.fetch_add(1, Ordering::Relaxed);
            }
            ExecutionEvent::ToolResult { .. } => {
                let _ = self.events_tool_result.fetch_add(1, Ordering::Relaxed);
            }
            ExecutionEvent::ApprovalNeeded { .. } => {
                let _ = self.events_approval.fetch_add(1, Ordering::Relaxed);
            }
            ExecutionEvent::Error { .. } => {
                let _ = self.events_error.fetch_add(1, Ordering::Relaxed);
            }
            ExecutionEvent::Complete { .. } => {
                let _ = self.events_complete.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    /// Increment error counter by category.
    pub fn on_error(&self, kind: &str) {
        match kind {
            "auth" => {
                let _ = self.errors_auth.fetch_add(1, Ordering::Relaxed);
            }
            "replay" => {
                let _ = self.errors_replay.fetch_add(1, Ordering::Relaxed);
            }
            _ => {
                let _ = self.errors_internal.fetch_add(1, Ordering::Relaxed);
            }
        }
    }
}

/// Snapshot of streaming metrics for API responses.
#[derive(Debug, Default, Serialize)]
pub struct StreamingStats {
    pub active_connections: u64,
    pub total_connections: u64,
    pub events: EventCounts,
    pub errors: ErrorCounts,
}

#[derive(Debug, Default, Serialize)]
pub struct EventCounts {
    pub thought: u64,
    pub tool_call: u64,
    pub tool_progress: u64,
    pub tool_result: u64,
    pub approval_needed: u64,
    pub error: u64,
    pub complete: u64,
    pub total: u64,
}

#[derive(Debug, Default, Serialize)]
pub struct ErrorCounts {
    pub auth: u64,
    pub replay: u64,
    pub internal: u64,
    pub total: u64,
}

impl From<&StreamingMetrics> for StreamingStats {
    fn from(m: &StreamingMetrics) -> Self {
        let thought = m.events_thought.load(Ordering::Relaxed);
        let tool_call = m.events_tool_call.load(Ordering::Relaxed);
        let tool_progress = m.events_tool_progress.load(Ordering::Relaxed);
        let tool_result = m.events_tool_result.load(Ordering::Relaxed);
        let approval = m.events_approval.load(Ordering::Relaxed);
        let error = m.events_error.load(Ordering::Relaxed);
        let complete = m.events_complete.load(Ordering::Relaxed);
        let total_events =
            thought + tool_call + tool_progress + tool_result + approval + error + complete;

        let auth = m.errors_auth.load(Ordering::Relaxed);
        let replay = m.errors_replay.load(Ordering::Relaxed);
        let internal = m.errors_internal.load(Ordering::Relaxed);
        let total_errors = auth + replay + internal;

        StreamingStats {
            active_connections: m.active_connections.load(Ordering::Relaxed),
            total_connections: m.total_connections.load(Ordering::Relaxed),
            events: EventCounts {
                thought,
                tool_call,
                tool_progress,
                tool_result,
                approval_needed: approval,
                error,
                complete,
                total: total_events,
            },
            errors: ErrorCounts {
                auth,
                replay,
                internal,
                total: total_errors,
            },
        }
    }
}

// Unit tests for StreamingMetrics surface and mapping to StreamingStats
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::Ordering;

    #[test]
    fn default_stats_zero() {
        let s = StreamingStats::default();
        assert_eq!(s.active_connections, 0);
        assert_eq!(s.total_connections, 0);
        assert_eq!(s.events.total, 0);
        assert_eq!(s.errors.total, 0);
    }

    #[test]
    fn from_zero_mapping() {
        let sm = StreamingMetrics::default();
        let s = StreamingStats::from(&sm);
        assert_eq!(s.active_connections, 0);
        assert_eq!(s.total_connections, 0);
        assert_eq!(s.events.thought, 0);
        assert_eq!(s.errors.total, 0);
    }

    #[test]
    fn from_values_mapping() {
        let mut sm = StreamingMetrics::default();
        sm.active_connections.fetch_add(3, Ordering::Relaxed);
        sm.total_connections.fetch_add(3, Ordering::Relaxed);
        sm.events_thought.fetch_add(2, Ordering::Relaxed);
        sm.errors_auth.fetch_add(1, Ordering::Relaxed);

        let s = StreamingStats::from(&sm);
        assert_eq!(s.active_connections, 3);
        assert_eq!(s.total_connections, 3);
        assert_eq!(s.events.thought, 2);
        assert_eq!(s.errors.auth, 1);
    }
}
