# Module: hyperedge.rs

## Purpose

**Group related nodes together** into "modules" - logical units that represent things working together.

---

## The Problem It Solves

In a codebase, you have hundreds of functions. Sometimes you want to know:

> "This function `Create` - what else is related to it?"

Without hyperedges, you'd have to look at the file manually. With hyperedges:

> "`Create` is part of `user_store module` - there are 14 other things in the same file."

---

## What is a Hyperedge?

**Regular Edge:** A → B (one-to-one relationship)
```
Create ──calls──▶ User{}
```

**Hyperedge:** A group of nodes (many-to-many relationship)
```
┌─────────────────────────────────────┐
│  user_store module (hyperedge)      │
│                                     │
│  • UserStore (struct)               │
│  • Create (method)                  │
│  • Find (method)                    │
│  • Update (method)                  │
│  • Delete (method)                  │
│  • List (method)                    │
│  • Count (method)                   │
│  • NewUserStore (function)          │
│  ... (14 total)                     │
└─────────────────────────────────────┘
```

---

## How It Works

### Detection Algorithm

The simplest algorithm: **File-Based Detection**

```
1. Group all nodes by source file
2. If a file has 3+ definitions, create a hyperedge
3. The hyperedge label = filename (without extension) + " module"
```

**Example:**
```
File: user_store.go
Definitions found: 14

→ Create hyperedge:
    id: "file_user_store"
    label: "user_store module"
    nodes: [UserStore, Create, Find, Update, Delete, List, Count, ...]
```

---

## Data Structure

```rust
pub struct Hyperedge {
    pub id: String,              // "file_user_store"
    pub label: String,           // "user_store module"
    pub nodes: Vec<String>,      // ["UserStore", "Create", ...]
    pub relation: String,        // "participate_in"
    pub confidence_score: f64,   // 1.0
}
```

---

## Where It Appears

### In Query Output

```
Query: "store"

## Nodes
  • Create [user_store module] [./examples/go/user_store.go @ L32]
             ^^^^^^^^^^^^^^^^^ ← Hyperedge label
```

### In Explain Output

```
═══ NODE ═══
ID: user_store:Create

═══ MODULE (Hyperedge) ═══
Module: user_store module
Members: 14 functions
Confidence: 1.00
```

---

## Detection Methods

| Method | When Used | Confidence |
|--------|-----------|------------|
| File-Based | File has 3+ definitions | 1.0 |
| Call Chain | A→B→C→D chain | 0.7 |
| Config Pattern | K8s/Docker/Terraform | 0.8 |

### File-Based Detection

```rust
// Group nodes by file
let mut file_groups: HashMap<String, Vec<Node>> = HashMap::new();
for node in &graph.nodes {
    file_groups.entry(node.source_file.clone())
        .or_default()
        .push(node.clone());
}

// Create hyperedge if 3+ definitions
for (file, nodes) in file_groups {
    if nodes.len() >= 3 {
        hyperedges.push(Hyperedge {
            id: format!("file_{}", file_stem),
            label: format!("{} module", file_stem),
            nodes: nodes.iter().map(|n| n.id.clone()).collect(),
            relation: "participate_in".to_string(),
            confidence_score: 1.0,
        });
    }
}
```

---

## Real Example

### Source Code
```go
// user_store.go
package user

type User struct { ... }
type UserStore struct { ... }

func NewUserStore() *UserStore { ... }
func (s *UserStore) Create(name, email string) (uint64, error) { ... }
func (s *UserStore) Find(id uint64) (User, bool) { ... }
func (s *UserStore) Update(...) bool { ... }
func (s *UserStore) Delete(id uint64) bool { ... }
func (s *UserStore) List() []User { ... }
func (s *UserStore) Count() int { ... }
```

### After Build
```
Hyperedge created:
  ID: "file_user_store"
  Label: "user_store module"
  Members: 14 (structs + functions)
  Confidence: 1.0
```

### Query Result
```
## Nodes
  • User [user_store module] [./file.go @ L10]
  • UserStore [user_store module] [./file.go @ L14]
  • Create [user_store module] [./file.go @ L32]
  • Find [user_store module] [./file.go @ L38]
```

---

## Why 3+ Definitions?

**Why not 1 or 2?**

| Threshold | Result |
|-----------|--------|
| 1+ | Every file is a module (too noisy) |
| 2+ | Almost every file (still noisy) |
| **3+** | Meaningful modules only ✓ |
| 5+ | Very few modules (might miss some) |

3 is the sweet spot - files with 3+ definitions are likely to be meaningful modules, not just noise.

---

## Edge Cases

### Single Function File
```
main.go:
  func main() { ... }

→ NO hyperedge (only 1 definition)
```

### Two Function File
```
utils.go:
  func helper1() { ... }
  func helper2() { ... }

→ NO hyperedge (only 2 definitions)
```

### Three Function File
```
utils.go:
  func parse() { ... }
  func validate() { ... }
  func format() { ... }

→ CREATES hyperedge "utils module" (3 definitions)
```

---

## Code Location

```
src/
├── hyperedge.rs              ← THIS FILE
│   ├── detect_hyperedges()  ← Main function
│   ├── file_based_detection() ← Algorithm 1
│   ├── call_chain_detection()  ← Algorithm 2
│   └── config_pattern_detection() ← Algorithm 3
│
└── types.rs
    └── Hyperedge struct
```

---

## Run It

```bash
# Build creates hyperedges
cargo run -- build ./examples/go

# Look at hyperedges in JSON
cat garfield-out/graph.json | jq '.hyperedges'

# Query shows hyperedge tags
cargo run -- query "store"

# Explain shows hyperedge section
cargo run -- explain "Create"
```

---

## See Also

- [Flow: build.md](../flow/build.md) - Where hyperedges fit in the build process
- [Flow: query.md](../flow/query.md) - How hyperedges appear in query
- [Flow: explain.md](../flow/explain.md) - How hyperedges appear in explain
- [Examples: go.md](../examples/go.md) - Real example with Go code
