//! Minimal embedding wiring integration helper for MVP.
//!
//! Uses apex_memory embedder for embedding lookups (Patch 10 integration point).

use apex_memory::embedder::{Embedder, EmbeddingProvider};

/// Build an embedder configured for the given task context.
/// Returns a new Embedder instance ready for embedding queries.
pub fn make_embedder_for_task(_task_id: &str) -> Embedder {
    // MVP: use local embedder with default settings
    // Production: would configure based on task requirements
    Embedder::new(
        EmbeddingProvider::Local {
            url: "http://localhost:8081".to_string(),
            model: "nomic-embed-text".to_string(),
        },
        768, // embedding dimension for nomic-embed-text
    )
}
