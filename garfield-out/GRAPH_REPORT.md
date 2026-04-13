# Graph Report - /home/jake/Compa/garfield  (2026-04-27)

## Corpus Check
- ⚠️ Small corpus: 1 files · ~79 words
  Graph may not add much value for small codebases.

## Summary
- 12 nodes · 4 edges · 10 communities detected
- Extraction: 25% EXTRACTED · 75% INFERRED · 0% AMBIGUOUS · INFERRED: 3 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output (no LLM used)

## God Nodes (most connected - your core abstractions)
1. `global` - 4 edges
   📁 ./example_ruby/user_service.rb · source: user_service:global
2. `query` - 1 edges
   📁 ./example_ruby/user_service.rb · source: user_service:query
3. `user` - 1 edges
   📁 ./example_ruby/user_service.rb · source: user_service:user
4. `find_user` - 1 edges
   📁 ./example_ruby/user_service.rb · source: user_service:find_user
5. `first` - 1 edges
   📁 ./example_ruby/user_service.rb · source: user_service:first

## Surprising Connections (you probably didn't know these)
| Source | Target | Relation | Why |
|--------|--------|----------|-----|
| `global` | `user` | calls | INFERRED connection - model-reasoned relationship; bridges separate communities |
| ↳ ./example_ruby/user_service.rb → ./example_ruby/user_service.rb | INFERRED | 3 |
| `global` | `find_user` | calls | bridges separate communities |
| ↳ ./example_ruby/user_service.rb → ./example_ruby/user_service.rb | EXTRACTED | 2 |

## Communities

### 1 "ExampleRuby" (2 nodes) 🔴
**Cohesion:** 0.00

**Key concepts:** find_user, first

### 0 "ExampleRuby" (2 nodes) 🟢
**Cohesion:** 1.00

**Key concepts:** global, query

### 9 "ExampleRuby" (1 nodes) 🟢
**Cohesion:** 1.00

**Key concepts:** UserService

### 4 "ExampleRuby" (1 nodes) 🟢
**Cohesion:** 1.00

**Key concepts:** create_user

### 5 "ExampleRuby" (1 nodes) 🟢
**Cohesion:** 1.00

**Key concepts:** list_users

### 3 "ExampleRuby" (1 nodes) 🟢
**Cohesion:** 1.00

**Key concepts:** update_user

### 8 "ExampleRuby" (1 nodes) 🟢
**Cohesion:** 1.00

**Key concepts:** delete_user

### 6 "ExampleRuby" (1 nodes) 🟢
**Cohesion:** 1.00

**Key concepts:** count_users

### 2 "ExampleRuby" (1 nodes) 🟢
**Cohesion:** 1.00

**Key concepts:** user

### 7 "ExampleRuby" (1 nodes) 🟢
**Cohesion:** 1.00

**Key concepts:** initialize

## Knowledge Gaps

### 🔌 Isolated Nodes

These have ≤1 connection - possible documentation gaps:

- `UserService`
- `count_users`
- `create_user`
- `delete_user`
- `find_user`

### 📉 Thin Communities

Too small to be meaningful - may be noise:

- `ExampleRuby` (9 nodes)
- `ExampleRuby` (4 nodes)
- `ExampleRuby` (1 nodes)
- `ExampleRuby` (5 nodes)
- `ExampleRuby` (3 nodes)
- `ExampleRuby` (0 nodes)
- `ExampleRuby` (8 nodes)
- `ExampleRuby` (6 nodes)
- `ExampleRuby` (2 nodes)
- `ExampleRuby` (7 nodes)

## 💡 Suggested Questions

Questions the graph is uniquely positioned to answer:

### 1. bridge node

**Q:** Why does `find_user` connect `ExampleRuby` to `ExampleRuby`?

**Why:** High betweenness centrality (0.07272727272727272) - this node is a cross-community bridge.

### 2. bridge node

**Q:** Why does `global` connect `ExampleRuby` to `ExampleRuby`, `ExampleRuby`?

**Why:** High betweenness centrality (0.07272727272727272) - this node is a cross-community bridge.

### 3. bridge node

**Q:** Why does `first` connect `ExampleRuby` to `ExampleRuby`?

**Why:** High betweenness centrality (0.07272727272727272) - this node is a cross-community bridge.

### 4. verify inferred

**Q:** Are the 3 inferred relationships involving `global` (e.g. with `query` and `user`) actually correct?

**Why:** `global` has 3 INFERRED edges - model-reasoned connections that need verification.

### 5. isolated nodes

**Q:** What connects `UserService`, `count_users`, `create_user` to the rest of the system?

**Why:** 3 weakly-connected nodes found - possible documentation gaps or missing edges.

