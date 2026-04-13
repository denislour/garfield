# Module: detect.rs

## Purpose

**Find files in a directory** by extension. This is the first step - figure out what code files exist.

---

## What It Does

Given a directory path, it:
1. Recursively scans for files
2. Filters by known extensions (.go, .rs, .py, .rb, .js, .ts, .java, etc.)
3. Skips hidden files and noise directories (.git, node_modules, target)
4. Computes file hashes for caching

---

## Input/Output

**Input:**
```
Directory path: "./examples/go"
```

**Output:**
```rust
struct DetectResult {
    files: Vec<DetectedFile>,
    total_scanned: usize,
    code_files: usize,
    noise_skipped: usize,
}

struct DetectedFile {
    path: String,        // "./examples/go/user_store.go"
    file_type: FileType, // Code, Markdown, Other
    size_bytes: usize,
    hash: String,        // For cache
}
```

---

## File Type Classification

| Extension | FileType |
|-----------|----------|
| .go, .rs, .py, .rb, .js, .ts, .java, .zig, .ex | `Code` |
| .md, .txt | `Markdown` |
| * | `Other` |

---

## Example

```bash
cargo run -- build ./examples/go
```

**Output:**
```
Detecting files...
📁 Detection Summary:
  Total entries scanned: 5
  Hidden files skipped: 0
  Noise directories skipped: 2 (.git, target)
  
📊 File Classification:
  Code files: 2 (user_store.go, main.go)
  Markdown: 0
  Other: 1 (README)
```

---

## Why It Matters

This is the **entry point** for the entire pipeline. If a file isn't detected, it won't be extracted.

**Typical issues:**
- Wrong directory path
- Unknown file extension
- File in noise directory (skipped)

---

## Code Location

```
src/
├── detect.rs         ← THIS FILE
│   ├── detect()      ← Main function
│   ├── classify_file() ← Determine FileType
│   └── should_skip() ← Check noise patterns
│
└── types.rs
    └── FileType enum
```

---

## See Also

- [Flow: build.md](../flow/build.md) - Where detect fits
- [Modules: extract.md](extract.md) - What happens next
