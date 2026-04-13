# Module: serve.rs

## Purpose

**Query the graph** - search, explain, find paths. This is what you use after building.

---

## What It Does

Given a `GraphData`:
1. `query()` - Search nodes by keyword
2. `get_node()` - Get detailed info about one node
3. `find_shortest_path()` - Find path between nodes
4. `score_nodes()` - Rank nodes by relevance

---

## Main Functions

### query()

Search and traverse the graph.

```rust
pub fn query(
    graph: &GraphData,
    question: &str,      // "store"
    use_dfs: bool,       // DFS or BFS
    depth: usize,         // How deep to go
    token_budget: usize,  // Output limit
) -> String
```

**Process:**
1. Split question into terms
2. `score_nodes()` - rank by match
3. BFS or DFS traversal
4. Format with `subgraph_to_text()`

**Output:**
```
## Nodes
  • Create [user_store module] [./file.go @ L32]
  • Find [user_store module] [./file.go @ L38]
```

### get_node()

Get detailed info for one node.

```rust
pub fn get_node(graph: &GraphData, identifier: &str) -> Option<NodeDetails>
```

**Returns:**
```rust
struct NodeDetails {
    id: String,
    label: String,
    source_file: String,
    source_location: String,
    community: Option<u32>,
    incoming_edges: Vec<EdgeInfo>,   // Who calls this?
    outgoing_edges: Vec<EdgeInfo>,   // What does this call?
    hyperedge: Option<HyperedgeInfo>, // What module?
}
```

**Output:**
```
═══ NODE ═══
ID: user_store:Create
Label: Create

═══ MODULE (Hyperedge) ═══
Module: user_store module
Members: 14 functions

═══ CALLED BY ═══
  ← NewUserStore (calls)
```

### find_shortest_path()

Find connection between two nodes.

```rust
pub fn find_shortest_path(
    graph: &GraphData,
    source: &str,
    target: &str,
    max_hops: usize,
) -> Option<Vec<String>>  // ["A", "B", "C", "D"]
```

**Example:**
```bash
cargo run -- path "main" "Create"
```

### score_nodes()

Rank nodes by search relevance.

```rust
pub fn score_nodes(graph: &GraphData, terms: &[String]) -> Vec<(f64, &str)>
```

**Scoring:**
- Label match: 1.0 per term
- File match: 0.5 per term

---

## Key Feature: Hyperedge Annotation

When rendering nodes in query output, each node gets annotated with its hyperedge:

```rust
// subgraph_to_text() - this is the key function

for node in nodes {
    // Find hyperedge containing this node
    let hyperedge_label = graph.hyperedges.iter()
        .find(|he| he.nodes.contains(&node.id))
        .map(|he| format!(" [{}]", he.label))
        .unwrap_or_default();
    
    // Output: "  • Create [user_store module] [./file.go @ L32]"
    lines.push(format!("  • {}{} [{} @ {}]",
        node.label,
        hyperedge_label,  // ← Added here
        node.source_file,
        node.source_location
    ));
}
```

---

## Code Location

```
src/
├── serve.rs         ← THIS FILE
│   ├── query()              ← Main search
│   ├── get_node()           ← Node details
│   ├── find_shortest_path() ← Path finding
│   ├── score_nodes()        ← Ranking
│   ├── subgraph_to_text()   ← Format output
│   ├── bfs()               ← Breadth-first search
│   └── dfs()               ← Depth-first search
│
└── types.rs
    ├── NodeDetails struct
    ├── HyperedgeInfo struct
    └── EdgeInfo struct
```

---

## Run It

```bash
# Build first
cargo run -- build ./examples/go

# Search
cargo run -- query "store"

# Get details
cargo run -- explain "Create"

# Find path
cargo run -- path "main" "Create"
```

---

## See Also

- [Flow: query.md](../flow/query.md) - How query works
- [Flow: explain.md](../flow/explain.md) - How explain works
- [Modules: hyperedge.md](hyperedge.md) - The hyperedge it annotates
