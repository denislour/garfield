//! Query integration tests

use garfield::{build_graph, find_shortest_path, score_nodes};

#[test]
fn test_query_score() {
    let mut extraction = garfield::ExtractionResult::new();

    extraction.add_node(garfield::Node::new("test.py:hello".into(), "hello".into(), "test.py".into(), "L1".into()));
    extraction.add_node(garfield::Node::new("test.py:world".into(), "world".into(), "test.py".into(), "L2".into()));

    let graph = build_graph(vec![extraction]);
    let terms = vec!["hello".to_string()];
    let scores = score_nodes(&graph, &terms);

    assert!(!scores.is_empty());
    assert_eq!(scores[0].1, "test.py:hello");
}

#[test]
fn test_shortest_path() {
    let mut extraction = garfield::ExtractionResult::new();

    extraction.add_node(garfield::Node::new("a.py:A".into(), "A".into(), "a.py".into(), "L1".into()));
    extraction.add_node(garfield::Node::new("a.py:B".into(), "B".into(), "a.py".into(), "L2".into()));
    extraction.add_node(garfield::Node::new("a.py:C".into(), "C".into(), "a.py".into(), "L3".into()));

    extraction.add_edge(garfield::Edge::new("a.py:A".into(), "a.py:B".into(), "calls".into(), garfield::Confidence::Extracted));
    extraction.add_edge(garfield::Edge::new("a.py:B".into(), "a.py:C".into(), "calls".into(), garfield::Confidence::Extracted));

    let graph = build_graph(vec![extraction]);
    let path = find_shortest_path(&graph, "a.py:A", "a.py:C", 10);

    assert!(path.is_some());
    assert!(path.unwrap().len() <= 3);
}
