---
name: garfield
description: Query Garfield knowledge graph for code architecture. Use when asked about architecture, god nodes, code relationships, or community structure in Garfield projects.
---

# Garfield Knowledge Graph

Garfield is a Rust-based knowledge graph builder for source code. It extracts code structure using tree-sitter (248+ languages).

## Rules

### Before Searching Files

When working with Garfield projects:

1. **First**: Check if `graphify-out/graph.json` exists
2. **If not**: Run `gf build <path>` to build the graph
3. **Then**: Use Garfield tools to understand code relationships

### When to Use Garfield

- User asks about "architecture", "code structure", "how does X work"
- User asks about "what connects A to B"
- User asks about "god nodes", "key classes", "core abstractions"
- Starting a new session to understand a Garfield codebase
- Before searching raw files for structural questions

### When to Search Directly

- User asks for specific code content or implementation details
- After understanding structure via Garfield, finding actual code
- Simple file lookups that don't need context

## Garfield Tools

Use the following tools in sequence:

```
1. gf_build     - Build graph if not exists
2. gf_graph_query - Query relationships
3. gf_path      - Find path between nodes
4. gf_explain   - Explain specific node
```

## Commands

```bash
# Build knowledge graph
gf build <path>          # Full build
gf build <path> --update # Incremental build

# Query (via /gf command)
gf query "what does X connect to?"     # BFS traversal (default)
gf query "X" --dfs                     # DFS traversal

# Find paths
gf path "SourceNode" "TargetNode"     # Shortest path A -> B

# Explain
gf explain "NodeName"                  # Node details

# Show report
gf report                              # GRAPH_REPORT.md content
```

## Workflow Example

```
User: "Giới thiệu về codebase"
→ gf build . (if graph doesn't exist)
→ gf report (show GRAPH_REPORT.md)
→ Understand god nodes and structure
→ Answer based on graph data

User: "Logic của create order là gì?"
→ gf graph_query "what does CreateOrderService connect to?"
→ gf path "CreateOrderService" "InsertOrderDetailService"
→ Then search specific files for implementation
```

## Output Files

- `graphify-out/graph.json` - Knowledge graph in JSON format
- `graphify-out/GRAPH_REPORT.md` - Human-readable report

## Garfield vs Graphify

| Feature | Graphify | Garfield |
|---------|----------|----------|
| Languages | 248+ | 248+ |
| Community detection | ✓ | ✓ |
| BFS/DFS query | ✓ | ✓ |
| Incremental cache | ✓ | ✓ |
| LLM integration | ✓ | ✗ |
| MCP server | ✓ | CLI only |
| Video/Audio | ✓ | ✗ |

Garfield is **code-only extraction** - no LLM, MCP, video/audio, or Neo4j.
