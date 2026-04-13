# Garfield Documentation

This directory contains documentation written in **natural language** - no code, just explanations with real examples.

---

## How to Read This Documentation

Each section explains:
1. **What it does** - Plain English explanation
2. **The flow** - Step by step with input/output
3. **Real examples** - Actual output from real code in `examples/` folder

---

## Quick Start

```bash
# Build a project
cargo run -- build ./examples/go

# Query for something
cargo run -- query "store"

# Get details
cargo run -- explain "Create"
```

---

## Table of Contents

### Flow Documentation (`flow/`)
These explain the **complete journey** of data through the system.

| File | What It Explains |
|------|-----------------|
| [build.md](flow/build.md) | How source code becomes a graph |
| [query.md](flow/query.md) | How to search the graph |
| [explain.md](flow/explain.md) | How to get detailed node info |

### Module Documentation (`modules/`)
These explain **what each module does**.

| File | Module | Purpose |
|------|--------|---------|
| [detect.md](modules/detect.md) | `src/detect.rs` | Find files by extension |
| [extract.md](modules/extract.md) | `src/extract.rs` | Parse code into nodes/edges |
| [build.md](modules/build.md) | `src/build.rs` | Merge and create graph |
| [hyperedge.md](modules/hyperedge.md) | `src/hyperedge.rs` | Group nodes into modules |
| [serve.md](modules/serve.md) | `src/serve.rs` | Query and explain |
| [cache.md](modules/cache.md) | `src/cache.rs` | Cache extractions |

### Examples (`examples/`)
Real code examples with actual output.

| File | Language | What It Shows |
|------|----------|---------------|
| [ruby.md](examples/ruby.md) | Ruby | Class with methods |
| [go.md](examples/go.md) | Go | Struct with methods |
| [python.md](examples/python.md) | Python | Class with methods |
| [rust.md](examples/rust.md) | Rust | Struct with impl |
| [zig.md](examples/zig.md) | Zig | Struct with methods |
| [elixir.md](examples/elixir.md) | Elixir | Module with functions |

---

## The Three Main Commands

```
cargo run -- build ./src      # Step 1: Build the graph
cargo run -- query "term"     # Step 2: Search
cargo run -- explain "name"   # Step 3: Get details
```

---

## Key Concepts

### Node
A single definition in code - a function, class, method, or struct.

### Edge
A relationship between two nodes - like "A calls B".

### Hyperedge (Module)
A group of nodes that work together in the same file. Example: All methods in `user_store.go` form one hyperedge called "user_store module".

---

## File Locations

```
garfield/
├── src/
│   ├── detect.rs      # Find files
│   ├── extract.rs     # Parse code
│   ├── build.rs       # Build graph
│   ├── hyperedge.rs   # Find modules
│   └── serve.rs       # Query/Explain
│
├── examples/          # Example source files
│   ├── ruby/user_service.rb
│   ├── go/user_store.go
│   └── ...
│
└── tests/
    └── docs/          # ← YOU ARE HERE
        ├── flow/       # Flow explanations
        ├── modules/    # Module explanations
        └── examples/   # Language examples
```
