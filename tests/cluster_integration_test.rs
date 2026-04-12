//! Cluster/Community integration tests

use garfield::{build_graph, cluster};

#[test]
fn test_clustering_basic() {
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

#[test]
fn test_leiden_triangle() {
    let mut extraction = garfield::ExtractionResult::new();

    extraction.add_node(garfield::Node::new("c1:A".into(), "A".into(), "c1.rs".into(), "L1".into()));
    extraction.add_node(garfield::Node::new("c1:B".into(), "B".into(), "c1.rs".into(), "L1".into()));
    extraction.add_node(garfield::Node::new("c1:C".into(), "C".into(), "c1.rs".into(), "L1".into()));

    extraction.add_edge(garfield::Edge::new("c1:A".into(), "c1:B".into(), "calls".into(), garfield::Confidence::Extracted));
    extraction.add_edge(garfield::Edge::new("c1:B".into(), "c1:C".into(), "calls".into(), garfield::Confidence::Extracted));
    extraction.add_edge(garfield::Edge::new("c1:A".into(), "c1:C".into(), "calls".into(), garfield::Confidence::Extracted));

    let graph = build_graph(vec![extraction]);
    let result = cluster(&graph);

    assert!(result.assignments.len() == 3);
}

#[test]
fn test_two_disconnected_cliques() {
    let mut extraction = garfield::ExtractionResult::new();

    // Clique 1
    extraction.add_node(garfield::Node::new("f1:A".into(), "A".into(), "f1.rs".into(), "L1".into()));
    extraction.add_node(garfield::Node::new("f1:B".into(), "B".into(), "f1.rs".into(), "L1".into()));
    extraction.add_edge(garfield::Edge::new("f1:A".into(), "f1:B".into(), "calls".into(), garfield::Confidence::Extracted));

    // Clique 2
    extraction.add_node(garfield::Node::new("f2:C".into(), "C".into(), "f2.rs".into(), "L1".into()));
    extraction.add_node(garfield::Node::new("f2:D".into(), "D".into(), "f2.rs".into(), "L1".into()));
    extraction.add_edge(garfield::Edge::new("f2:C".into(), "f2:D".into(), "calls".into(), garfield::Confidence::Extracted));

    let graph = build_graph(vec![extraction]);
    let result = cluster(&graph);

    // Should have 2 communities
    let unique: std::collections::HashSet<u32> = result.assignments.iter().cloned().collect();
    assert!(unique.len() >= 2);
}
