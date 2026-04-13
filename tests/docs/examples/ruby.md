# Example: Ruby (user_service.rb)

## Source Code

**File:** `examples/ruby/user_service.rb`

```ruby
class UserService
  def initialize(database)
    @db = database
  end

  def find_user(id)
    @db.query("SELECT * FROM users WHERE id = ?", id)
  end

  def create_user(name, email)
    @db.query("INSERT INTO users (name, email) VALUES (?, ?)", name, email)
  end

  def update_user(id, attrs)
    user = find_user(id)
    user.update(attrs)
    user.save
  end

  def delete_user(id)
    @db.query("DELETE FROM users WHERE id = ?", id)
  end

  def list_users
    @db.query("SELECT * FROM users")
  end

  def count_users
    @db.query("SELECT COUNT(*) FROM users").first
  end
end
```

---

## Step 1: BUILD

```bash
cargo run -- build ./examples/ruby
```

**Output:**
```
✅ Build complete!
  Nodes: 12
  Edges: 4
  Communities: 10
  Hyperedges: 1
```

### What was extracted:

**Nodes (12 total):**
| Label | Type | Location |
|-------|------|----------|
| UserService | class | L1 |
| initialize | method | L3 |
| find_user | method | L6 |
| create_user | method | L10 |
| update_user | method | L14 |
| delete_user | method | L20 |
| list_users | method | L24 |
| count_users | method | L28 |
| @db | instance_variable | L4 |
| first | method | - |
| update | method | - |
| save | method | - |

**Edges (4 total):**
| Source | Relation | Target |
|--------|----------|--------|
| update_user | calls | find_user |
| count_users | calls | first |

**Hyperedges (1 total):**
```
id: "file_user_service"
label: "user_service module"
members: 12
confidence: 1.0
```

---

## Step 2: QUERY

```bash
cargo run -- query "user"
```

**Output:**
```
Query: "user"
Mode: BFS (depth=3, budget=2000)

## Nodes
  • count_users [user_service module] [./examples/ruby/user_service.rb @ L28]
  • create_user [user_service module] [./examples/ruby/user_service.rb @ L10]
  • UserService [user_service module] [./examples/ruby/user_service.rb @ L1]
```

---

## Step 3: EXPLAIN

```bash
cargo run -- explain "update_user"
```

**Output:**
```
═══ NODE ═══
ID: user_service:update_user
Label: update_user
File: ./examples/ruby/user_service.rb
Location: L14

═══ MODULE (Hyperedge) ═══
Module: user_service module
Members: 12 functions
Confidence: 1.00

═══ CALLED BY ═══
  ← global (calls)
```

**Interesting:** `update_user` calls `find_user` (shown in edges).

---

## Try It Yourself

```bash
cd /home/jake/Compa/garfield

# Build Ruby example
cargo run -- build ./examples/ruby

# Query
cargo run -- query "user"

# Explain
cargo run -- explain "create_user"
cargo run -- explain "UserService"
```

---

## See Also

- [Examples: go.md](go.md) - Similar example in Go
- [Examples: python.md](python.md) - Similar example in Python
- [Modules: extract.md](../modules/extract.md) - How Ruby is parsed
