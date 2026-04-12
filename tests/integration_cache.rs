//! Cache integration tests

use garfield::cache::CacheEntry;
use garfield::FileCache;
use std::collections::HashMap;
use std::path::Path;

#[test]
fn test_cache_functionality() {
    let cache_path = Path::new("/tmp/test_garfield_cache.json");

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
