# EXPLAIN Flow

## What Happens When You Run `cargo run -- explain "Create"`

This document explains how to get detailed information about a specific node.

---

## The Big Picture

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│ graph.json  │───▶│  Find Node  │───▶│ Find Hyper  │
│             │    │  (by ID)    │    │   (edge)    │
└─────────────┘    └─────────────┘    └─────────────┘
                                             │
                                             ▼
                                    ┌─────────────────┐
                                    │  OUTPUT:        │
                                    │  Node + Module  │
                                    └─────────────────┘
```

---

## Step 1: FIND NODE - Locate the Node

**What:** Search for a node by ID or label (partial match).

**Input:** Node identifier (e.g., "Create", "user_store:Create")

**Output:** The matching node

**Matching rules:**
- Exact ID match: "user_store:Create"
- Partial ID match: "Create" (finds first with "Create" in ID)
- Label match: "Create" (finds first with "Create" in label)

**Example:**
```
Search: "Create"

Found:
  ID: "user_store:Create"
  Label: "Create"
  File: "./examples/go/user_store.go"
  Location: "L32"
```

---

## Step 2: FIND CONNECTIONS - Get Edges

**What:** Find all edges connected to this node.

**Input:** Node ID

**Output:** Incoming edges (called by) + Outgoing edges (calls)

**Example:**
```
For node "Create":

Incoming edges (who calls Create?):
  ← NewUserStore (calls Create as part of initialization)
  ← global

Outgoing edges (what does Create call?):
  → User{} (creates a User struct)
  → s.users (accesses field)
```

---

## Step 3: FIND HYPEREDGE - Get Module Info

**What:** Find which hyperedge (module) this node belongs to.

**Input:** Node ID

**Output:** Hyperedge info if found

**Logic:**
```python
For each hyperedge:
    If node.id is in hyperedge.nodes:
        return hyperedge
```

**Example:**
```
Hyperedges:
  - { 
      id: "file_user_store",
      label: "user_store module",
      nodes: ["NewUserStore", "Create", "Find", "Update", ...],
      member_count: 14
    }

For node "Create":
  → Found in hyperedge "user_store module"
  → Return hyperedge info
```

---

## Final Output

**What you see:**

```
═══ NODE ═══
ID: user_store:Create
Label: Create
File: ./examples/go/user_store.go
Location: L32

═══ MODULE (Hyperedge) ═══
Module: user_store module
Members: 14 functions
Confidence: 1.00

═══ CALLED BY ═══
  ← NewUserStore (calls)
  ← global (calls)

═══ CALLS ═══
  → User{} (creates)
```

---

## The MODULE Section is New

The **hyperedge** appears as a dedicated section:

```
═══ MODULE (Hyperedge) ═══
Module: user_store module      ← What module this belongs to
Members: 14 functions         ← How many things in this module
Confidence: 1.00               ← Detection confidence
```

This tells you:
- **Module name**: "user_store module"
- **Size**: 14 functions in the same file
- **Confidence**: How sure we are (1.0 = file-based detection)

---

## Why This Matters

Without hyperedge:
```
═══ NODE ═══
ID: user_store:Create
Label: Create
File: ./examples/go/user_store.go
Location: L32

═══ CALLED BY ═══
  ← NewUserStore
```

With hyperedge:
```
═══ NODE ═══
ID: user_store:Create
Label: Create

═══ MODULE (Hyperedge) ═══      ← NEW!
Module: user_store module        ← You now know this is part of user_store
Members: 14 functions           ← There are 14 other things in the same file

═══ CALLED BY ═══
  ← NewUserStore
```

The hyperedge tells you **context** - what this function belongs to.

---

## Run It Yourself

```bash
# First build
cargo run -- build ./examples/go

# Explain a function
cargo run -- explain "Create"

# Explain a struct
cargo run -- explain "UserStore"

# Explain something not in a module (if exists)
cargo run -- explain "main"
```

**Expected output:**
```
═══ NODE ═══
ID: user_store:Create
Label: Create
File: ./examples/go/user_store.go
Location: L32

═══ MODULE (Hyperedge) ═══
Module: user_store module
Members: 14 functions
Confidence: 1.00

═══ CALLED BY ═══
  ← global (calls)
```

---

## What Each Part Means

| Section | What It Shows |
|---------|---------------|
| `═══ NODE ═══` | Basic node info |
| `ID` | Full qualified name (module:name) |
| `Label` | Just the name |
| `File` | Source file path |
| `Location` | Line number |
| `═══ MODULE ═══` | **Hyperedge info** - which module this belongs to |
| `Members` | How many items in the same module |
| `═══ CALLED BY ═══` | Incoming edges - who uses this |
| `═══ CALLS ═══` | Outgoing edges - what this uses |

---

## Node Without Hyperedge

If a node doesn't belong to any hyperedge, the MODULE section is omitted:

```
═══ NODE ═══
ID: main
Label: main
File: ./examples/go/main.go
Location: L1

═══ CALLED BY ═══
  ← global
```

This happens for:
- Standalone functions in their own files
- Entry points (main, init)
- Global utilities

---

## See Also

- [Flow: build.md](build.md) - How the graph is created
- [Flow: query.md](query.md) - How to search for nodes
- [Modules: serve.md](../modules/serve.md) - How explain is implemented
- [Modules: hyperedge.md](../modules/hyperedge.md) - How modules work
