# Garfield (gf) - Rust Knowledge Graph Builder

A fast, deterministic knowledge graph builder for source code. Port of [graphify](https://github.com/your-org/graphify) from Python to Rust for better performance.

## Installation

### Option 1: Build and run locally
```bash
cargo build --release
./target/release/gf build ./src
# or
./target/release/garfield build ./src
```

### Option 2: Install globally
```bash
cargo install --path .
gf --help  # Available globally
```

### Option 3: Agent integrations
```bash
# PI agent
gf agent pi

# Claude Code / Cursor
gf agent claude
gf agent cursor
```

## Quick Start

```bash
# Build
cargo build --release

# Run
./target/release/gf build ./src
# or
./target/release/garfield build ./src
```

## CLI Commands

```bash
# Build knowledge graph
gf build <path>          # Full build
gf build <path> --update # Incremental build

# Query
gf query "function_name" # BFS traversal (default)
gf query "X" --dfs       # DFS traversal

# Find paths
gf path "A" "B"          # Shortest path A -> B

# Explain
gf explain "NodeName"     # Node details
```

## Architecture

```
garfield/src/
├── types.rs        # Core data structures (Node, Edge, GraphData)
├── detect.rs       # File detection by extension
├── extract.rs      # tree-sitter AST extraction
├── build.rs        # Graph building + deduplication
├── cluster.rs      # Label propagation community detection
├── analyze.rs      # God nodes, surprising connections
├── serve.rs        # BFS/DFS query engine
├── report.rs       # GRAPH_REPORT.md generation
├── cache.rs        # SHA256 caching for incremental builds
├── export.rs       # JSON serialization
├── validate.rs     # Schema validation
├── lib.rs          # Library interface
└── main.rs         # CLI with clap
```

## Output

- `graphify-out/graph.json` - Knowledge graph in JSON format
- `graphify-out/GRAPH_REPORT.md` - Human-readable report

### graph.json Structure

```json
{
  "nodes": [
    {
      "id": "module:function_name",
      "label": "function_name",
      "source_file": "path/to/file.py",
      "source_location": "L42",
      "community": 5
    }
  ],
  "edges": [
    {
      "source": "a.py:foo",
      "target": "a.py:bar",
      "relation": "calls",
      "confidence": "EXTRACTED"
    }
  ],
  "metadata": {
    "total_nodes": 100,
    "total_edges": 250,
    "communities": 15
  }
}
```

## Library Usage

```rust
use garfield::{run_build, run_query, run_path};

fn main() {
    // Build graph
    let summary = run_build("./src", "graphify-out", false).unwrap();
    println!("Built {} nodes, {} edges", summary.total_nodes, summary.total_edges);
    
    // Query
    let result = run_query("function_name", false, 3, 2000).unwrap();
    println!("{}", result);
    
    // Path
    if let Some(path) = run_path("A", "B", 10).unwrap() {
        println!("Path: {}", path.join(" -> "));
    }
}
```

## Comparison with Python graphify

| Feature | Python | Rust |
|---------|--------|------|
| tree-sitter | ✓ | ✓ |
| Community detection | ✓ | ✓ |
| BFS/DFS query | ✓ | ✓ |
| Incremental cache | ✓ | ✓ |
| MCP server | ✓ | CLI only |
| LLM integration | ✓ | ✗ |

Garfield is **code-only extraction** - no LLM, MCP, video/audio, or Neo4j.

## Testing

```bash
# Unit tests
cargo test

# Integration tests
cargo test --test integration_test

# With output
cargo test -- --nocapture
```

## License

MIT
