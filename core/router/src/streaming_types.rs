use crate::execution_stream::ExecutionEvent;
use axum::response::sse::Event;
use serde::Serialize;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// Streaming event types - formal SSE envelope
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum StreamEventType {
    Connected,
    Disconnected,
    Hands,
    Mcp,
    Task,
    Stats,
    Heartbeat,
    Error,
    // Phase 1: Richer event types
    SessionStart,
    SessionEnd,
    Checkpoint,
    UserIntervention,
}

impl StreamEventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            StreamEventType::Connected => "connected",
            StreamEventType::Disconnected => "disconnected",
            StreamEventType::Hands => "hands",
            StreamEventType::Mcp => "mcp",
            StreamEventType::Task => "task",
            StreamEventType::Stats => "stats",
            StreamEventType::Heartbeat => "heartbeat",
            StreamEventType::Error => "error",
            // Phase 1: Richer event types
            StreamEventType::SessionStart => "session_start",
            StreamEventType::SessionEnd => "session_end",
            StreamEventType::Checkpoint => "checkpoint",
            StreamEventType::UserIntervention => "user_intervention",
        }
    }
}

/// Formal SSE envelope for streaming events
#[derive(Debug, Serialize)]
pub struct SseEnvelope<T> {
    #[serde(rename = "type")]
    pub event_type: StreamEventType,
    pub timestamp: u64,
    pub trace_id: Option<String>,
    pub payload: T,
}

impl<T> SseEnvelope<T> {
    pub fn new(event_type: StreamEventType, payload: T) -> Self {
        Self {
            event_type,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
            trace_id: None,
            payload,
        }
    }

    pub fn with_trace(mut self, trace_id: String) -> Self {
        self.trace_id = Some(trace_id);
        self
    }
}

/// Payload for connection events
#[derive(Debug, Serialize)]
pub struct ConnectionPayload {
    pub task_id: String,
    pub connection_id: String,
    pub message: String,
}

/// Payload for heartbeat events
#[derive(Debug, Serialize)]
pub struct HeartbeatPayload {
    pub server_time: u64,
    pub active_connections: u64,
}

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
    // Phase 1: Additional event types
    pub events_session_start: AtomicU64,
    pub events_session_end: AtomicU64,
    pub events_checkpoint: AtomicU64,
    pub events_user_intervention: AtomicU64,
    // Error counters
    pub errors_auth: AtomicU64,
    pub errors_replay: AtomicU64,
    pub errors_internal: AtomicU64,
    // Phase 1: Performance metrics
    pub connection_duration_total_ms: AtomicU64,
    pub events_per_second_sum: AtomicU64,
}

// TinySSE scaffolding removed in favor of adapter-based lines using SSEItem only
pub type SSEItem = Result<Event, axum::Error>;

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

    /// Track connection duration in milliseconds
    pub fn on_disconnect_duration(&self, duration_ms: u64) {
        self.connection_duration_total_ms
            .fetch_add(duration_ms, Ordering::Relaxed);
    }

    /// Track events per second
    pub fn on_events_per_second(&self, rate: u64) {
        self.events_per_second_sum
            .fetch_add(rate, Ordering::Relaxed);
    }

    /// Increment Phase 1 event type counter
    pub fn on_stream_event(&self, event_type: StreamEventType) {
        match event_type {
            StreamEventType::SessionStart => {
                let _ = self.events_session_start.fetch_add(1, Ordering::Relaxed);
            }
            StreamEventType::SessionEnd => {
                let _ = self.events_session_end.fetch_add(1, Ordering::Relaxed);
            }
            StreamEventType::Checkpoint => {
                let _ = self.events_checkpoint.fetch_add(1, Ordering::Relaxed);
            }
            StreamEventType::UserIntervention => {
                let _ = self
                    .events_user_intervention
                    .fetch_add(1, Ordering::Relaxed);
            }
            // Legacy events handled by ExecutionEvent::on_event
            _ => {}
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
    // Phase 1: Performance metrics
    pub performance: PerformanceMetrics,
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
    // Phase 1: Additional event types
    pub session_start: u64,
    pub session_end: u64,
    pub checkpoint: u64,
    pub user_intervention: u64,
    pub total: u64,
}

#[derive(Debug, Default, Serialize)]
pub struct ErrorCounts {
    pub auth: u64,
    pub replay: u64,
    pub internal: u64,
    pub total: u64,
}

/// Phase 1: Performance metrics for SLO tracking
#[derive(Debug, Default, Serialize)]
pub struct PerformanceMetrics {
    pub connection_duration_total_ms: u64,
    pub events_per_second_sum: u64,
    /// Calculated average connection duration in ms
    pub avg_connection_duration_ms: u64,
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
        // Phase 1: Additional events
        let session_start = m.events_session_start.load(Ordering::Relaxed);
        let session_end = m.events_session_end.load(Ordering::Relaxed);
        let checkpoint = m.events_checkpoint.load(Ordering::Relaxed);
        let user_intervention = m.events_user_intervention.load(Ordering::Relaxed);

        let total_events = thought
            + tool_call
            + tool_progress
            + tool_result
            + approval
            + error
            + complete
            + session_start
            + session_end
            + checkpoint
            + user_intervention;

        let auth = m.errors_auth.load(Ordering::Relaxed);
        let replay = m.errors_replay.load(Ordering::Relaxed);
        let internal = m.errors_internal.load(Ordering::Relaxed);
        let total_errors = auth + replay + internal;

        // Phase 1: Performance metrics
        let connection_duration_total_ms = m.connection_duration_total_ms.load(Ordering::Relaxed);
        let events_per_second_sum = m.events_per_second_sum.load(Ordering::Relaxed);
        let total_conns = m.total_connections.load(Ordering::Relaxed);
        let avg_duration_ms = if total_conns > 0 {
            connection_duration_total_ms / total_conns
        } else {
            0
        };

        StreamingStats {
            active_connections: m.active_connections.load(Ordering::Relaxed),
            total_connections: total_conns,
            events: EventCounts {
                thought,
                tool_call,
                tool_progress,
                tool_result,
                approval_needed: approval,
                error,
                complete,
                session_start,
                session_end,
                checkpoint,
                user_intervention,
                total: total_events,
            },
            errors: ErrorCounts {
                auth,
                replay,
                internal,
                total: total_errors,
            },
            performance: PerformanceMetrics {
                connection_duration_total_ms,
                events_per_second_sum,
                avg_connection_duration_ms: avg_duration_ms,
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
