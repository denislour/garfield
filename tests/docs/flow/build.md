# BUILD Flow

## What Happens When You Run `cargo run -- build ./examples/go`

This document explains **step by step** how source code becomes a knowledge graph.

---

## The Big Picture

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│ Source Code │───▶│   Detect    │───▶│   Extract   │───▶│   Build     │
│  (.go file) │    │  (find files)│    │ (parse AST) │    │  (merge)    │
└─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘
                                                                │
                                                                ▼
                                                       ┌─────────────┐
                                                       │  Hyperedge  │
                                                       │  (modules)  │
                                                       └─────────────┘
                                                                │
                                                                ▼
                                                       ┌─────────────┐
                                                       │ graph.json  │
                                                       │ (the result)│
                                                       └─────────────┘
```

---

## Step 1: DETECT - Find Files

**What:** Scan the directory and find files by extension.

**Input:** Directory path `./examples/go`

**Output:** List of files with their extensions

**Example:**
```
./examples/go/user_store.go   → extension: .go
./examples/go/main.go         → extension: .go
./examples/README.md          → extension: .md (skipped)
```

**Behind the scenes:**
- Read directory recursively
- Filter by known extensions (.go, .rs, .py, .rb, etc.)
- Skip hidden files, noise directories (.git, node_modules, etc.)
- Compute file hashes for caching

---

## Step 2: EXTRACT - Parse Code

**What:** Use tree-sitter to parse each file and extract definitions and relationships.

**Input:** One source file

**Output:** 
- `nodes[]` - Every function, class, method, struct found
- `edges[]` - Relationships like "A calls B"

**Example - Input:**
```go
// user_store.go
type UserStore struct {
    users map[uint64]User
}

func (s *UserStore) Create(name, email string) uint64 {
    id := s.nextID
    s.nextID++
    user := User{Name: name, Email: email}
    s.users[id] = user
    return id
}
```

**Example - Output:**
```
NODES:
  • UserStore (struct)
  • User (struct)
  • Create (method)
  • Find (method)
  • Delete (method)
  ...

EDGES:
  • Create → s.nextID++  (calls)
  • Create → User{}      (creates)
  • Create → s.users     (accesses)
```

**Behind the scenes:**
- tree-sitter parses the AST (Abstract Syntax Tree)
- Find nodes matching language-specific patterns
- For Go: `function_declaration`, `method_declaration`, `type_declaration`
- For Ruby: `class`, `method`
- For Python: `class_definition`, `function_definition`

---

## Step 3: BUILD - Create Graph

**What:** Merge all extractions into one graph, deduplicate, add metadata.

**Input:** Multiple `ExtractionResult` (one per file)

**Output:** Single `GraphData`

**What happens:**
1. **Merge nodes** - Combine all nodes from all files, remove duplicates
2. **Merge edges** - Combine all edges, deduplicate
3. **Add communities** - Use Leiden algorithm to group related nodes

**Example:**
```
File 1 (user_store.go): 14 nodes, 5 edges
File 2 (main.go): 3 nodes, 2 edges

Merged: 17 nodes, 7 edges
```

---

## Step 4: HYPEREDGE - Find Modules

**What:** Group nodes that belong to the same file into a "module" (hyperedge).

**Logic:**
- If a file has 3+ definitions, create a hyperedge
- The hyperedge represents that these nodes work together

**Example:**
```
File: user_store.go
  - UserStore (struct)
  - Create (method)
  - Find (method)
  - Update (method)
  - Delete (method)
  - List (method)
  - Count (method)
  - NewUserStore (function)
  ... (14 total)

HYPEREDGE CREATED:
  id: "file_user_store"
  label: "user_store module"
  nodes: [UserStore, Create, Find, Update, Delete, List, Count, NewUserStore, ...]
  confidence: 1.0
```

---

## Final Output: graph.json

**What:** The complete knowledge graph in JSON format.

**Location:** `garfield-out/graph.json`

**Structure:**
```json
{
  "nodes": [
    {
      "id": "user_store:Create",
      "label": "Create",
      "source_file": "./examples/go/user_store.go",
      "source_location": "L32",
      "node_type": "method"
    },
    ...
  ],
  "links": [
    {
      "source": "user_store:Create",
      "target": "user_store:User",
      "relation": "creates",
      "confidence": "Extracted"
    },
    ...
  ],
  "hyperedges": [
    {
      "id": "file_user_store",
      "label": "user_store module",
      "nodes": ["user_store:Create", "user_store:Find", ...],
      "relation": "participate_in",
      "confidence_score": 1.0
    }
  ]
}
```

---

## Run It Yourself

```bash
# Build the Go example
cargo run -- build ./examples/go

# Look at the result
cat garfield-out/graph.json | jq '.nodes | length'   # How many nodes
cat garfield-out/graph.json | jq '.hyperedges'     # The modules
```

**Expected output:**
```
✅ Build complete!
  Nodes: 14
  Edges: 5
  Communities: 11
  Hyperedges: 1
```

---

## See Also

- [Modules: extract.md](../modules/extract.md) - How extraction works
- [Modules: hyperedge.md](../modules/hyperedge.md) - How modules are detected
- [Examples: go.md](../examples/go.md) - Real example with all output
