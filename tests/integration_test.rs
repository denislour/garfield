//! Integration tests for garfield

use std::path::Path;
use garfield::{
    detect, extract_file, build_graph, cluster, analyze, 
    find_god_nodes, find_shortest_path, score_nodes,
    FileCache, validate_graph,
};

#[test]
fn test_end_to_end_build() {
    // Test that we can build a graph from real Python files
    let test_dir = Path::new("../graphify");
    if !test_dir.exists() {
        println!("Skipping integration test - test directory not found");
        return;
    }
    
    // 1. Detect files
    let files = detect(test_dir).expect("detect should work");
    let code_files: Vec<_> = files.into_iter()
        .filter(|f| f.file_type == garfield::FileType::Code)
        .collect();
    
    assert!(!code_files.is_empty(), "Should find some code files");
    
    // 2. Extract from a Python file
    let sample_file = code_files.first().expect("should have a file");
    let source = std::fs::read_to_string(&sample_file.path)
        .expect("should read file");
    
    let extraction = extract_file(&sample_file.path, &source)
        .expect("extraction should work");
    
    println!("Extracted {} nodes, {} edges", 
        extraction.nodes.len(), extraction.edges.len());
    
    // 3. Build graph
    let graph = build_graph(vec![extraction]);
    
    // 4. Validate
    let result = validate_graph(&graph);
    if let Err(e) = &result {
        println!("Validation warning: {:?}", e);
    }
    
    // 5. Analyze
    let gods = find_god_nodes(&graph, 5);
    println!("Top god nodes:");
    for god in &gods {
        println!("  {} - {} edges", god.node.label, god.degree);
    }
    
    assert!(graph.nodes.len() > 0, "Should have nodes");
}

#[test]
fn test_cache_functionality() {
    use garfield::cache::{CacheEntry, compute_hash};
    use std::collections::HashMap;
    
    let cache_path = Path::new("/tmp/test_garfield_cache.json");
    
    // Create a cache with entries
    let mut entries = HashMap::new();
    entries.insert("file1.py".to_string(), CacheEntry {
        path: "file1.py".to_string(),
        hash: "hash1".to_string(),
        size: 100,
        modified: 0,
    });
    entries.insert("file2.py".to_string(), CacheEntry {
        path: "file2.py".to_string(),
        hash: "hash2".to_string(),
        size: 200,
        modified: 0,
    });
    
    let cache = FileCache {
        entries,
        version: "1.0".to_string(),
    };
    
    // Save and load
    cache.save(cache_path).expect("should save");
    
    let loaded = FileCache::load(cache_path).expect("should load");
    assert_eq!(loaded.entries.len(), 2);
    
    // Cleanup
    let _ = std::fs::remove_file(cache_path);
}

#[test]
fn test_clustering() {
    let mut extraction = garfield::ExtractionResult::new();
    
    // Add nodes
    extraction.add_node(garfield::Node::new(
        "a.py:A".into(), "A".into(), "a.py".into(), "L1".into()
    ));
    extraction.add_node(garfield::Node::new(
        "a.py:B".into(), "B".into(), "a.py".into(), "L2".into()
    ));
    extraction.add_node(garfield::Node::new(
        "b.py:C".into(), "C".into(), "b.py".into(), "L1".into()
    ));
    
    // Add edges (A -> B, B -> C)
    extraction.add_edge(garfield::Edge::new(
        "a.py:A".into(), "a.py:B".into(), "calls".into(), garfield::Confidence::Extracted
    ));
    extraction.add_edge(garfield::Edge::new(
        "a.py:B".into(), "b.py:C".into(), "calls".into(), garfield::Confidence::Inferred
    ));
    
    let graph = build_graph(vec![extraction]);
    let result = cluster(&graph);
    
    assert!(result.assignments.len() > 0, "should have community assignments");
    assert_eq!(result.assignments.len(), graph.nodes.len());
}

#[test]
fn test_query_score() {
    let mut extraction = garfield::ExtractionResult::new();
    
    extraction.add_node(garfield::Node::new(
        "test.py:hello".into(), "hello".into(), "test.py".into(), "L1".into()
    ));
    extraction.add_node(garfield::Node::new(
        "test.py:world".into(), "world".into(), "test.py".into(), "L2".into()
    ));
    extraction.add_node(garfield::Node::new(
        "test.py:foo".into(), "foo".into(), "test.py".into(), "L3".into()
    ));
    
    let graph = build_graph(vec![extraction]);
    
    // Search for "hello"
    let terms = vec!["hello".to_string()];
    let scores = score_nodes(&graph, &terms);
    assert!(!scores.is_empty(), "should find matching nodes");
    assert_eq!(scores[0].1, "test.py:hello", "hello should be top match");
}

#[test]
fn test_shortest_path() {
    let mut extraction = garfield::ExtractionResult::new();
    
    extraction.add_node(garfield::Node::new("a.py:A".into(), "A".into(), "a.py".into(), "L1".into()));
    extraction.add_node(garfield::Node::new("a.py:B".into(), "B".into(), "a.py".into(), "L2".into()));
    extraction.add_node(garfield::Node::new("a.py:C".into(), "C".into(), "a.py".into(), "L3".into()));
    extraction.add_node(garfield::Node::new("a.py:D".into(), "D".into(), "a.py".into(), "L4".into()));
    
    // A -> B -> C -> D chain
    extraction.add_edge(garfield::Edge::new("a.py:A".into(), "a.py:B".into(), "calls".into(), garfield::Confidence::Extracted));
    extraction.add_edge(garfield::Edge::new("a.py:B".into(), "a.py:C".into(), "calls".into(), garfield::Confidence::Extracted));
    extraction.add_edge(garfield::Edge::new("a.py:C".into(), "a.py:D".into(), "calls".into(), garfield::Confidence::Extracted));
    
    let graph = build_graph(vec![extraction]);
    
    // Find path from A to D
    let path = find_shortest_path(&graph, "a.py:A", "a.py:D", 10);
    assert!(path.is_some(), "should find path");
    let path = path.unwrap();
    assert!(path.len() <= 4, "path should be A->B->C->D");
}

#[test]
fn test_confidence_stats() {
    let mut extraction = garfield::ExtractionResult::new();
    
    extraction.add_node(garfield::Node::new("a.py:A".into(), "A".into(), "a.py".into(), "L1".into()));
    extraction.add_node(garfield::Node::new("a.py:B".into(), "B".into(), "a.py".into(), "L1".into()));
    
    extraction.add_edge(garfield::Edge::new("a.py:A".into(), "a.py:B".into(), "calls".into(), garfield::Confidence::Extracted));
    extraction.add_edge(garfield::Edge::new("a.py:A".into(), "a.py:B".into(), "imports".into(), garfield::Confidence::Inferred));
    
    let graph = build_graph(vec![extraction]);
    let analysis = analyze(&graph);
    
    assert_eq!(analysis.confidence_stats.extracted, 1);
    assert_eq!(analysis.confidence_stats.inferred, 1);
    assert_eq!(analysis.confidence_stats.ambiguous, 0);
}

#[test]
fn test_surprising_connections() {
    let mut extraction = garfield::ExtractionResult::new();
    
    extraction.add_node(garfield::Node::new("file1.py:A".into(), "A".into(), "file1.py".into(), "L1".into()));
    extraction.add_node(garfield::Node::new("file2.py:B".into(), "B".into(), "file2.py".into(), "L1".into()));
    extraction.add_node(garfield::Node::new("file3.py:C".into(), "C".into(), "file3.py".into(), "L1".into()));
    
    // Add edges
    extraction.add_edge(garfield::Edge::new("file1.py:A".into(), "file2.py:B".into(), "calls".into(), garfield::Confidence::Inferred));
    
    let graph = build_graph(vec![extraction]);
    let surprises = garfield::find_surprising_connections(&graph);
    
    // Should have at least one cross-community edge
    println!("Surprising connections: {}", surprises.len());
}

#[test]
fn test_file_detection() {
    let test_dir = Path::new("../graphify");
    if !test_dir.exists() {
        return;
    }
    
    let files = detect(test_dir).expect("detect should work");
    
    // Count by type
    let code = files.iter().filter(|f| f.file_type == garfield::FileType::Code).count();
    let markdown = files.iter().filter(|f| f.file_type == garfield::FileType::Markdown).count();
    
    println!("Detected: {} code, {} markdown files", code, markdown);
    assert!(code > 0, "should detect code files");
}
