//! Cache tests

use garfield::cache::CacheEntry;
use garfield::FileCache;
use std::collections::HashMap;
use std::path::Path;

#[test]
fn test_cache_save_load() {
    let cache_path = Path::new("/tmp/test_cache.json");
    let mut entries = HashMap::new();
    entries.insert("file1.py".to_string(), CacheEntry {
        path: "file1.py".to_string(), hash: "h1".to_string(), size: 100, modified: 0, source_file: None,
    });
    let cache = FileCache { entries, by_source_file: HashMap::new(), version: "1.0".to_string() };
    cache.save(cache_path).unwrap();
    let loaded = FileCache::load(cache_path).unwrap();
    assert_eq!(loaded.entries.len(), 1);
    let _ = std::fs::remove_file(cache_path);
}

#[test]
fn test_cache_source_tracking() {
    let cache_path = Path::new("/tmp/test_source.json");
    let mut entries = HashMap::new();
    let mut by_source = HashMap::new();
    entries.insert("mod/a.py".to_string(), CacheEntry {
        path: "mod/a.py".to_string(), hash: "ha".to_string(), size: 50, modified: 0,
        source_file: Some("main.py".to_string()),
    });
    by_source.insert("main.py".to_string(), vec!["mod/a.py".to_string()]);
    let cache = FileCache { entries, by_source_file: by_source, version: "1.0".to_string() };
    cache.save(cache_path).unwrap();
    let loaded = FileCache::load(cache_path).unwrap();
    assert!(loaded.by_source_file.contains_key("main.py"));
    let _ = std::fs::remove_file(cache_path);
}
