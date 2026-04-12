//! Detect integration tests

use garfield::detect::{classify_extension, filter_code_files, get_stats};
use garfield::DetectedFile;

fn detected_file(path: &str, ext: &str) -> DetectedFile {
    DetectedFile {
        path: std::path::PathBuf::from(path),
        file_type: classify_extension(ext),
        extension: ext.to_string(),
        size_bytes: 100,
    }
}

#[test]
fn test_classify_rust() {
    assert_eq!(classify_extension("rs"), garfield::FileType::Code);
}

#[test]
fn test_classify_python() {
    assert_eq!(classify_extension("py"), garfield::FileType::Code);
}

#[test]
fn test_classify_typescript() {
    assert_eq!(classify_extension("ts"), garfield::FileType::Code);
    assert_eq!(classify_extension("tsx"), garfield::FileType::Code);
}

#[test]
fn test_classify_javascript() {
    assert_eq!(classify_extension("js"), garfield::FileType::Code);
    assert_eq!(classify_extension("jsx"), garfield::FileType::Code);
}

#[test]
fn test_classify_java() {
    assert_eq!(classify_extension("java"), garfield::FileType::Code);
}

#[test]
fn test_classify_go() {
    assert_eq!(classify_extension("go"), garfield::FileType::Code);
}

#[test]
fn test_classify_ruby() {
    assert_eq!(classify_extension("rb"), garfield::FileType::Code);
}

#[test]
fn test_classify_c() {
    assert_eq!(classify_extension("c"), garfield::FileType::Code);
    assert_eq!(classify_extension("h"), garfield::FileType::Code);
}

#[test]
fn test_classify_cpp() {
    assert_eq!(classify_extension("cpp"), garfield::FileType::Code);
    assert_eq!(classify_extension("hpp"), garfield::FileType::Code);
    assert_eq!(classify_extension("cc"), garfield::FileType::Code);
}

#[test]
fn test_classify_csharp() {
    assert_eq!(classify_extension("cs"), garfield::FileType::Code);
}

#[test]
fn test_classify_markdown() {
    assert_eq!(classify_extension("md"), garfield::FileType::Markdown);
    assert_eq!(classify_extension("markdown"), garfield::FileType::Markdown);
}

#[test]
fn test_classify_binary() {
    assert_eq!(classify_extension("png"), garfield::FileType::Binary);
    assert_eq!(classify_extension("jpg"), garfield::FileType::Binary);
    assert_eq!(classify_extension("gif"), garfield::FileType::Binary);
    assert_eq!(classify_extension("pdf"), garfield::FileType::Binary);
    assert_eq!(classify_extension("zip"), garfield::FileType::Binary);
    assert_eq!(classify_extension("exe"), garfield::FileType::Binary);
    assert_eq!(classify_extension("so"), garfield::FileType::Binary);
    assert_eq!(classify_extension("dll"), garfield::FileType::Binary);
}

#[test]
fn test_classify_empty() {
    assert_eq!(classify_extension(""), garfield::FileType::Binary);
}

#[test]
fn test_filter_code_files_basic() {
    let files = vec![
        detected_file("test.py", "py"),
        detected_file("readme.md", "md"),
        detected_file("image.png", "png"),
        detected_file("main.rs", "rs"),
    ];
    
    let code_files = filter_code_files(&files);
    
    assert_eq!(code_files.len(), 2);
    assert!(code_files.iter().any(|f| f.path.to_string_lossy().contains("test.py")));
    assert!(code_files.iter().any(|f| f.path.to_string_lossy().contains("main.rs")));
    assert!(!code_files.iter().any(|f| f.path.to_string_lossy().contains("readme.md")));
}

#[test]
fn test_filter_code_files_empty() {
    let files = vec![
        detected_file("image.png", "png"),
        detected_file("doc.pdf", "pdf"),
    ];
    
    let code_files = filter_code_files(&files);
    assert!(code_files.is_empty());
}

#[test]
fn test_get_stats() {
    let files = vec![
        detected_file("test.py", "py"),
        detected_file("main.rs", "rs"),
        detected_file("readme.md", "md"),
    ];
    
    let stats = get_stats(&files);
    
    assert_eq!(stats.total, 3);
    assert_eq!(stats.code, 2);
    assert_eq!(stats.markdown, 1);
}

#[test]
fn test_detect_real_directory() {
    let test_dir = std::path::Path::new("../graphify");
    if !test_dir.exists() {
        println!("SKIP: graphify directory not found");
        return;
    }
    
    let result = garfield::detect::detect(test_dir).unwrap();
    assert!(!result.files.is_empty());
    
    let code_count = result.files.iter()
        .filter(|f| f.file_type == garfield::FileType::Code)
        .count();
    assert!(code_count > 0, "Should detect code files");
}
