//! Detect integration tests

use garfield::detect;
use std::path::Path;

#[test]
fn test_detect_real_directory() {
    let test_dir = Path::new("../graphify");
    if !test_dir.exists() {
        println!("SKIP: graphify not found");
        return;
    }

    let result = detect::detect(test_dir).expect("detect should work");
    assert!(!result.files.is_empty());
}

#[test]
fn test_detect_code_extensions() {
    use garfield::detect::classify_extension;
    
    assert_eq!(classify_extension("py"), garfield::FileType::Code);
    assert_eq!(classify_extension("rs"), garfield::FileType::Code);
    assert_eq!(classify_extension("ts"), garfield::FileType::Code);
    assert_eq!(classify_extension("js"), garfield::FileType::Code);
}

#[test]
fn test_detect_markdown_extensions() {
    use garfield::detect::classify_extension;
    assert_eq!(classify_extension("md"), garfield::FileType::Markdown);
}

#[test]
fn test_detect_binary_extensions() {
    use garfield::detect::classify_extension;
    assert_eq!(classify_extension("png"), garfield::FileType::Binary);
    assert_eq!(classify_extension("pdf"), garfield::FileType::Binary);
}
