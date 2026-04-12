---
name: garfield
description: Garfield knowledge graph for code architecture. Use when asked about architecture, code relationships, god nodes, or community structure.
trigger: /gf
---

# /gf - Garfield Knowledge Graph

Garfield is a fast Rust-based code knowledge graph builder. It extracts code structure using tree-sitter (248+ languages) and provides BFS/DFS query, shortest path, and community detection.

## What Garfield is for

- Understanding codebase architecture before touching code
- Finding what connects A to B
- Identifying god nodes and key abstractions
- Starting a new session to understand unfamiliar code

## Rules

### Before Searching Files

When answering architecture questions or exploring codebase structure:

1. **First**: Check if `graphify-out/graph.json` exists
2. **If not**: Run `garfield build <path>` to build the graph
3. **Then**: Use `/gf query` to understand connections

### When to Use Garfield

- User asks about "architecture", "code structure", "how does X work"
- User asks about "god nodes", "key classes", "core abstractions"
- User asks about "what connects A to B"
- Starting a new session to understand a codebase
- Before searching raw files for structural questions

### When to Search Directly

- User asks for specific code content or implementation details
- After understanding structure via Garfield, finding actual code
- Simple file lookups that don't need context

## Workflow

```
User: "Giới thiệu về codebase"
→ garfield build . (if graph doesn't exist)
→ /gf report (show GRAPH_REPORT.md)
→ Understand god nodes and structure
→ Answer based on graph data

User: "Logic của create order là gì?"
→ /gf query "what does CreateOrderService connect to?"
→ /gf path "CreateOrderService" "InsertOrderDetailService"
→ Then search specific files for implementation
```

---

## /gf build - Build Knowledge Graph

Build graph if it doesn't exist or needs updating.

```bash
# Detect the garfield binary
GARFIELD_BIN=$(which garfield 2>/dev/null || which garf 2>/dev/null)
if [ -z "$GARFIELD_BIN" ]; then
    # Try to find in project directory
    GARFIELD_BIN="./target/release/garfield"
    if [ ! -f "$GARFIELD_BIN" ]; then
        echo "ERROR: Garfield not found. Build with: cargo build --release"
        exit 1
    fi
fi

# Ensure graphify-out directory exists
mkdir -p graphify-out

"$GARFIELD_BIN" build PATH
```

Replace PATH with the actual path. If no path given, use `.` (current directory).

After build, present the summary:
```
✅ Graph built successfully
  Nodes: N
  Edges: M
  Communities: C
```

---

## /gf query - Query the Graph

Query the knowledge graph using BFS or DFS traversal.

```bash
GARFIELD_BIN=$(which garfield 2>/dev/null || which garf 2>/dev/null || ./target/release/garfield)
GRAPH_PATH="${GRAPH_PATH:-graphify-out/graph.json}"

# Check if graph exists
if [ ! -f "$GRAPH_PATH" ]; then
    echo "ERROR: No graph found. Run 'garfield build .' first."
    exit 1
fi

QUESTION="QUESTION_TEXT"
MODE="bfs"  # or 'dfs'
DEPTH=3
BUDGET=2000

"$GARFIELD_BIN" query "$QUESTION" --$MODE --depth $DEPTH --budget $BUDGET --graph "$GRAPH_PATH"
```

Two traversal modes:

| Mode | Flag | Best for |
|------|------|----------|
| BFS (default) | _(none)_ | "What is X connected to?" - broad context, nearest neighbors first |
| DFS | `--dfs` | "How does X reach Y?" - trace a specific path |

After query, present results. Answer using **only** what the graph contains. Quote `source_location` when citing a specific fact.

---

## /gf path - Find Shortest Path

Find the shortest path between two named concepts in the graph.

```bash
GARFIELD_BIN=$(which garfield 2>/dev/null || which garf 2>/dev/null || ./target/release/garfield)
GRAPH_PATH="${GRAPH_PATH:-graphify-out/graph.json}"

# Check if graph exists
if [ ! -f "$GRAPH_PATH" ]; then
    echo "ERROR: No graph found. Run 'garfield build .' first."
    exit 1
fi

SOURCE_NODE="NodeA"
TARGET_NODE="NodeB"
MAX_HOPS=8

"$GARFIELD_BIN" path "$SOURCE_NODE" "$TARGET_NODE" --max-hops $MAX_HOPS --graph "$GRAPH_PATH"
```

After path, explain in plain language - what each hop means, why it's significant.

---

## /gf explain - Explain a Node

Give a plain-language explanation of a single node - everything connected to it.

```bash
GARFIELD_BIN=$(which garfield 2>/dev/null || which garf 2>/dev/null || ./target/release/garfield)
GRAPH_PATH="${GRAPH_PATH:-graphify-out/graph.json}"

# Check if graph exists
if [ ! -f "$GRAPH_PATH" ]; then
    echo "ERROR: No graph found. Run 'garfield build .' first."
    exit 1
fi

NODE_NAME="NodeName"

"$GARFIELD_BIN" explain "$NODE_NAME" --graph "$GRAPH_PATH"
```

Then write a 3-5 sentence explanation of what this node is, what it connects to, and why those connections are significant.

---

## /gf report - Show Graph Report

Show the human-readable GRAPH_REPORT.md.

```bash
if [ -f "graphify-out/GRAPH_REPORT.md" ]; then
    cat graphify-out/GRAPH_REPORT.md
else
    echo "No report found. Run 'garfield build .' first."
fi
```

---

## After Code Changes

To keep the knowledge graph current:

```bash
GARFIELD_BIN=$(which garfield 2>/dev/null || which garf 2>/dev/null || ./target/release/garfield)

# Incremental update (fast, only changed files)
"$GARFIELD_BIN" build . --update

# Or rebuild from scratch
rm -rf graphify-out/
"$GARFIELD_BIN" build .
```

---

## Output Files

- `graphify-out/graph.json` - Knowledge graph in JSON format
- `graphify-out/GRAPH_REPORT.md` - Human-readable report

---

## Garfield vs Graphify

| Feature | Graphify | Garfield |
|---------|----------|----------|
| Languages | 248+ | 248+ |
| Community detection | ✓ | ✓ |
| BFS/DFS query | ✓ | ✓ |
| Incremental cache | ✓ | ✓ |
| LLM integration | ✓ | ✗ |
| Video/Audio | ✓ | ✗ |
| MCP server | ✓ | Via garfield binary |

Garfield is **code-only extraction** - no LLM, MCP, video/audio, or Neo4j.
