use futures_core::Stream;
use futures_util::stream;
use std::pin::Pin;

// TinySSE surface for MCP. This is intentionally small and isolated to avoid churn in streaming.rs.
pub struct TinySseMcpSurface;

impl TinySseMcpSurface {
    pub fn new() -> TinySseMcpSurface {
        TinySseMcpSurface
    }

    // Return a minimal, empty TinySSE stream for MCP events.
    // In subsequent patches, this will be wired to publish real MCP events.
    pub fn stream(&self) -> Pin<Box<dyn Stream<Item = Result<String, String>> + Send>> {
        Box::pin(stream::empty())
    }
}
