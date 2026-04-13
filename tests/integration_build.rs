//! Build integration tests

use garfield::types::{Confidence, Edge, ExtractionResult, Node};
use garfield::{build_graph, merge_extractions, merge_into_graph};

fn node(id: &str, file: &str) -> Node {
    Node::new(id.into(), id.into(), file.into(), "L1".into())
}

fn edge(src: &str, tgt: &str) -> Edge {
    Edge::new(
        src.into(),
        tgt.into(),
        "calls".into(),
        Confidence::Extracted,
    )
}

#[test]
fn test_build_graph_empty() {
    let graph = build_graph(vec![]);
    assert!(graph.nodes.is_empty());
    assert!(graph.links.is_empty());
}

#[test]
fn test_build_graph_single_extraction() {
    let mut extraction = ExtractionResult::new();
    extraction.add_node(node("A", "test.rs"));
    extraction.add_node(node("B", "test.rs"));
    extraction.add_edge(edge("A", "B"));

    let graph = build_graph(vec![extraction]);

    assert_eq!(graph.nodes.len(), 2);
    assert_eq!(graph.links.len(), 1);
}

#[test]
fn test_build_graph_multiple_extractions() {
    let mut ext1 = ExtractionResult::new();
    ext1.add_node(node("A", "f1.rs"));
    ext1.add_node(node("B", "f1.rs"));
    ext1.add_edge(edge("A", "B"));

    let mut ext2 = ExtractionResult::new();
    ext2.add_node(node("C", "f2.rs"));
    ext2.add_node(node("D", "f2.rs"));
    ext2.add_edge(edge("C", "D"));

    let graph = build_graph(vec![ext1, ext2]);

    assert_eq!(graph.nodes.len(), 4);
    assert_eq!(graph.links.len(), 2);
}

#[test]
fn test_build_graph_deduplication() {
    let mut ext1 = ExtractionResult::new();
    ext1.add_node(node("A", "test.rs"));

    let mut ext2 = ExtractionResult::new();
    ext2.add_node(node("A", "test.rs")); // Duplicate

    let graph = build_graph(vec![ext1, ext2]);

    assert_eq!(graph.nodes.len(), 1);
}

#[test]
fn test_build_graph_with_imports() {
    let mut extraction = ExtractionResult::new();
    extraction.add_node(node("module_a:func_a", "module_a.py"));
    extraction.add_node(node("module_b:func_b", "module_b.py"));
    extraction.add_edge(Edge::new(
        "module_a:func_a".into(),
        "module_b:func_b".into(),
        "imports".into(),
        Confidence::Inferred,
    ));

    let graph = build_graph(vec![extraction]);

    assert_eq!(graph.nodes.len(), 2);
    assert_eq!(graph.links.len(), 1);
    assert_eq!(graph.links[0].relation, "imports");
}

#[test]
fn test_merge_extractions_basic() {
    let mut ext1 = ExtractionResult::new();
    ext1.add_node(node("A", "f1.rs"));

    let mut ext2 = ExtractionResult::new();
    ext2.add_node(node("B", "f2.rs"));

    let merged = merge_extractions(vec![ext1, ext2]);

    assert_eq!(merged.nodes.len(), 2);
}

#[test]
fn test_merge_extractions_duplicate_nodes() {
    let mut ext1 = ExtractionResult::new();
    ext1.add_node(node("A", "test.rs"));

    let mut ext2 = ExtractionResult::new();
    ext2.add_node(node("A", "test.rs")); // Same ID

    let merged = merge_extractions(vec![ext1, ext2]);

    assert_eq!(merged.nodes.len(), 1);
}

#[test]
fn test_merge_extractions_duplicate_edges() {
    let mut ext1 = ExtractionResult::new();
    ext1.add_edge(edge("A", "B"));

    let mut ext2 = ExtractionResult::new();
    ext2.add_edge(edge("A", "B")); // Same edge

    let merged = merge_extractions(vec![ext1, ext2]);

    assert_eq!(merged.links.len(), 1);
}

#[test]
fn test_merge_into_graph_new_nodes() {
    let mut existing = garfield::GraphData::new(vec![], vec![], 0);
    existing.nodes.push(node("A", "f1.rs"));

    let mut extraction = ExtractionResult::new();
    extraction.add_node(node("B", "f2.rs"));

    merge_into_graph(&mut existing, extraction);

    assert_eq!(existing.nodes.len(), 2);
}

#[test]
fn test_merge_into_graph_new_edges() {
    let mut existing = garfield::GraphData::new(vec![], vec![], 0);
    existing.nodes.push(node("A", "f1.rs"));
    existing.nodes.push(node("B", "f2.rs"));

    let mut extraction = ExtractionResult::new();
    extraction.add_edge(edge("A", "B"));

    merge_into_graph(&mut existing, extraction);

    assert_eq!(existing.links.len(), 1);
}

#[test]
fn test_merge_into_graph_updates_metadata() {
    let mut existing = garfield::GraphData::new(vec![], vec![], 0);
    existing.nodes.push(node("A", "test.rs"));
    existing.metadata.total_nodes = 1;

    let mut extraction = ExtractionResult::new();
    extraction.add_node(node("B", "test.rs"));

    merge_into_graph(&mut existing, extraction);

    assert!(existing.metadata.total_nodes >= 2);
}

#[test]
fn test_build_graph_maintains_metadata() {
    let mut extraction = ExtractionResult::new();
    extraction.add_node(node("A", "test.rs"));
    extraction.add_node(node("B", "test.rs"));
    extraction.add_edge(edge("A", "B"));

    let graph = build_graph(vec![extraction]);

    assert!(graph.metadata.total_nodes >= 2);
    assert!(graph.metadata.total_edges >= 1);
}
