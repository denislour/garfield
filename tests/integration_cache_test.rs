//! Integration tests for cache module

use garfield::cache::{FileCache, CacheEntry};
use std::collections::HashMap;

#[test]
fn test_file_cache_new() {
    let cache = FileCache::new();
    assert!(cache.entries.is_empty());
    assert!(cache.by_source_file.is_empty());
    assert_eq!(cache.version, "2.0");
}

#[test]
fn test_file_cache_save_load() {
    use tempfile::TempDir;
    
    let temp_dir = TempDir::new().unwrap();
    let cache_path = temp_dir.path().join("cache.json");

    let mut cache = FileCache::new();
    cache.entries.insert(
        "test.rs".to_string(),
        CacheEntry {
            path: "test.rs".to_string(),
            hash: "abc123def456".to_string(),
            size: 100,
            modified: 12345,
            source_file: None,
        },
    );

    cache.save(&cache_path).unwrap();
    let loaded = FileCache::load(&cache_path).unwrap();

    assert_eq!(loaded.entries.len(), 1);
    assert!(loaded.entries.contains_key("test.rs"));
    assert_eq!(loaded.entries.get("test.rs").unwrap().hash, "abc123def456");
}

#[test]
fn test_file_cache_load_nonexistent() {
    let cache = FileCache::load(std::path::Path::new("nonexistent_file_12345.json"));

    assert!(cache.is_ok());
    assert!(cache.unwrap().entries.is_empty());
}

#[test]
fn test_cache_entry_source_file_tracking() {
    let mut cache = FileCache::new();
    
    // Simulate adding entries with source file
    cache.by_source_file.insert(
        "main.rs".to_string(),
        vec!["src/lib.rs".to_string(), "src/main.rs".to_string()],
    );

    assert_eq!(cache.by_source_file.len(), 1);
    assert_eq!(cache.by_source_file.get("main.rs").unwrap().len(), 2);
}

#[test]
fn test_cache_multiple_entries() {
    let mut cache = FileCache::new();
    
    for i in 0..10 {
        cache.entries.insert(
            format!("file{}.rs", i),
            CacheEntry {
                path: format!("file{}.rs", i),
                hash: format!("hash{}", i),
                size: 100 * (i + 1) as u64,
                modified: 1000 * (i + 1) as u64,
                source_file: None,
            },
        );
    }

    assert_eq!(cache.entries.len(), 10);
    
    // Verify all entries are accessible
    for i in 0..10 {
        let key = format!("file{}.rs", i);
        assert!(cache.entries.contains_key(&key));
    }
}

#[test]
fn test_cache_entry_size_tracking() {
    let mut cache = FileCache::new();
    
    cache.entries.insert(
        "small.rs".to_string(),
        CacheEntry {
            path: "small.rs".to_string(),
            hash: "abc".to_string(),
            size: 10,
            modified: 100,
            source_file: None,
        },
    );
    
    cache.entries.insert(
        "large.rs".to_string(),
        CacheEntry {
            path: "large.rs".to_string(),
            hash: "def".to_string(),
            size: 10000,
            modified: 200,
            source_file: None,
        },
    );

    let total_size: u64 = cache.entries.values().map(|e| e.size).sum();
    assert_eq!(total_size, 10010);
}
