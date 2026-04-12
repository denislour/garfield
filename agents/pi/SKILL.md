---
name: garfield
description: Garfield knowledge graph builder for code architecture. Use when asked about architecture, code relationships, god nodes, or community structure.
trigger: /gf
---

# /gf - Garfield Knowledge Graph

Garfield is a fast Rust-based code knowledge graph builder. It extracts code structure using tree-sitter (248+ languages) and provides BFS/DFS query, shortest path, and community detection.

## When to Use Garfield

- User asks about "architecture", "code structure", "how does X work"
- User asks about "god nodes", "key classes", "core abstractions"
- User asks about "what connects A to B"
- Starting a new session to understand unfamiliar code
- Before searching raw files for structural questions

## Workflow - Follow These Steps

### Step 1: Check if Graph Exists

Before doing anything, check if the knowledge graph already exists:

```bash
# Check for existing graph
if [ -f "garfield-out/graph.json" ]; then
    echo "Graph found at garfield-out/graph.json"
    # Read metadata to show stats
    if command -v garfield &> /dev/null; then
        garfield stats --graph garfield-out/graph.json 2>/dev/null || echo "Run /gf report to see graph stats"
    fi
else
    echo "No graph found - need to build first"
fi
```

### Step 2: Build Graph (If Needed)

If no graph exists, or if user asks to rebuild:

```bash
# Ensure garfield-out directory exists
mkdir -p garfield-out

# Find garfield binary (try multiple locations)
GARFIELD_BIN=""
for bin in "garfield" "./target/release/garfield" "/home/jake/Compa/garfield/target/release/garfield"; do
    if command -v "$bin" &> /dev/null || [ -f "$bin" ]; then
        GARFIELD_BIN="$bin"
        break
    fi
done

if [ -z "$GARFIELD_BIN" ]; then
    echo "ERROR: Garfield binary not found"
    echo "Please build with: cd /home/jake/Compa/garfield && cargo build --release"
    exit 1
fi

# Determine path (default to current directory)
BUILD_PATH="${1:-.}"

# Run build
echo "Building graph from: $BUILD_PATH"
"$GARFIELD_BIN" build "$BUILD_PATH" --output garfield-out

# Show result
if [ -f "garfield-out/graph.json" ]; then
    echo ""
    echo "✅ Graph built successfully"
    cat garfield-out/GRAPH_REPORT.md 2>/dev/null | head -50 || echo "Run /gf report for full details"
else
    echo "❌ Build failed - check errors above"
fi
```

### Step 3: Present Initial Findings

After building, show the user:

1. **Graph Stats** - nodes, edges, communities
2. **God Nodes** - most connected concepts
3. **Community Structure** - what major sections exist

```bash
# Get graph stats
garfield stats --graph garfield-out/graph.json

# Show top communities
garfield report --graph garfield-out/graph.json 2>/dev/null | head -80
```

### Step 4: Answer User Questions

Use these commands based on what user asks:

#### Query - BFS (broad context)
```bash
garfield query "what does SERVICE_NAME connect to?" --graph garfield-out/graph.json
```

#### Query - DFS (specific path)
```bash
garfield query "how does A reach B" --dfs --graph garfield-out/graph.json
```

#### Find Path
```bash
garfield path "NodeA" "NodeB" --graph garfield-out/graph.json
```

#### Explain Node
```bash
garfield explain "NodeName" --graph garfield-out/graph.json
```

#### Show Report
```bash
cat garfield-out/GRAPH_REPORT.md
```

### Step 5: After Code Changes

When user modifies code and wants to update:

```bash
# Incremental update (fast, only changed files)
garfield build . --update --output garfield-out

# Full rebuild (if needed)
rm -rf garfield-out/
garfield build . --output garfield-out
```

---

## Garfield vs Graphify

| Feature | Graphify | Garfield |
|---------|----------|----------|
| Languages | 248+ | 248+ |
| Community detection | ✓ | ✓ |
| BFS/DFS query | ✓ | ✓ |
| Incremental cache | ✓ | ✓ |
| LLM integration | ✓ | ✗ (code-only) |
| Semantic extraction | ✓ | ✗ |
| Video/Audio | ✓ | ✗ |
| HTML viz | ✓ | ✗ |

Garfield is **code-only extraction** - no LLM, no semantic extraction, no video/audio support.

---

## Example Conversations

### User: "Understand this codebase"
```
→ Step 1: Check graph exists
→ Step 2: Build if needed
→ Step 3: Show stats + god nodes + communities
→ Step 4: Ask "What would you like to explore?"
```

### User: "How does authentication work?"
```
→ garfield query "authentication" --graph garfield-out/graph.json
→ Present connections and paths
→ Offer to trace specific chains
```

### User: "Trace from UserService to Database"
```
→ garfield path "UserService" "Database" --graph garfield-out/graph.json
→ Explain each hop in plain language
→ Show source locations for reference
```

---

## Garfield Binary Locations

The agent tries these paths in order:
1. `garfield` (if in PATH)
2. `./target/release/garfield`
3. `/home/jake/Compa/garfield/target/release/garfield`

Build if not found:
```bash
cd /home/jake/Compa/garfield && cargo build --release
```

## Output Files

- `garfield-out/graph.json` - Knowledge graph (JSON)
- `garfield-out/GRAPH_REPORT.md` - Human-readable report
- `garfield-out/cache/` - Incremental build cache