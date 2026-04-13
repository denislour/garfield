# Example: Rust (lib.rs)

## Source Code

**File:** `examples/rust/lib.rs`

```rust
//! User management module
//! Handles user CRUD operations

use std::collections::HashMap;

/// User data structure
#[derive(Debug, Clone)]
pub struct User {
    pub id: u64,
    pub name: String,
    pub email: String,
    pub active: bool,
}

/// User storage
pub struct UserStore {
    users: HashMap<u64, User>,
    next_id: u64,
}

impl UserStore {
    /// Create new empty store
    pub fn new() -> Self {
        UserStore {
            users: HashMap::new(),
            next_id: 1,
        }
    }

    /// Add new user
    pub fn create(&mut self, name: String, email: String) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        
        let user = User {
            id,
            name,
            email,
            active: true,
        };
        
        self.users.insert(id, user);
        id
    }

    /// Find user by ID
    pub fn find(&self, id: u64) -> Option<&User> {
        self.users.get(&id)
    }

    /// Find user by email
    pub fn find_by_email(&self, email: &str) -> Option<&User> {
        self.users.values().find(|u| u.email == email)
    }

    /// Update user
    pub fn update(&mut self, id: u64, name: String, email: String) -> bool {
        if let Some(user) = self.users.get_mut(&id) {
            user.name = name;
            user.email = email;
            true
        } else {
            false
        }
    }

    /// Delete user
    pub fn delete(&mut self, id: u64) -> bool {
        self.users.remove(&id).is_some()
    }

    /// List all users
    pub fn list(&self) -> Vec<&User> {
        self.users.values().collect()
    }

    /// Count users
    pub fn count(&self) -> usize {
        self.users.len()
    }
}

impl Default for UserStore {
    fn default() -> Self {
        Self::new()
    }
}
```

---

## Step 1: BUILD

```bash
cargo run -- build ./examples/rust
```

**Output:**
```
✅ Build complete!
  Nodes: 17
  Edges: 0
  Communities: 17
  Hyperedges: 1
```

### What was extracted:

**Nodes (17 total):**
| Label | Type | Location |
|-------|------|----------|
| User | struct | L8 |
| UserStore | struct | L16 |
| new | function | L19 |
| create | method | L26 |
| find | method | L37 |
| find_by_email | method | L41 |
| update | method | L47 |
| delete | method | L55 |
| list | method | L59 |
| count | method | L65 |
| default | method | L69 |
| Debug | trait | - |
| Clone | trait | - |
| HashMap | struct | - |
| ... and more | | |

**Hyperedges (1 total):**
```
id: "file_lib"
label: "lib module"
members: 17
confidence: 0.5
```

---

## Step 2: QUERY

```bash
cargo run -- query "store"
```

**Output:**
```
Query: "store"
Mode: BFS (depth=3, budget=2000)

## Nodes
  • UserStore [lib module] [./examples/rust/lib.rs @ L16] (community: 2)
```

---

## Step 3: EXPLAIN

```bash
cargo run -- explain "UserStore"
```

**Output:**
```
═══ NODE ═══
ID: lib:UserStore
Label: UserStore
File: ./examples/rust/lib.rs
Location: L16

═══ MODULE (Hyperedge) ═══
Module: lib module
Members: 17 functions
Confidence: 0.50
```

---

## Interesting Differences from Go

| Aspect | Go | Rust |
|--------|-----|------|
| Struct methods | `(s *Struct)` receiver | `impl Struct` block |
| Private fields | lowercase | No `pub` keyword |
| Constructor naming | `NewXxx` | `new` in impl |
| Traits | interfaces | `impl Trait for Type` |
| Error handling | return `(value, error)` | `Result<T, E>` |

---

## Try It Yourself

```bash
cd /home/jake/Compa/garfield

# Build Rust example
cargo run -- build ./examples/rust

# Query
cargo run -- query "user"

# Explain
cargo run -- explain "UserStore"
cargo run -- explain "create"
```

---

## See Also

- [Examples: go.md](go.md) - Similar example in Go
- [Modules: extract.md](../modules/extract.md) - How Rust is parsed
