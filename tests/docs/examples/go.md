# Example: Go (user_store.go)

## Source Code

**File:** `examples/go/user_store.go`

```go
package user

import (
	"errors"
	"sync"
)

type User struct {
	ID     uint64
	Name   string
	Email  string
	Active bool
}

type UserStore struct {
	mu      sync.RWMutex
	users   map[uint64]User
	nextID  uint64
}

func NewUserStore() *UserStore {
	return &UserStore{
		users: make(map[uint64]User),
		nextID: 1,
	}
}

func (s *UserStore) Create(name, email string) (uint64, error) {
	if name == "" {
		return 0, errors.New("name is required")
	}
	id := s.nextID
	s.nextID++
	user := User{ID: id, Name: name, Email: email, Active: true}
	s.users[id] = user
	return id, nil
}

func (s *UserStore) Find(id uint64) (User, bool) {
	s.mu.RLock()
	defer s.mu.RUnlock()
	user, ok := s.users[id]
	return user, ok
}

func (s *UserStore) Update(id uint64, name, email string) bool {
	// ... implementation
	return true
}

func (s *UserStore) Delete(id uint64) bool {
	// ... implementation
	return true
}

func (s *UserStore) List() []User {
	// ... implementation
	return nil
}

func (s *UserStore) Count() int {
	// ... implementation
	return 0
}
```

---

## Step 1: BUILD

```bash
cargo run -- build ./examples/go
```

**Output:**
```
✅ Build complete!
  Nodes: 14
  Edges: 5
  Communities: 11
  Hyperedges: 1
```

### What was extracted:

**Nodes (14 total):**
| Label | Type | Location |
|-------|------|----------|
| User | struct | L12 |
| UserStore | struct | L18 |
| NewUserStore | function | L24 |
| Create | method | L32 |
| Find | method | L45 |
| Update | method | L58 |
| Delete | method | L63 |
| List | method | L68 |
| Count | method | L73 |
| errors.New | (import) | L5 |
| make | (builtin) | L26 |
| sync.RWMutex | (import) | L4 |
| map | (builtin) | L19 |
| append | (builtin) | - |

**Edges (5 total):**
| Source | Relation | Target |
|--------|----------|--------|
| Create | calls | errors.New |
| Create | calls | make |
| Find | calls | len |
| List | calls | make |
| List | calls | append |

**Hyperedges (1 total):**
```
id: "file_user_store"
label: "user_store module"
members: 14
confidence: 1.0
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
  • global [user_store module] [./examples/go/user_store.go @ L?]
  • make [user_store module] [./examples/go/user_store.go @ L?]
  • NewUserStore [user_store module] [./examples/go/user_store.go @ L24]
  • append [user_store module] [./examples/go/user_store.go @ L?]
  • len [user_store module] [./examples/go/user_store.go @ L?]
  • delete [user_store module] [./examples/go/user_store.go @ L?]
  • Count [user_store module] [./examples/go/user_store.go @ L95]
  • Create [user_store module] [./examples/go/user_store.go @ L32]
```

**Notice:** Every node has `[user_store module]` tag - this is the hyperedge!

---

## Step 3: EXPLAIN

```bash
cargo run -- explain "Create"
```

**Output:**
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

**Interpretation:**
- `Create` is a method in the `user_store` package
- It belongs to a module with 14 functions total
- Nothing calls it directly (only global/initialization)

---

## Try It Yourself

```bash
cd /home/jake/Compa/garfield

# Build
cargo run -- build ./examples/go

# Query for different things
cargo run -- query "user"
cargo run -- query "Create"
cargo run -- query "error"

# Explain different nodes
cargo run -- explain "UserStore"
cargo run -- explain "NewUserStore"
cargo run -- explain "Find"
```

---

## See Also

- [Flow: build.md](../flow/build.md) - How the build works
- [Flow: query.md](../flow/query.md) - How query works
- [Flow: explain.md](../flow/explain.md) - How explain works
- [Modules: extract.md](../modules/extract.md) - How extraction works
- [Modules: hyperedge.md](../modules/hyperedge.md) - How modules are detected
