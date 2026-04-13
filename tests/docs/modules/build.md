# Module: build.rs

## Purpose

**Merge extractions into a graph** - combine nodes/edges from multiple files, deduplicate, add metadata.

---

## What It Does

Given multiple `ExtractionResult` objects:
1. Merge all nodes (deduplicate by ID)
2. Merge all edges (deduplicate by source+target+relation)
3. Create `GraphData` structure
4. Add community detection

---

## Input/Output

**Input:**
```rust
// From file 1
ExtractionResult { nodes: [A, B], links: [A→B] }

// From file 2  
ExtractionResult { nodes: [C, D], links: [C→D] }
```

**Output:**
```rust
GraphData {
    nodes: [A, B, C, D],     // All nodes merged
    links: [A→B, C→D],       // All edges merged
    hyperedges: [...],        // Modules detected
    metadata: GraphMetadata {
        source_path: "./src",
        node_count: 4,
        edge_count: 2,
    }
}
```

---

## Main Functions

### 1. build_graph()
The main entry point. Orchestrates the full pipeline.

```rust
pub fn build_graph(source: &str) -> BuildSummary {
    // 1. Detect files
    let files = detect(source)?;
    
    // 2. Extract from each file
    let extractions = extract_files(&files)?;
    
    // 3. Merge into graph
    let graph = merge_into_graph(extractions)?;
    
    // 4. Add communities
    let graph = add_communities(&graph)?;
    
    // 5. Detect hyperedges
    let hyperedges = detect_hyperedges(&graph);
    
    // 6. Export
    to_json(&graph, "garfield-out/graph.json")?;
    
    BuildSummary { ... }
}
```

### 2. merge_extractions()
Combine multiple extraction results into one.

```rust
pub fn merge_extractions(results: Vec<ExtractionResult>) -> ExtractionResult {
    // Collect all nodes, dedupe by ID
    // Collect all edges, dedupe by source+target+relation
}
```

### 3. merge_into_graph()
Build GraphData from ExtractionResult.

```rust
pub fn merge_into_graph(extraction: ExtractionResult) -> GraphData {
    GraphData {
        nodes: extraction.nodes,
        links: extraction.links,
        hyperedges: vec![],
        metadata: GraphMetadata { ... },
    }
}
```

### 4. dedup_edges()
Remove duplicate edges.

```rust
// Before: [A→B, A→B, A→B]
// After:  [A→B]
```

---

## Community Detection

After merging, Leiden algorithm groups related nodes into communities.

```rust
pub fn add_communities(graph: &GraphData) -> GraphData {
    // Run Leiden algorithm
    // Assign community ID to each node
}
```

**Why communities?**
- Nodes in same community are likely related
- Helps with query ranking
- Used for graph visualization

---

## Code Location

```
src/
├── build.rs          ← THIS FILE
│   ├── build_graph()         ← Main entry
│   ├── merge_extractions()   ← Combine extractions
│   ├── merge_into_graph()    ← Build GraphData
│   ├── dedup_edges()         ← Remove duplicates
│   └── add_communities()    ← Community detection
│
├── community.rs      ← Leiden algorithm
│
└── types.rs
    └── GraphData struct
```

---

## See Also

- [Flow: build.md](../flow/build.md) - The full build flow
- [Modules: hyperedge.md](hyperedge.md) - What happens after
- [Modules: community.rs](../flow/community.md) - Leiden algorithm
