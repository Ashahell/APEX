# APEX Performance Profiling Report

**Date:** 2026-04-25  
**Status:** Complete

---

## Executive Summary

| Layer | Metric | Value | Status |
|-------|--------|-------|--------|
| **Rust Core** | Binary size | 20.5 MB | ⚠️ Large |
| **Rust Core** | Build time | 87s (release) | ⚠️ Slow |
| **Rust Core** | Test count | 544 tests | ✅ |
| **UI (React)** | TypeScript | Passes | ✅ |
| **Python** | Test count | 64 tests | ✅ |

---

## Rust Core (L2/L3)

### Build Performance
- **Release build time:** 87 seconds
- **Incremental build:** ~10-20s (after first build)

### Binary Size
- **apex-router.exe:** 20.5 MB
- **apex-router-bin.exe:** 3.56 MB (slim)

### Dependency Analysis
Key dependencies (tree):
- `tokio` (async runtime)
- `axum` (HTTP framework)
- `sqlx` (SQLite + runtime)
- `reqwest` (HTTP client)
- `serde` (serialization)
- `tracing` (logging)

### Duplicate Dependencies (Potential Optimization)
- `chrono` - appears 3x (use feature flags)
- `futures-channel` - appears with multiple versions
- `sqlx-core` - appears 4x

### Recommendations for Rust
1. **Enable LTO (Link Time Optimization)** - `cargo build --release -Clto=true`
2. **Use codegen-units=1** - `cargo build --release -Ccodegen-units=1`
3. **Trim dependencies** - Review feature flags on tokio, axum, sqlx
4. **Split binary** - Consider separating router-bin (3.5 MB) as standalone

---

## TypeScript/UI (L6)

### TypeScript Checks
- **tsc --noEmit:** Passes ✅
- **No type errors found**

### Dependencies
React ecosystem:
- `@radix-ui/react-*` (7 components)
- `@tanstack/react-query` (data fetching)
- `framer-motion` (animations)
- `socket.io-client` (WebSocket)
- `react-markdown` (rendering)

### Recommendations for UI
1. **Lazy loading** - Use React.lazy() for route-based code splitting
2. **Tree shaking** - Ensure Radix components are individually imported
3. **Bundle analysis** - Run `rollup-plugin-visualizer` to see chunk sizes

---

## Python Execution (L5)

### Test Coverage
- **Test count:** 64 tests
- **Modules:**
  - `test_enforcement.py` - Security enforcement
  - `test_sandbox.py` - Execution sandbox
  - `test_agent_config.py` - Configuration

### Dependencies
- `poetry` for package management
- Standard Python 3.11+ typing

### Recommendations for Python
1. **Type hints** - Add strict mypy checking
2. **pytest --co** - Collect and profile test collection time
3. **pytest-benchmark** - Add performance benchmarks

---

## Test Suite Summary

| Component | Tests |
|-----------|-------|
| Rust unit | 72 |
| Rust integration | 349 |
| Rust e2e | 2 |
| Python | 64 |
| **Total** | **487** |

---

## Performance Optimizations Applied

### Rust Build Profile (2026-04-25)
```toml
[profile.release]
lto = true           # Link Time Optimization
codegen-units = 1    # Better optimization
opt-level = 3       # Maximum optimization
strip = true        # Remove debug symbols
```

**Result:**
| Binary | Before | After | Reduction |
|--------|--------|-------|-----------|
| apex-router.exe | 20.5 MB | 18.56 MB | **9.5%** |
| apex-router-bin.exe | 3.56 MB | 1.62 MB | **54.5%** |

### UI Build Optimization
- 5 vendor chunks: react, radix, motion, query, markdown
- 28 lazy-loaded routes (already existed)
- Chunk size warning limit: 500 KB

### Test Results
All 544 Rust tests pass after optimization.

---

## Performance Recommendations

### High Priority
1. ~~**Add LTO to Rust build**~~ - ✅ Done, 9.5% size reduction
2. ~~**Enable incremental compilation**~~ - Already in use
3. ~~**Add Rust codegen-units=1**~~ - ✅ Done

### Medium Priority
1. ~~**Split apex-router binary**~~ - ✅ Already split (bin vs exe)
2. ~~**Lazy load React components**~~ - ✅ Already done (28 routes)
3. **Add bundle analyzer** - Chunk splitting in place

### Low Priority
1. **Add benchmarks** - tokio-benches for async hot paths
2. **Profile SQLite queries** - Enable `EXPLAIN QUERY PLAN`
3. **Cache embedding model** - Pre-load for cold start

---

## Next Steps

1. ✅ Run `cargo build --release` with LTO - done
2. ✅ Lazy load React routes - done
3. Add vite-bundle-visualizer for detailed chunk analysis
4. Add pytest performance markers for Python