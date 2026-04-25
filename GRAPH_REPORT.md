# Graph Report - .  (2026-04-23)

## Corpus Check
- Corpus is ~344 words - fits in a single context window. You may not need a graph.

## Summary
- 7 nodes · 9 edges · 2 communities detected
- Extraction: 100% EXTRACTED · 0% INFERRED · 0% AMBIGUOUS
- Token cost: 0 input · 0 output

## Community Hubs (Navigation)
- [[_COMMUNITY_TinySSE Test Functions|TinySSE Test Functions]]
- [[_COMMUNITY_TinySseTest Struct|TinySseTest Struct]]

## God Nodes (most connected - your core abstractions)
1. `TinySseTest` - 3 edges
2. `hands_tinysse_three_events()` - 2 edges
3. `mcp_tinysse_three_events()` - 2 edges
4. `task_tinysse_three_events()` - 2 edges

## Surprising Connections (you probably didn't know these)
- None detected - all connections are within the same source files.

## Communities

### Community 0 - "TinySSE Test Functions"
Cohesion: 0.6
Nodes (3): hands_tinysse_three_events(), mcp_tinysse_three_events(), task_tinysse_three_events()

### Community 1 - "TinySseTest Struct"
Cohesion: 1.0
Nodes (1): TinySseTest

## Knowledge Gaps
- **Thin community `TinySseTest Struct`** (2 nodes): `TinySseTest`, `.poll()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `TinySseTest` connect `TinySseTest Struct` to `TinySSE Test Functions`?**
  _High betweenness centrality (0.350) - this node is a cross-community bridge._