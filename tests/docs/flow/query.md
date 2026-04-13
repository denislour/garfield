# QUERY Flow

## What Happens When You Run `cargo run -- query "store"`

This document explains how searching works and what you get back.

---

## The Big Picture

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│ graph.json  │───▶│    Score    │───▶│  Traverse   │───▶│   Format    │
│             │    │  (ranking)  │    │  (BFS/DFS)  │    │  (output)   │
└─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘
                                                                │
                                                                ▼
                                                       ┌─────────────────┐
                                                       │  User sees:     │
                                                       │  Nodes with      │
                                                       │  [module] tag    │
                                                       └─────────────────┘
```

---

## Step 1: SCORE - Rank by Relevance

**What:** Give each node a score based on how well it matches the search term.

**Input:** Search term (e.g., "store")

**Output:** List of `(score, node_id)` sorted by score

**Scoring rules:**
- Label match = 1.0 point per term
- File match = 0.5 point per term

**Example:**
```
Search term: "store"

Nodes scored:
  • "UserStore"        → 1.0 (label matches "store")
  • "StoreManager"     → 1.0 (label matches "store")
  • "user_store.go"    → 0.5 (file matches "store")
  • "update_user"      → 0.0 (no match)
  • "delete_user"      → 0.0 (no match)

Top 3 results: UserStore, StoreManager, user_store.go
```

---

## Step 2: TRAVERSE - Explore the Graph

**What:** Start from the top matches and explore outward.

**Input:** Top 3 scored nodes

**Output:** Set of nodes and edges within N hops

**Two modes:**

| Mode | Flag | Behavior |
|------|------|----------|
| BFS | (default) | Explore all neighbors first |
| DFS | `--dfs` | Go deep down one path |

**Example (BFS, depth=3):**
```
Start: UserStore

Depth 0: UserStore
Depth 1: Create, Find, Update, Delete (methods of UserStore)
Depth 2: User, errors.New (things Create calls)
Depth 3: Map, etc. (things those call)
```

---

## Step 3: FORMAT - Create Output

**What:** Turn the found nodes/edges into readable text.

**Special feature:** Each node gets annotated with its **hyperedge (module)** if it belongs to one.

**Input:** Nodes + Edges + Hyperedges

**Output:**
```
## Nodes
  • UserStore [user_store module] [./examples/go/user_store.go @ L16]
  • Create [user_store module] [./examples/go/user_store.go @ L32]
  • Find [user_store module] [./examples/go/user_store.go @ L38]
```

Notice the `[user_store module]` tag - this is the hyperedge!

---

## How Hyperedge Annotation Works

**The logic:**
```
For each node in results:
    Find hyperedge that contains this node
    If found: add "[hyperedge.label]" to output
    If not: output node without tag
```

**Example:**
```
Hyperedges:
  - { label: "user_store module", nodes: [Create, Find, Update, ...] }

Query "store":
  • Create → belongs to "user_store module" → "Create [user_store module]"
  • Find   → belongs to "user_store module" → "Find [user_store module]"
  • delete_user → NO hyperedge → "delete_user" (no tag)
```

---

## Run It Yourself

```bash
# First build (creates graph.json)
cargo run -- build ./examples/go

# Query for "store"
cargo run -- query "store"

# Query with DFS
cargo run -- query "store" --dfs

# Query with deeper traversal
cargo run -- query "store" --depth 5
```

**Expected output:**
```
Query: "store"
Mode: BFS (depth=3, budget=2000)

## Nodes
  • global [user_store module] [./examples/go/user_store.go @ L?]
  • NewUserStore [user_store module] [./examples/go/user_store.go @ L24]
  • Create [user_store module] [./examples/go/user_store.go @ L32]
  • Find [user_store module] [./examples/go/user_store.go @ L38]
  • Count [user_store module] [./examples/go/user_store.go @ L95]
```

---

## What Each Part Means

| Part | Meaning |
|------|---------|
| `• Create` | Node label (function/method name) |
| `[user_store module]` | Hyperedge - this node belongs to the user_store module |
| `[./examples/go/user_store.go @ L32]` | File location |
| `(community: 4)` | Community ID from Leiden algorithm |

---

## The `[module]` Tag is Key

This is what makes hyperedges useful in query:

- Without hyperedge: `• Create [./file.go @ L32]`
- With hyperedge: `• Create [user_store module] [./file.go @ L32]`

The `[user_store module]` tells you **which group this function belongs to** without having to look it up.

---

## See Also

- [Flow: build.md](build.md) - How the graph is created
- [Flow: explain.md](explain.md) - Get detailed info about one node
- [Modules: serve.md](../modules/serve.md) - How query is implemented
- [Modules: hyperedge.md](../modules/hyperedge.md) - How modules work
