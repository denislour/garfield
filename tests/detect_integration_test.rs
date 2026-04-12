//! Detect integration tests

use garfield::detect;
use std::path::Path;

#[test]
fn test_file_detection() {
    let test_dir = Path::new("../graphify");
    if !test_dir.exists() {
        println!("Skipping - test directory not found");
        return;
    }

    let files = detect(test_dir).expect("detect should work");

    let code = files.files
        .iter()
        .filter(|f| f.file_type == garfield::FileType::Code)
        .count();
    let markdown = files.files
        .iter()
        .filter(|f| f.file_type == garfield::FileType::Markdown)
        .count();

    println!("Detected: {} code, {} markdown files", code, markdown);
    assert!(code > 0, "should detect code files");
}

#[test]
fn test_code_file_types() {
    use garfield::detect::classify_extension;

    assert_eq!(classify_extension("py"), garfield::FileType::Code);
    assert_eq!(classify_extension("rs"), garfield::FileType::Code);
    assert_eq!(classify_extension("ts"), garfield::FileType::Code);
    assert_eq!(classify_extension("js"), garfield::FileType::Code);
    assert_eq!(classify_extension("rb"), garfield::FileType::Code);
}

#[test]
fn test_markdown_file_types() {
    use garfield::detect::classify_extension;

    assert_eq!(classify_extension("md"), garfield::FileType::Markdown);
}

#[test]
fn test_binary_files_filtered() {
    use garfield::detect::classify_extension;

    assert_eq!(classify_extension("png"), garfield::FileType::Binary);
    assert_eq!(classify_extension("jpg"), garfield::FileType::Binary);
}
