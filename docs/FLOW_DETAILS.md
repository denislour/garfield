# Chi Tiết Từng Function - Python vs Rust

## 1. build.py → build.rs

### Python: `build_from_json()`

```python
def build_from_json(extraction: dict, *, directed: bool = False) -> nx.Graph:
```

#### Input (extraction dict):
```json
{
    "nodes": [
        {"id": "a.py:foo", "label": "foo", "source_file": "a.py", ...},
        {"id": "a.py:bar", "label": "bar", "source_file": "a.py", ...}
    ],
    "edges": [
        {"source": "a.py:foo", "target": "a.py:bar", "relation": "calls", ...}
    ],
    "hyperedges": []
}
```

#### Flow (Python):

```
┌─────────────────────────────────────────────────────────────────┐
│                    build_from_json(extraction)                     │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
                    ┌─────────────────┐
                    │ validate_extraction │
                    │ (kiểm tra schema)  │
                    └────────┬─────────┘
                              │
              ┌───────────────┴───────────────┐
              │ Nếu có lỗi thật (không phải  │
              │ dangling edges) → in warning  │
              └───────────────┬───────────────┘
                              │
                              ▼
                    ┌─────────────────┐
                    │ Tạo nx.Graph()  │
                    │ hoặc nx.DiGraph()│
                    └────────┬─────────┘
                              │
                              ▼
              ┌───────────────────────────────────┐
              │  LOOP: nodes trong extraction      │
              │  G.add_node(node["id"], attributes) │
              └───────────────┬───────────────────┘
                              │
                              ▼
              ┌───────────────────────────────────┐
              │  LOOP: edges trong extraction      │
              │  • Skip nếu src/target không tồn tại│
              │  • G.add_edge(src, tgt, attrs)     │
              │  • Lưu _src, _tgt để preserve     │
              │    direction trong undirected graph │
              └───────────────┬───────────────────┘
                              │
                              ▼
                    ┌─────────────────┐
                    │ Return G (Graph)│
                    └─────────────────┘
```

#### Giải thích chi tiết:

1. **`validate_extraction(extraction)`**: Kiểm tra xem extraction dict có đúng schema không
   - Nodes phải có `id`
   - Edges phải có `source`, `target`
   - Nếu có lỗi → in warning (ngoại trừ dangling edges - tức edges trỏ đến external nodes)

2. **Tạo Graph**: 
   - `directed=True` → `nx.DiGraph()` (có hướng)
   - `directed=False` → `nx.Graph()` (vô hướng)

3. **Thêm nodes**:
   - Mỗi node được thêm với `G.add_node(id, attributes)`
   - NetworkX tự động ignore duplicate IDs (overwrites)

4. **Thêm edges**:
   - Skip edges mà source/target không tồn tại (external imports)
   - Lưu `_src` và `_tgt` để preserve direction trong undirected graph

---

### Python: `build()`

```python
def build(extractions: list[dict], *, directed: bool = False) -> nx.Graph:
```

#### Flow:

```
┌─────────────────────────────────────────┐
│           build(extractions: list[dict])  │
└─────────────────────────────────────────┘
                      │
                      ▼
        ┌─────────────────────────────┐
        │ combined = {                 │
        │   "nodes": [],              │
        │   "edges": [],              │
        │   "hyperedges": []          │
        │ }                           │
        └─────────────┬───────────────┘
                      │
        ┌─────────────┴───────────────┐
        │ LOOP: extractions           │
        │ • combined["nodes"].extend() │
        │ • combined["edges"].extend()│
        │ • combined["hyperedges"]... │
        └─────────────┬───────────────┘
                      │
                      ▼
        ┌─────────────────────────────┐
        │ build_from_json(combined)   │
        └─────────────┬───────────────┘
                      │
                      ▼
              ┌───────────────┐
              │ Return Graph  │
              └───────────────┘
```

**Đơn giản:** Merge tất cả extractions thành một dict, rồi gọi `build_from_json()`.

---

### Rust: `build_graph()`

```rust
pub fn build_graph(extractions: Vec<ExtractionResult>) -> GraphData {
```

#### Rust Data Structures (khác Python):

```rust
// Rust dùng native structs thay vì dict
pub struct Node {
    pub id: String,
    pub label: String,
    pub source_file: String,
    pub source_location: String,
    pub community: Option<u32>,
    pub node_type: Option<String>,
}

pub struct Edge {
    pub source: String,
    pub target: String,
    pub relation: String,
    pub confidence: Confidence,
}

pub struct ExtractionResult {
    pub nodes: Vec<Node>,      // Vec thay vì list[dict]
    pub edges: Vec<Edge>,
}

pub struct GraphData {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub metadata: GraphMetadata,
}
```

#### Flow (Rust):

```
┌─────────────────────────────────────────────┐
│          build_graph(extractions)            │
└─────────────────────────────────────────────┘
                        │
        ┌───────────────┴───────────────┐
        │  LOOP: extractions            │
        │  nodes.extend(extraction.nodes)│
        │  edges.extend(extraction.edges)│
        └───────────────┬───────────────┘
                        │
                        ▼
        ┌───────────────────────────────────┐
        │ DEDUPLICATE nodes                  │
        │ nodes.sort_by_key(id)              │
        │ nodes.dedup_by_key(id)             │
        └───────────────┬───────────────────┘
                        │
                        ▼
        ┌───────────────────────────────────┐
        │ DEDUPLICATE edges                  │
        │ edges.sort()                       │
        │ edges.dedup()                      │
        └───────────────┬───────────────────┘
                        │
                        ▼
        ┌───────────────────────────────────┐
        │ Tạo GraphData (không có community)│
        └───────────────┬───────────────────┘
                        │
                        ▼
        ┌───────────────────────────────────┐
        │ cluster(&graph)                    │
        │ (chạy community detection)         │
        └───────────────┬───────────────────┘
                        │
                        ▼
        ┌───────────────────────────────────┐
        │ add_communities(&mut graph, ...)   │
        │ (gán community IDs vào nodes)      │
        └───────────────┬───────────────────┘
                        │
                        ▼
        ┌───────────────────────────────────┐
        │ split_oversized(&mut graph, 25)    │
        │ (tách communities > 25 nodes)      │
        └───────────────┬───────────────────┘
                        │
                        ▼
                ┌───────────────┐
                │ Return GraphData│
                └───────────────┘
```

#### So Sánh Python vs Rust:

| Khía cạnh | Python | Rust |
|-----------|--------|------|
| Data structure | `dict` (dynamic) | `struct` (type-safe) |
| Graph type | NetworkX `Graph`/`DiGraph` | Simple `Vec<Node>` + `Vec<Edge>` |
| Deduplication | Tự động (NetworkX) | Thủ công (`sort` + `dedup`) |
| Community detection | Cần gọi riêng | Tích hợp trong `build_graph` |
| Directed/undirected | Có option | Không - luôn vô hướng |

---

## 2. extract.py → extract.rs (MINH HỌA)

### Python: `extract_file()` - Phức tạp hơn nhiều

```python
def extract_file(path: Path) -> dict:
    """
    Python có LanguageConfig cho 15+ ngôn ngữ:
    """
    config = LANGUAGES.get(language, DEFAULT_CONFIG)
    
    # Parse với tree-sitter
    tree = parser.parse(source)
    
    # Walk tree với config
    for node in walk(tree, config):
        if is_definition(node, config):
            add_node(node)
        elif is_call(node, config):
            add_edge(node)
        elif is_import(node, config):
            handle_import(node, config)
```

#### Flow Chi Tiết (Python):

```
┌─────────────────────────────────────────┐
│              extract_file(path)           │
└─────────────────────────────────────────┘
                      │
        ┌─────────────┴─────────────┐
        │ Xác định ngôn ngữ          │
        │ (extension → language)      │
        │ Python, JS, Go, Rust...    │
        └─────────────┬─────────────┘
                      │
                      ▼
        ┌───────────────────────────────────┐
        │ Lấy LanguageConfig cho ngôn ngữ    │
        │ Ví dụ:                            │
        │ Python:                           │
        │   class_types = {"class_def"}     │
        │   function_types = {"func_def"}   │
        │   import_types = {"import"}       │
        └─────────────┬───────────────────┘
                      │
                      ▼
        ┌───────────────────────────────────┐
        │ Parse với tree-sitter              │
        │ parser.parse(source) → AST tree     │
        └─────────────┬───────────────────┘
                      │
                      ▼
        ┌───────────────────────────────────┐
        │ PASS 1: Collect definitions        │
        │ • Tìm class definitions           │
        │ • Tìm function definitions        │
        │ • Tìm imports                    │
        │ Tạo nodes với IDs                │
        └─────────────┬───────────────────┘
                      │
                      ▼
        ┌───────────────────────────────────┐
        │ PASS 2: Collect calls              │
        │ • Tìm function calls              │
        │ • Tạo edges đến definitions       │
        │ • Confidence: EXTRACTED/INFERRED  │
        └─────────────┬───────────────────┘
                      │
                      ▼
        ┌───────────────────────────────────┐
        │ PASS 3: Handle imports (ngôn ngữ)  │
        │ • _import_python() cho Python      │
        │ • _import_js() cho JavaScript      │
        │ • _import_java() cho Java          │
        │ • ... 15+ ngôn ngữ                │
        └─────────────┬───────────────────┘
                      │
                      ▼
              ┌───────────────┐
              │ Return dict    │
              │ {nodes, edges} │
              └───────────────┘
```

### Rust: `extract_file()` - Đơn giản hơn

```rust
pub fn extract_file(path: &Path, source: &str) -> anyhow::Result<ExtractionResult> {
    // Chỉ có simple_extract fallback
    // Không có LanguageConfig phức tạp
    
    // Parse với tree-sitter
    let tree = parser.parse(source, None)?;
    
    // Walk tree
    walk_tree_pass1(&root, ...);  // Collect definitions
    walk_tree_pass2(&root, ...);  // Collect calls
    
    // Không có language-specific import handlers!
}
```

#### Flow Chi Tiết (Rust):

```
┌─────────────────────────────────────────┐
│          extract_file(path, source)       │
└─────────────────────────────────────────┘
                      │
        ┌─────────────┴─────────────┐
        │ Xác định extension        │
        │ → language name            │
        └─────────────┬─────────────┘
                      │
                      ▼
        ┌───────────────────────────────────┐
        │ Thử get_language(language)         │
        │ (tree-sitter-language-pack)        │
        │ Nếu fail → simple_extract()        │
        └─────────────┬───────────────────┘
                      │
                      ▼
        ┌───────────────────────────────────┐
        │ Parse với tree-sitter              │
        │ parser.set_language(&lang)          │
        │ parser.parse(source, None)          │
        └─────────────┬───────────────────┘
                      │
                      ▼
        ┌───────────────────────────────────┐
        │ PASS 1: walk_tree_pass1            │
        │ • Tìm "function_definition"        │
        │ • Tìm "class_definition"          │
        │ • Tạo nodes                        │
        │ Lưu known_nodes để reference       │
        └─────────────┬───────────────────┘
                      │
                      ▼
        ┌───────────────────────────────────┐
        │ PASS 2: walk_tree_pass2            │
        │ • Tìm "call_expression"           │
        │ • Tạo edges calls                 │
        │ • Confidence: Extracted nếu known  │
        │            Inferred nếu unknown     │
        └─────────────┬───────────────────┘
                      │
                      ▼
              ┌───────────────┐
              │ Return Result │
              │ ExtractionResult│
              └───────────────┘
```

---

## 3. cache.py → cache.rs

### Python: `load_cached()` vs Rust: `FileCache::load()`

#### Python:
```python
def load_cached(path: Path, root: Path = Path(".")) -> dict | None:
    cache_file = cache_dir(root) / f"{path.name}.json"
    if cache_file.exists():
        return json.loads(cache_file.read_text())
    return None
```

**Flow:**
```
load_cached(path, root)
       │
       ▼
cache_file = cache_dir(root) / f"{path.name}.json"
       │
       ▼
cache_file.exists()?
  ├── Yes → json.loads(read_text()) → return dict
  └── No  → return None
```

#### Rust:
```rust
impl FileCache {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        if !path.exists() {
            return Ok(Self::new());  // Return empty cache
        }
        let content = fs::read_to_string(path)?;
        let cache: FileCache = serde_json::from_str(&content)?;
        Ok(cache)
    }
}
```

**Flow:**
```
FileCache::load(path)
       │
       ▼
path.exists()?
  ├── Yes → fs::read_to_string() → serde_json::from_str() → return Cache
  └── No  → FileCache::new() (empty)
```

---

## 4. analyze.py → analyze.rs

### Python: `god_nodes()` vs Rust: `find_god_nodes()`

#### Python:
```python
def god_nodes(G: nx.Graph, top_n: int = 10) -> list[dict]:
    degree = dict(G.degree())
    sorted_nodes = sorted(degree.items(), key=lambda x: x[1], reverse=True)
    result = []
    for node_id, deg in sorted_nodes:
        # Filter: bỏ qua file nodes và concept nodes
        if _is_file_node(G, node_id) or _is_concept_node(G, node_id):
            continue
        result.append({"id": node_id, "label": ..., "edges": deg})
        if len(result) >= top_n:
            break
    return result
```

#### Rust:
```rust
pub fn find_god_nodes(graph: &GraphData, top_n: usize) -> Vec<GodNode> {
    // Tính degree cho mỗi node
    let mut degree: HashMap<&str, usize> = HashMap::new();
    for edge in &graph.edges {
        *degree.entry(&edge.source).or_insert(0) += 1;
        *degree.entry(&edge.target).or_insert(0) += 1;
    }
    
    // Sort theo degree
    let mut sorted: Vec<_> = degree.into_iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));
    
    // Take top N (KHÔNG có filter _is_file_node/_is_concept_node!)
    sorted.into_iter()
        .take(top_n)
        .filter_map(...)
        .collect()
}
```

---

## Tổng Kết So Sánh Flow

| Function | Python Flow | Rust Flow | Khác Biệt Chính |
|----------|------------|-----------|-----------------|
| `build_from_json` | validate → create Graph → add nodes → add edges | collect → dedup → create GraphData | Rust tích hợp community detection |
| `build` | merge dicts → build_from_json | collect → dedup → cluster | Tương tự |
| `extract_file` | 3 passes với LanguageConfig | 2 passes đơn giản | Python có 15+ language handlers |
| `load_cached` | Đọc file riêng cho mỗi path | Đọc 1 file cache chính | Python dùng nhiều files, Rust dùng 1 file |
| `god_nodes` | Filter file/concept nodes | Không filter | Rust thiếu `_is_file_node` |

---

## Ví Dụ Cụ Thể: Extract 1 File Python

### Input:
```python
# a.py
import os
from sys import path

class Foo:
    def bar(self):
        os.getcwd()
```

### Python Output:
```json
{
  "nodes": [
    {"id": "a.py", "label": "a.py", "source_file": "a.py"},
    {"id": "a.py:Foo", "label": "Foo", "source_file": "a.py"},
    {"id": "a.py:Foo.bar", "label": ".bar()", "source_file": "a.py"},
    {"id": "os", "label": "os"},
    {"id": "sys", "label": "sys"},
    {"id": "sys.path", "label": "path"}
  ],
  "edges": [
    {"source": "a.py", "target": "os", "relation": "imports", "confidence": "EXTRACTED"},
    {"source": "a.py", "target": "sys", "relation": "imports_from", "confidence": "EXTRACTED"},
    {"source": "a.py:Foo", "target": "a.py:Foo.bar", "relation": "contains", "confidence": "EXTRACTED"},
    {"source": "a.py:Foo.bar", "target": "os", "relation": "calls", "confidence": "INFERRED"}
  ]
}
```

### Rust Output (hiện tại - đơn giản hơn):
```json
{
  "nodes": [
    {"id": "a:Foo", "label": "Foo", "source_file": "a.py"},
    {"id": "a:bar", "label": "bar", "source_file": "a.py"}
  ],
  "edges": [
    {"source": "a:bar", "target": "a:Foo", "relation": "calls", "confidence": "INFERRED"}
  ]
}
```

**Lý do khác biệt:**
- Rust không extract imports
- Rust không tạo file-level node
- Rust không tạo method nodes riêng
