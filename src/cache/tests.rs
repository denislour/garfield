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
    fn test_md_body_only_hash() {
        let dir = TempDir::new().unwrap();
        
        // Create MD file with YAML frontmatter
        let md_path = dir.path().join("test.md");
        let md_content = r#"---
title: My Document
tags: [foo, bar]
---
# Actual content

This is the body.
"#;
        fs::write(&md_path, md_content).unwrap();
        
        let hash1 = compute_hash(&md_path).unwrap();
        
        // Change only metadata
        let md_content_changed = r#"---
title: Changed Title
tags: [different]
---
# Actual content

This is the body.
"#;
        fs::write(&md_path, md_content_changed).unwrap();
        
        let hash2 = compute_hash(&md_path).unwrap();
        
        // Hashes should be SAME because only frontmatter changed (body-only hashing)
        assert_eq!(hash1, hash2);
        
        // Change only body
        let md_body_changed = r#"---
title: My Document
tags: [foo, bar]
---
# Actual content

This is the body - MODIFIED.
"#;
        fs::write(&md_path, md_body_changed).unwrap();
        
        let hash3 = compute_hash(&md_path).unwrap();
        
        // Hash should change when body changes
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_cache_roundtrip() {
        let mut cache = FileCache::new();

        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("test.txt");

        let mut file = fs::File::create(&file_path).unwrap();
        file.write_all(b"hello").unwrap();

        update_cache(&mut cache, &[file_path.clone()], Some("test.txt")).unwrap();

        let (changed, unchanged) = check_cache(&[file_path.clone()], &cache);

        assert_eq!(changed.len(), 0);
        assert_eq!(unchanged.len(), 1);
    }
    
    #[test]
    fn test_group_by_source_file() {
        let mut cache = FileCache::new();
        
        let dir = TempDir::new().unwrap();
        let file1 = dir.path().join("module1").join("a.py");
        let file2 = dir.path().join("module1").join("b.py");
        let file3 = dir.path().join("module2").join("c.py");
        
        fs::create_dir_all(file1.parent().unwrap()).unwrap();
        fs::create_dir_all(file3.parent().unwrap()).unwrap();
        
        fs::write(&file1, "def foo(): pass").unwrap();
        fs::write(&file2, "def bar(): pass").unwrap();
        fs::write(&file3, "def baz(): pass").unwrap();
        
        update_cache(&mut cache, &[file1.clone(), file2.clone(), file3.clone()], Some("module1")).unwrap();
        update_cache(&mut cache, &[file3.clone()], Some("module2")).unwrap();
        
        // Check by_source_file indexing
        // First call adds all 3 files to module1, second call adds file3 to module2
        assert_eq!(cache.by_source_file.get("module1").map(|v| v.len()), Some(3));
        assert_eq!(cache.by_source_file.get("module2").map(|v| v.len()), Some(1));
    }
}
