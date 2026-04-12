//! Analyze integration tests

use garfield::{analyze, build_graph, find_surprising_connections};

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

#[test]
fn test_surprising_connections() {
    let mut extraction = garfield::ExtractionResult::new();

    extraction.add_node(garfield::Node::new("file1.py:A".into(), "A".into(), "file1.py".into(), "L1".into()));
    extraction.add_node(garfield::Node::new("file2.py:B".into(), "B".into(), "file2.py".into(), "L1".into()));

    extraction.add_edge(garfield::Edge::new("file1.py:A".into(), "file2.py:B".into(), "calls".into(), garfield::Confidence::Inferred));

    let graph = build_graph(vec![extraction]);
    let _surprises = find_surprising_connections(&graph);
}
