//! Detect tests

use garfield::detect;
use garfield::detect::classify_extension;
use std::path::Path;

#[test]
fn test_detect_real_dir() {
    let dir = Path::new("../graphify");
    if !dir.exists() { println!("SKIP"); return; }
    let r = detect::detect(dir).unwrap();
    assert!(!r.files.is_empty());
}

#[test]
fn test_classify_code() {
    assert_eq!(classify_extension("py"), garfield::FileType::Code);
    assert_eq!(classify_extension("rs"), garfield::FileType::Code);
    assert_eq!(classify_extension("ts"), garfield::FileType::Code);
    assert_eq!(classify_extension("js"), garfield::FileType::Code);
    assert_eq!(classify_extension("rb"), garfield::FileType::Code);
}

#[test]
fn test_classify_markdown() {
    assert_eq!(classify_extension("md"), garfield::FileType::Markdown);
}

#[test]
fn test_classify_binary() {
    assert_eq!(classify_extension("png"), garfield::FileType::Binary);
    assert_eq!(classify_extension("pdf"), garfield::FileType::Binary);
    assert_eq!(classify_extension("zip"), garfield::FileType::Binary);
}
