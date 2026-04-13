//! Cache integration tests

use garfield::cache::{check_cache, compute_hash, update_cache, CacheEntry, FileCache};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::Path;

fn create_temp_file(dir: &std::path::Path, name: &str, content: &str) -> std::path::PathBuf {
    let path = dir.join(name);
    let mut file = fs::File::create(&path).unwrap();
    file.write_all(content.as_bytes()).unwrap();
    path
}

#[test]
fn test_cache_new() {
    let cache = FileCache::new();
    assert!(cache.entries.is_empty());
    assert_eq!(cache.version, "2.0");
}

#[test]
fn test_cache_add_entry() {
    let mut cache = FileCache::new();
    cache.add_entry(CacheEntry {
        path: "test.py".into(),
        hash: "abc123".into(),
        size: 100,
        modified: 0,
        source_file: Some("main.py".into()),
    });
    assert_eq!(cache.entries.len(), 1);
    assert!(cache.by_source_file.contains_key("main.py"));
}

#[test]
fn test_cache_remove_entry() {
    let mut cache = FileCache::new();
    cache.add_entry(CacheEntry {
        path: "test.py".into(),
        hash: "abc123".into(),
        size: 100,
        modified: 0,
        source_file: Some("main.py".into()),
    });
    cache.remove_entries(&["test.py".into()]);
    assert!(cache.entries.is_empty());
}

#[test]
fn test_cache_save_load() {
    let tmp_dir = tempfile::tempdir().unwrap();
    let cache_path = tmp_dir.path().join("cache.json");

    let mut cache = FileCache::new();
    cache.add_entry(CacheEntry {
        path: "file1.py".into(),
        hash: "hash1".into(),
        size: 100,
        modified: 0,
        source_file: None,
    });

    cache.save(&cache_path).unwrap();
    let loaded = FileCache::load(&cache_path).unwrap();

    assert_eq!(loaded.entries.len(), 1);
    assert_eq!(loaded.entries.get("file1.py").unwrap().hash, "hash1");
}

#[test]
fn test_compute_hash_deterministic() {
    let tmp_dir = tempfile::tempdir().unwrap();
    let file_path = create_temp_file(tmp_dir.path(), "test.txt", "hello world");

    let hash1 = compute_hash(&file_path).unwrap();
    let hash2 = compute_hash(&file_path).unwrap();

    assert_eq!(hash1, hash2);
    assert_eq!(hash1.len(), 64); // SHA256 hex
}

#[test]
fn test_compute_hash_different_content() {
    let tmp_dir = tempfile::tempdir().unwrap();
    let file1 = create_temp_file(tmp_dir.path(), "a.txt", "content1");
    let file2 = create_temp_file(tmp_dir.path(), "b.txt", "content2");

    let hash1 = compute_hash(&file1).unwrap();
    let hash2 = compute_hash(&file2).unwrap();

    assert_ne!(hash1, hash2);
}

#[test]
fn test_md_body_only_hash() {
    let tmp_dir = tempfile::tempdir().unwrap();

    // MD with YAML frontmatter - only body should be hashed
    let md_path = tmp_dir.path().join("test.md");
    fs::write(&md_path, "---\ntitle: A\n---\n# Content").unwrap();
    let hash1 = compute_hash(&md_path).unwrap();

    // Change frontmatter, same body
    fs::write(&md_path, "---\ntitle: B\n---\n# Content").unwrap();
    let hash2 = compute_hash(&md_path).unwrap();

    assert_eq!(hash1, hash2, "Frontmatter change should not affect hash");
}

#[test]
fn test_check_cache_unchanged() {
    let tmp_dir = tempfile::tempdir().unwrap();
    let file_path = create_temp_file(tmp_dir.path(), "test.txt", "hello");

    let mut cache = FileCache::new();
    update_cache(&mut cache, &[file_path.clone()], None).unwrap();

    let (changed, unchanged) = check_cache(&[file_path.clone()], &cache);

    assert!(changed.is_empty());
    assert_eq!(unchanged.len(), 1);
}

#[test]
fn test_check_cache_changed() {
    let tmp_dir = tempfile::tempdir().unwrap();
    let file_path = create_temp_file(tmp_dir.path(), "test.txt", "hello");

    let mut cache = FileCache::new();
    update_cache(&mut cache, &[file_path.clone()], None).unwrap();

    // Modify file
    let mut file = fs::OpenOptions::new()
        .write(true)
        .append(true)
        .open(&file_path)
        .unwrap();
    file.write_all(b" world").unwrap();

    let (changed, unchanged) = check_cache(&[file_path.clone()], &cache);

    assert_eq!(changed.len(), 1);
    assert!(unchanged.is_empty());
}

#[test]
fn test_check_cache_new_file() {
    let tmp_dir = tempfile::tempdir().unwrap();
    let file_path = create_temp_file(tmp_dir.path(), "new.txt", "content");

    let cache = FileCache::new();
    let (changed, unchanged) = check_cache(&[file_path], &cache);

    assert_eq!(changed.len(), 1);
    assert!(unchanged.is_empty());
}

#[test]
fn test_group_by_source_file() {
    let tmp_dir = tempfile::tempdir().unwrap();
    let file1 = tmp_dir.path().join("mod/a.py");
    let file2 = tmp_dir.path().join("mod/b.py");
    fs::create_dir_all(file1.parent().unwrap()).unwrap();

    fs::write(&file1, "def foo(): pass").unwrap();
    fs::write(&file2, "def bar(): pass").unwrap();

    let mut cache = FileCache::new();
    update_cache(&mut cache, &[file1.clone(), file2], Some("mod")).unwrap();

    let files = cache.by_source_file.get("mod").unwrap();
    assert_eq!(files.len(), 2);
}
