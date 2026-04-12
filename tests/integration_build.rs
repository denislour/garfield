//! Build integration tests

use garfield::{build_graph, detect, extract_file, validate_graph};
use std::path::Path;

#[test]
fn test_end_to_end_build() {
    let test_dir = Path::new("../graphify");
    if !test_dir.exists() {
        println!("Skipping - test directory not found");
        return;
    }

    let files = detect(test_dir).expect("detect should work");
    let code_files: Vec<_> = files.files
        .into_iter()
        .filter(|f| f.file_type == garfield::FileType::Code)
        .collect();

    assert!(!code_files.is_empty(), "Should find some code files");

    let mut extraction = None;
    for sample_file in &code_files {
        if let Ok(source) = std::fs::read_to_string(&sample_file.path) {
            if let Ok(result) = extract_file(&sample_file.path, &source) {
                if !result.nodes.is_empty() {
                    extraction = Some(result);
                    break;
                }
            }
        }
    }

    let extraction = match extraction {
        Some(e) => e,
        None => return,
    };

    let graph = build_graph(vec![extraction]);
    let _ = validate_graph(&graph);
    assert!(graph.nodes.len() > 0);
}
