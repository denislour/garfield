# Module: cache.rs

## Purpose

**Cache extraction results** to avoid re-parsing unchanged files.

---

## The Problem It Solves

Parsing code is slow. If you run `build` twice on the same codebase:

| Without Cache | With Cache |
|--------------|------------|
| Parse everything twice | Parse only changed files |
| 10 seconds | 0.1 seconds |
| 100% CPU both times | Minimal CPU second time |

---

## How It Works

### 1. Compute Hash

```rust
pub fn compute_hash(content: &str) -> String {
    // SHA256 hash of file content
    // "abc123..."
}
```

### 2. Check Cache

```rust
pub fn check_cache(
    cache_dir: &Path,
    file_path: &str,
    content_hash: &str,
) -> Option<FileCache>
```

**Logic:**
```
If cache file exists AND hash matches:
    Return cached result
Else:
    Return None (need to re-extract)
```

### 3. Save to Cache

```rust
pub fn update_cache(
    cache_dir: &Path,
    file_path: &str,
    content_hash: &str,
    nodes: &[Node],
    edges: &[Edge],
) -> anyhow::Result<()>
```

---

## Cache Structure

**Location:** `garfield-out/cache/`

```
garfield-out/
├── graph.json
├── cache/
│   ├── file_abc123.json    # Cached extraction
│   ├── file_def456.json
│   └── manifest.json       # Index of cached files
```

**Cache file content:**
```json
{
  "path": "./src/main.rs",
  "hash": "sha256:abc123...",
  "extracted_at": "2024-01-15T10:30:00Z",
  "nodes": [...],
  "edges": [...]
}
```

---

## Build Integration

In `build_graph()`:

```rust
for file in files {
    let hash = compute_hash(&file.content)?;
    
    // Check cache first
    if let Some(cached) = check_cache(&cache_dir, &file.path, &hash)? {
        println!("  [cached] {}", file.path);
        extractions.push(cached.into());
        continue;
    }
    
    // Cache miss - extract
    let result = extract_file(&file.path)?;
    update_cache(&cache_dir, &file.path, &hash, &result.nodes, &result.edges)?;
    
    extractions.push(result);
}
```

---

## Benefits

| Scenario | Without Cache | With Cache |
|----------|--------------|------------|
| First build | Parse all files | Parse all files |
| Second build (no changes) | Parse all files | Use cache |
| After edit 1 file | Parse all files | Parse 1 file + use cache |
| Large codebase | Slow | Fast after first run |

---

## Cache Invalidation

Cache is invalidated when:
1. File content changes (hash doesn't match)
2. File is deleted
3. Cache is manually cleared

```bash
# Clear all cache
cargo run -- build --clear-cache ./src

# Or delete manually
rm -rf garfield-out/cache/
```

---

## Code Location

```
src/
├── cache.rs         ← THIS FILE
│   ├── compute_hash()      ← Hash file content
│   ├── check_cache()       ← Check if cached
│   ├── update_cache()      ← Save to cache
│   ├── load_cached()       ← Load cache file
│   ├── save_cached()       ← Save cache file
│   └── clear_all_cache()  ← Clear cache
│
└── types.rs
    └── FileCache struct
```

---

## Run It

```bash
# First build (no cache)
cargo run -- build ./examples/go
# → Extracts all files

# Second build (uses cache)
cargo run -- build ./examples/go
# → Shows "[cached]" for unchanged files

# Clear cache
rm -rf garfield-out/cache/
cargo run -- build ./examples/go
# → Re-extracts everything
```

---

## See Also

- [Flow: build.md](../flow/build.md) - Where cache fits
