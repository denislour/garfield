//! Cluster + Leiden tests

use garfield::{build_graph, cluster};

#[test]
fn test_cluster_triangle() {
    let mut e = garfield::ExtractionResult::new();
    e.add_node(garfield::Node::new("tri:A".into(), "A".into(), "tri.rs".into(), "L1".into()));
    e.add_node(garfield::Node::new("tri:B".into(), "B".into(), "tri.rs".into(), "L2".into()));
    e.add_node(garfield::Node::new("tri:C".into(), "C".into(), "tri.rs".into(), "L3".into()));
    e.add_edge(garfield::Edge::new("tri:A".into(), "tri:B".into(), "calls".into(), garfield::Confidence::Extracted));
    e.add_edge(garfield::Edge::new("tri:B".into(), "tri:C".into(), "calls".into(), garfield::Confidence::Extracted));
    e.add_edge(garfield::Edge::new("tri:C".into(), "tri:A".into(), "calls".into(), garfield::Confidence::Extracted));
    let g = build_graph(vec![e]);
    let r = cluster(&g);
    assert_eq!(r.assignments.len(), 3);
}

#[test]
fn test_cluster_two_disconnected() {
    let mut e = garfield::ExtractionResult::new();
    e.add_node(garfield::Node::new("c1:A".into(), "A".into(), "c1.rs".into(), "L1".into()));
    e.add_node(garfield::Node::new("c1:B".into(), "B".into(), "c1.rs".into(), "L2".into()));
    e.add_edge(garfield::Edge::new("c1:A".into(), "c1:B".into(), "calls".into(), garfield::Confidence::Extracted));
    e.add_node(garfield::Node::new("c2:C".into(), "C".into(), "c2.rs".into(), "L1".into()));
    e.add_node(garfield::Node::new("c2:D".into(), "D".into(), "c2.rs".into(), "L2".into()));
    e.add_edge(garfield::Edge::new("c2:C".into(), "c2:D".into(), "calls".into(), garfield::Confidence::Extracted));
    let g = build_graph(vec![e]);
    let r = cluster(&g);
    let unique: std::collections::HashSet<u32> = r.assignments.iter().cloned().collect();
    assert_eq!(unique.len(), 2);
}

#[test]
fn test_cluster_cross_file() {
    let mut e = garfield::ExtractionResult::new();
    e.add_node(garfield::Node::new("f1.rs:A".into(), "A".into(), "f1.rs".into(), "L1".into()));
    e.add_node(garfield::Node::new("f1.rs:B".into(), "B".into(), "f1.rs".into(), "L2".into()));
    e.add_edge(garfield::Edge::new("f1.rs:A".into(), "f1.rs:B".into(), "calls".into(), garfield::Confidence::Extracted));
    e.add_node(garfield::Node::new("f2.rs:C".into(), "C".into(), "f2.rs".into(), "L1".into()));
    e.add_node(garfield::Node::new("f2.rs:D".into(), "D".into(), "f2.rs".into(), "L2".into()));
    e.add_edge(garfield::Edge::new("f2.rs:C".into(), "f2.rs:D".into(), "calls".into(), garfield::Confidence::Extracted));
    e.add_edge(garfield::Edge::new("f1.rs:B".into(), "f2.rs:C".into(), "calls".into(), garfield::Confidence::Inferred));
    let g = build_graph(vec![e]);
    let r = cluster(&g);
    let unique: std::collections::HashSet<u32> = r.assignments.iter().cloned().collect();
    assert!(unique.len() >= 1 && unique.len() <= 2);
}
