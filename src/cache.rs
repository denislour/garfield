//! Caching module - SHA256 hashing

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::fs;
use sha2::{Sha256, Digest};
use serde::{Deserialize, Serialize};

/// Cache entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub path: String,
    pub hash: String,
    pub size: u64,
    pub modified: u64,
}

/// File cache
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FileCache {
    pub entries: HashMap<String, CacheEntry>,
    pub version: String,
}

impl FileCache {
    /// Create new cache
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            version: "1.0".to_string(),
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
}

/// Compute SHA256 hash of file content
pub fn compute_hash(path: &Path) -> anyhow::Result<String> {
    let content = fs::read(path)?;
    let mut hasher = Sha256::new();
    hasher.update(&content);
    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

/// Check which files have changed
pub fn check_cache(
    files: &[PathBuf],
    cache: &FileCache,
) -> (Vec<PathBuf>, Vec<PathBuf>) {
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
pub fn update_cache(
    cache: &mut FileCache,
    files: &[PathBuf],
) -> anyhow::Result<()> {
    for file in files {
        let path_str = file.to_string_lossy().to_string();
        
        if let Ok(hash) = compute_hash(file) {
            let metadata = fs::metadata(file)?;
            
            cache.entries.insert(
                path_str.clone(),
                CacheEntry {
                    path: path_str,
                    hash,
                    size: metadata.len(),
                    modified: metadata
                        .modified()
                        .ok()
                        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                        .map(|d| d.as_secs())
                        .unwrap_or(0),
                },
            );
        }
    }
    Ok(())
}

/// Clear cache for specific files
pub fn clear_cache(cache: &mut FileCache, files: &[PathBuf]) {
    for file in files {
        let path_str = file.to_string_lossy().to_string();
        cache.entries.remove(&path_str);
    }
}

/// Get cache stats
pub fn cache_stats(cache: &FileCache) -> CacheStats {
    CacheStats {
        total_entries: cache.entries.len(),
        file_paths: cache.entries.keys().cloned().collect(),
    }
}

#[derive(Debug)]
pub struct CacheStats {
    pub total_entries: usize,
    pub file_paths: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_compute_hash() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("test.txt");
        
        let mut file = fs::File::create(&file_path).unwrap();
        file.write_all(b"hello world").unwrap();
        
        let hash = compute_hash(&file_path).unwrap();
        assert_eq!(hash.len(), 64); // SHA256 produces 64 hex characters
    }

    #[test]
    fn test_cache_roundtrip() {
        let mut cache = FileCache::new();
        
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("test.txt");
        
        let mut file = fs::File::create(&file_path).unwrap();
        file.write_all(b"hello").unwrap();
        
        update_cache(&mut cache, &[file_path.clone()]).unwrap();
        
        let (changed, unchanged) = check_cache(&[file_path.clone()], &cache);
        
        assert_eq!(changed.len(), 0);
        assert_eq!(unchanged.len(), 1);
    }
}
