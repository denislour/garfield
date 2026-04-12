//! Community detection integration tests

use garfield::{build_graph, cluster};

#[test]
fn test_clustering() {
    let mut extraction = garfield::ExtractionResult::new();

    extraction.add_node(garfield::Node::new("a.py:A".into(), "A".into(), "a.py".into(), "L1".into()));
    extraction.add_node(garfield::Node::new("a.py:B".into(), "B".into(), "a.py".into(), "L2".into()));
    extraction.add_node(garfield::Node::new("b.py:C".into(), "C".into(), "b.py".into(), "L1".into()));

    extraction.add_edge(garfield::Edge::new("a.py:A".into(), "a.py:B".into(), "calls".into(), garfield::Confidence::Extracted));
    extraction.add_edge(garfield::Edge::new("a.py:B".into(), "b.py:C".into(), "calls".into(), garfield::Confidence::Inferred));

    let graph = build_graph(vec![extraction]);
    let result = cluster(&graph);

    assert!(result.assignments.len() > 0);
    assert_eq!(result.assignments.len(), graph.nodes.len());
}
