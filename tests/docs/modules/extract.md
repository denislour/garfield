# Module: extract.rs

## Purpose

**Parse source code** into nodes (definitions) and edges (relationships). This is the core extraction logic.

---

## What It Does

Given a source file, it:
1. Uses tree-sitter to parse the AST (Abstract Syntax Tree)
2. Finds definitions (functions, classes, methods, structs)
3. Finds relationships (calls, imports, creates)
4. Returns structured data

---

## Input/Output

**Input:**
```
File path: "./examples/go/user_store.go"
File content: "package user\n\ntype User struct { ... }"
```

**Output:**
```rust
struct ExtractionResult {
    nodes: Vec<Node>,   // Every definition found
    links: Vec<Edge>,   // Every relationship found
}
```

---

## What Becomes a Node?

### By Language

| Language | Node Types |
|----------|------------|
| Go | `function_declaration`, `method_declaration`, `type_declaration` |
| Ruby | `class`, `method`, `module` |
| Python | `class_definition`, `function_definition` |
| Rust | `function_item`, `method`, `struct`, `impl_item` |
| JavaScript | `function_declaration`, `class_declaration` |
| TypeScript | `function_declaration`, `class_declaration`, `interface_declaration` |

### Node Structure

```rust
struct Node {
    id: String,          // "user_store:Create" (file:function)
    label: String,       // "Create"
    source_file: String, // "./examples/go/user_store.go"
    source_location: String, // "L32"
    node_type: Option<String>, // "method"
}
```

---

## What Becomes an Edge?

### Relationship Types

| Relation | Meaning | Example |
|----------|---------|---------|
| `calls` | A calls B | `Create` calls `fmt.Errorf` |
| `imports` | A imports B | `user_store` imports `errors` |
| `creates` | A creates B | `Create` creates `User` |
| `defines` | A defines B | `UserStore` defines `Create` |
| `accesses` | A accesses B | `Create` accesses `s.nextID` |

### Edge Structure

```rust
struct Edge {
    source: String,       // "user_store:Create"
    target: String,       // "errors:New"
    relation: String,    // "calls"
    confidence: Confidence, // Extracted, Inferred
}
```

---

## Real Example

### Input (Go file):
```go
package user

type User struct {
    Name  string
    Email string
}

type UserStore struct {
    users map[uint64]User
}

func (s *UserStore) Create(name, email string) (uint64, error) {
    id := s.nextID
    s.nextID++
    return id, nil
}
```

### Output (Nodes):
```
NODES:
  • User (struct)
  • UserStore (struct)
  • Create (method)
```

### Output (Edges):
```
EDGES:
  • Create → s.nextID (accesses field)
  • UserStore → User (contains)
```

---

## How tree-sitter Works

tree-sitter parses code into an **AST** (Abstract Syntax Tree):

```
source_code
└── package_clause
│   └── identifier "user"
└── type_declaration
│   └── struct_type
│       └── field_declaration "User"
└── type_declaration
│   └── struct_type
│       └── field_declaration "UserStore"
└── method_declaration
│   ├── receiver (UserStore)
│   ├── function_name "Create"
│   └── block
│       └── assignment
```

Garfield walks this tree and extracts relevant nodes.

---

## Language Configuration

Each language is configured in `src/lang.rs`:

```rust
LangConfig {
    name: "Go",
    extensions: &[".go"],
    comment_style: CommentStyle::DoubleSlash,
    import_kinds: &["import"],
    node_kinds: &["function_declaration", "method_declaration", "type_declaration"],
}
```

---

## Edge Cases

### Nested Functions
```python
def outer():
    def inner():  # inner is a node too
        pass
```
→ Both `outer` and `inner` become nodes

### Anonymous Functions
```javascript
const fn = function() { };  // Anonymous, might be skipped
```
→ Depends on language parser

### Docstrings/Comments
```python
"""This is a docstring"""
# This is a comment
```
→ Not extracted as nodes

---

## Code Location

```
src/
├── extract.rs       ← THIS FILE
│   ├── extract_file()      ← Main entry point
│   ├── extract_nodes()     ← Find definitions
│   ├── extract_edges()     ← Find relationships
│   └── infer_calls()      ← Heuristic edge detection
│
├── lang.rs          ← Language configs
│
└── types.rs
    ├── Node struct
    └── Edge struct
```

---

## Run It

```bash
# Build extracts from all files
cargo run -- build ./examples/go

# Look at extracted nodes
cat garfield-out/graph.json | jq '.nodes | length'

# See specific node
cat garfield-out/graph.json | jq '.nodes[] | select(.label == "Create")'
```

---

## See Also

- [Flow: build.md](../flow/build.md) - Where extract fits
- [Modules: detect.md](detect.md) - What comes before
- [Modules: hyperedge.md](hyperedge.md) - What happens to nodes
