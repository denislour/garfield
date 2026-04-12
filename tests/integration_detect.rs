//! Detect integration tests

use garfield::detect;
use std::path::Path;

#[test]
fn test_file_detection() {
    let test_dir = Path::new("../graphify");
    if !test_dir.exists() {
        return;
    }

    let files = detect(test_dir).expect("detect should work");
    let code = files.files
        .iter()
        .filter(|f| f.file_type == garfield::FileType::Code)
        .count();

    assert!(code > 0, "should detect code files");
}
