//! End-to-end build and query tests

use garfield::{build_graph, detect, extract_file, score_nodes, find_shortest_path, analyze};
use std::path::Path;

#[test]
fn test_full_build_pipeline() {
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

    assert!(!code_files.is_empty(), "Should find code files");

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

    assert!(graph.nodes.len() > 0);
    assert!(graph.links.len() >= 0);

    let analysis = analyze(&graph);
    let community_count = analysis.community_sizes.len();
    println!("Nodes: {}, Edges: {}, Communities: {}",
        graph.nodes.len(), graph.links.len(), community_count);
}

#[test]
fn test_query_by_name() {
    let mut extraction = garfield::ExtractionResult::new();

    extraction.add_node(garfield::Node::new("test.py:hello".into(), "hello".into(), "test.py".into(), "L1".into()));
    extraction.add_node(garfield::Node::new("test.py:world".into(), "world".into(), "test.py".into(), "L2".into()));
    extraction.add_node(garfield::Node::new("test.py:foo".into(), "foo".into(), "test.py".into(), "L3".into()));

    let graph = build_graph(vec![extraction]);

    let terms = vec!["hello".to_string()];
    let scores = score_nodes(&graph, &terms);

    assert!(!scores.is_empty());
    assert_eq!(scores[0].1, "test.py:hello");
}

#[test]
fn test_path_finding() {
    let mut extraction = garfield::ExtractionResult::new();

    extraction.add_node(garfield::Node::new("a.py:A".into(), "A".into(), "a.py".into(), "L1".into()));
    extraction.add_node(garfield::Node::new("a.py:B".into(), "B".into(), "a.py".into(), "L2".into()));
    extraction.add_node(garfield::Node::new("a.py:C".into(), "C".into(), "a.py".into(), "L3".into()));
    extraction.add_node(garfield::Node::new("a.py:D".into(), "D".into(), "a.py".into(), "L4".into()));

    extraction.add_edge(garfield::Edge::new("a.py:A".into(), "a.py:B".into(), "calls".into(), garfield::Confidence::Extracted));
    extraction.add_edge(garfield::Edge::new("a.py:B".into(), "a.py:C".into(), "calls".into(), garfield::Confidence::Extracted));
    extraction.add_edge(garfield::Edge::new("a.py:C".into(), "a.py:D".into(), "calls".into(), garfield::Confidence::Extracted));

    let graph = build_graph(vec![extraction]);

    let path = find_shortest_path(&graph, "a.py:A", "a.py:D", 10);
    assert!(path.is_some());
    assert!(path.unwrap().len() <= 4);
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
}
