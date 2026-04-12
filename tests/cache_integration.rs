//! Cache integration tests

use garfield::cache::CacheEntry;
use garfield::FileCache;
use std::collections::HashMap;
use std::path::Path;

#[test]
fn test_cache_save_and_load() {
    let cache_path = Path::new("/tmp/garfield_integration_cache.json");

    let mut entries = HashMap::new();
    entries.insert("file1.py".to_string(), CacheEntry {
        path: "file1.py".to_string(),
        hash: "hash1".to_string(),
        size: 100,
        modified: 0,
        source_file: None,
    });

    let cache = FileCache {
        entries,
        by_source_file: HashMap::new(),
        version: "1.0".to_string(),
    };

    cache.save(cache_path).expect("should save");
    let loaded = FileCache::load(cache_path).expect("should load");
    assert_eq!(loaded.entries.len(), 1);
    let _ = std::fs::remove_file(cache_path);
}

#[test]
fn test_cache_with_source_tracking() {
    let cache_path = Path::new("/tmp/garfield_integration_source.json");

    let mut entries = HashMap::new();
    let mut by_source = HashMap::new();
    
    entries.insert("module/a.py".to_string(), CacheEntry {
        path: "module/a.py".to_string(),
        hash: "hash_a".to_string(),
        size: 50,
        modified: 0,
        source_file: Some("main.py".to_string()),
    });
    
    by_source.insert("main.py".to_string(), vec!["module/a.py".to_string()]);
    
    let cache = FileCache {
        entries,
        by_source_file: by_source,
        version: "1.0".to_string(),
    };
    
    cache.save(cache_path).expect("should save");
    let loaded = FileCache::load(cache_path).expect("should load");
    assert!(loaded.by_source_file.contains_key("main.py"));
    let _ = std::fs::remove_file(cache_path);
}
