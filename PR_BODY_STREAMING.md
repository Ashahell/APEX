## Summary
- MVP: TinySSE-based in-memory streaming surface for Hands, MCP, and Task using iterator-based streams (futures_util::stream::iter) to avoid trait-object boxing issues.
- Relocation surface simplified to use single SSEItem type alias (Result<Event, axum::Error>).
- Tests added for streaming integration and TinySSE paths.
- CI updated to use Node 24 for compatibility.

## Changes
- **core/router/src/streaming.rs**: Clean baseline with iterator-based SSE streams for /hands, /mcp, /task endpoints. Re-exports StreamingMetrics, StreamingStats from streaming_types for API compatibility.
- **core/router/src/streaming_types.rs**: Simplified to SSEItem type alias. Removed complex TinySseStream aliases that caused type conflicts.
- **core/router/tests/streaming_tinysse_tests.rs**: Updated tests using proper futures_util types.
- **CI workflows**: Updated TypeScript and UI workflows to use Node 24 instead of deprecated Node 20.

## Validation
- All Rust tests pass (9 streaming_integration + 2 streaming_tinysse_tests + many more)
- Local validation commands:
  ```bash
  cargo test -p apex-router
  cargo fmt
  cargo clippy -p apex-router
  ```

## Test Results
```
test result: ok. 9 passed (streaming_integration)
test result: ok. 2 passed (streaming_tinysse_tests)
```

## Known Caveats
- Pre-existing clippy warnings in memory module (unrelated to streaming changes)
- Streaming module uses minimal baseline - can be extended with richer event data in follow-up patches
