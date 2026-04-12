//! Caching module - SHA256 hashing with smart content detection
//!
//! ## Cache Flow
//!
//! 1. compute_hash() - SHA256 of file content (MD body only for .md files)
//! 2. check_cache() - Compare current hashes vs cached hashes
//! 3. update_cache() - Add/update cache entries
//! 4. load_cached() / save_cached() - Per-file cache with group by file
//! 5. clear_cache() - Remove specific files
//!
//! ## Smart MD Hashing
//!
//! For Markdown files (.md), only the body below the YAML frontmatter is hashed.
//! This prevents cache invalidation on metadata-only changes (reviewed, status, tags).
//!
//! ## Group by File
//!
//! Cache entries are organized by source_file, making incremental builds more efficient.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Cache entry for a single file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    /// File path
    pub path: String,
    /// SHA256 hash of file content
    pub hash: String,
    /// File size in bytes
    pub size: u64,
    /// Last modified timestamp
    pub modified: u64,
    /// Source file this belongs to
    pub source_file: Option<String>,
}

/// File cache with group by file support
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FileCache {
    /// HashMap: path -> CacheEntry
    pub entries: HashMap<String, CacheEntry>,
    /// Group by source_file: source_file -> [paths]
    #[serde(default)]
    pub by_source_file: HashMap<String, Vec<String>>,
    /// Cache version for compatibility
    pub version: String,
}

impl FileCache {
    /// Create new cache
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            by_source_file: HashMap::new(),
            version: "2.0".to_string(),
        }
    }

    /// Load cache from file
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        if !path.exists() {
            return Ok(Self::new());
        }
        let content = fs::read_to_string(path)?;
        let cache: FileCache = serde_json::from_str(&content)?;
        Ok(cache)
    }

    /// Save cache to file
    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    /// Add an entry and update by_source_file index
    pub fn add_entry(&mut self, entry: CacheEntry) {
        let path = entry.path.clone();
        let source_file = entry.source_file.clone();
        
        // Add to entries
        self.entries.insert(path.clone(), entry);
        
        // Update by_source_file index
        if let Some(sf) = source_file {
            self.by_source_file
                .entry(sf)
                .or_default()
                .push(path);
        }
    }

    /// Remove entries by path
    pub fn remove_entries(&mut self, paths: &[String]) {
        for path in paths {
            if let Some(entry) = self.entries.remove(path) {
                // Remove from by_source_file index
                if let Some(sf) = entry.source_file {
                    if let Some(paths_in_file) = self.by_source_file.get_mut(&sf) {
                        paths_in_file.retain(|p| p != path);
                        if paths_in_file.is_empty() {
                            self.by_source_file.remove(&sf);
                        }
                    }
                }
            }
        }
    }
}

/// Extract body content from Markdown (strip YAML frontmatter)
/// 
/// For Markdown files with YAML frontmatter like:
/// ```markdown
/// ---
/// title: My Document
/// tags: [foo, bar]
/// ---
/// 
/// # Actual content
/// ```
/// 
/// Only the body after the closing `---` is hashed, so metadata changes
/// don't invalidate the cache.
fn extract_md_body(content: &str) -> &str {
    if content.starts_with("---") {
        if let Some(end_pos) = content[3..].find("\n---") {
            return &content[end_pos + 4..];
        }
    }
    content
}

/// Compute SHA256 hash of file content
/// 
/// For .md files, only the body (below YAML frontmatter) is hashed.
/// This prevents cache invalidation on metadata changes.
pub fn compute_hash(path: &Path) -> anyhow::Result<String> {
    let content = fs::read(path)?;
    
    let hash_input = if path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default() == "md" 
    {
        // For MD files, hash only the body
        let content_str = String::from_utf8_lossy(&content);
        let body = extract_md_body(&content_str);
        format!("{}\n\x00{}", body, path.to_string_lossy())
    } else {
        // For other files, hash full content + path
        format!(
            "{}\n\x00{}",
            String::from_utf8_lossy(&content),
            path.to_string_lossy()
        )
    };
    
    let mut hasher = Sha256::new();
    hasher.update(hash_input.as_bytes());
    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

/// Check which files have changed
/// 
/// Returns (changed_files, unchanged_files)
pub fn check_cache(files: &[PathBuf], cache: &FileCache) -> (Vec<PathBuf>, Vec<PathBuf>) {
    let mut changed = Vec::new();
    let mut unchanged = Vec::new();

    for file in files {
        let path_str = file.to_string_lossy().to_string();

        match (compute_hash(file), cache.entries.get(&path_str)) {
            (Ok(hash), Some(entry)) => {
                if hash != entry.hash {
                    changed.push(file.clone());
                } else {
                    unchanged.push(file.clone());
                }
            }
            _ => {
                // Error or not in cache = changed
                changed.push(file.clone());
            }
        }
    }

    (changed, unchanged)
}

/// Update cache with new file hashes
pub fn update_cache(cache: &mut FileCache, files: &[PathBuf], source_file: Option<&str>) -> anyhow::Result<()> {
    for file in files {
        let path_str = file.to_string_lossy().to_string();

        if let Ok(hash) = compute_hash(file) {
            let metadata = fs::metadata(file)?;

            cache.add_entry(CacheEntry {
                path: path_str,
                hash,
                size: metadata.len(),
                modified: metadata
                    .modified()
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs())
                    .unwrap_or(0),
                source_file: source_file.map(|s| s.to_string()),
            });
        }
    }
    Ok(())
}

/// Clear cache for specific files
pub fn clear_cache(cache: &mut FileCache, files: &[PathBuf]) {
    let paths: Vec<String> = files.iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();
    cache.remove_entries(&paths);
}

/// Get cache stats
pub fn cache_stats(cache: &FileCache) -> CacheStats {
    CacheStats {
        total_entries: cache.entries.len(),
        files_by_source: cache.by_source_file.len(),
        file_paths: cache.entries.keys().cloned().collect(),
    }
}

#[derive(Debug)]
pub struct CacheStats {
    pub total_entries: usize,
    pub files_by_source: usize,
    pub file_paths: Vec<String>,
}

/// Per-file cache operations (graphify-out/cache/{hash}.json style)

/// Get the cache directory path
pub fn get_cache_dir(root: &Path) -> PathBuf {
    let dir = root.join("graphify-out").join("cache");
    fs::create_dir_all(&dir).ok();
    dir
}

/// Load cached extraction for a file
/// 
/// Returns cached nodes/edges if hash matches, else None
pub fn load_cached(path: &Path, root: &Path) -> Option<CachedExtraction> {
    let hash = compute_hash(path).ok()?;
    let cache_file = get_cache_dir(root).join(format!("{}.json", hash));
    
    if !cache_file.exists() {
        return None;
    }
    
    serde_json::from_str(&fs::read_to_string(cache_file).ok()?).ok()
}

/// Save extraction result to cache
pub fn save_cached(path: &Path, result: &CachedExtraction, root: &Path) -> anyhow::Result<()> {
    let hash = compute_hash(path)?;
    let cache_file = get_cache_dir(root).join(format!("{}.json", hash));
    
    let json = serde_json::to_string_pretty(result)?;
    
    // Write atomically using temp file
    let tmp_file = cache_file.with_extension("tmp");
    fs::write(&tmp_file, json)?;
    std::fs::rename(tmp_file, cache_file)?;
    
    Ok(())
}

/// Cached extraction result
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CachedExtraction {
    pub nodes: Vec<crate::types::Node>,
    pub edges: Vec<crate::types::Edge>,
    #[serde(default)]
    pub hyperedges: Vec<crate::types::Hyperedge>,
}

/// Check semantic cache for multiple files
/// 
/// Returns (cached_nodes, cached_edges, cached_hyperedges, uncached_files)
pub fn check_semantic_cache(
    files: &[String],
    root: &Path,
) -> (Vec<crate::types::Node>, Vec<crate::types::Edge>, Vec<crate::types::Hyperedge>, Vec<String>) {
    let mut cached_nodes = Vec::new();
    let mut cached_edges = Vec::new();
    let mut cached_hyperedges = Vec::new();
    let mut uncached = Vec::new();
    
    for fpath in files {
        let path = Path::new(fpath);
        if let Some(cached) = load_cached(path, root) {
            cached_nodes.extend(cached.nodes);
            cached_edges.extend(cached.edges);
            cached_hyperedges.extend(cached.hyperedges);
        } else {
            uncached.push(fpath.clone());
        }
    }
    
    (cached_nodes, cached_edges, cached_hyperedges, uncached)
}

/// Save semantic cache grouped by source_file
/// 
/// Groups nodes and edges by source_file, saves one cache entry per file
pub fn save_semantic_cache(
    nodes: &[crate::types::Node],
    edges: &[crate::types::Edge],
    hyperedges: &[crate::types::Hyperedge],
    root: &Path,
) -> anyhow::Result<usize> {
    use std::collections::HashMap;
    
    // Group by source_file
    let mut by_file: HashMap<String, CachedExtraction> = HashMap::new();
    
    for node in nodes {
        let src = &node.source_file;
        if !src.is_empty() {
            by_file.entry(src.clone())
                .or_default()
                .nodes
                .push(node.clone());
        }
    }
    
    for edge in edges {
        let src = &edge.source_file;
        if !src.is_empty() {
            by_file.entry(src.clone())
                .or_default()
                .edges
                .push(edge.clone());
        }
    }
    
    for hyperedge in hyperedges {
        let src = &hyperedge.source_file;
        if !src.is_empty() {
            by_file.entry(src.clone())
                .or_default()
                .hyperedges
                .push(hyperedge.clone());
        }
    }
    
    // Save each file's cache
    let mut saved = 0;
    for (fpath, result) in by_file {
        let path = Path::new(&fpath);
        if path.exists() {
            save_cached(path, &result, root)?;
            saved += 1;
        }
    }
    
    Ok(saved)
}

/// Clear all cache entries
pub fn clear_all_cache(root: &Path) -> usize {
    let cache_dir = get_cache_dir(root);
    let mut removed = 0;
    
    if let Ok(entries) = fs::read_dir(cache_dir) {
        for entry in entries.flatten() {
            if entry.path().extension().map(|e| e == "json").unwrap_or(false) {
                if fs::remove_file(entry.path()).is_ok() {
                    removed += 1;
                }
            }
        }
    }
    
    removed
}
