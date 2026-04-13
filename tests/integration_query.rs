//! Query and analyze integration tests

use garfield::types::{Confidence, Edge, ExtractionResult, Node};
use garfield::{
    analyze, build_graph, find_god_nodes, find_shortest_path, find_surprising_connections,
    score_nodes,
};

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

// ===== Score Nodes Tests =====

#[test]
fn test_score_nodes_exact_match() {
    let mut extraction = ExtractionResult::new();
    extraction.add_node(node("hello", "test.py"));
    extraction.add_node(node("world", "test.py"));

    let graph = build_graph(vec![extraction]);
    let terms = vec!["hello".to_string()];
    let scores = score_nodes(&graph, &terms);

    assert!(!scores.is_empty());
    assert_eq!(scores[0].1, "hello");
}

#[test]
fn test_score_nodes_partial_match() {
    let mut extraction = ExtractionResult::new();
    extraction.add_node(node("calculate_price", "test.py"));
    extraction.add_node(node("calculate_total", "test.py"));

    let graph = build_graph(vec![extraction]);
    let terms = vec!["calculate".to_string()];
    let scores = score_nodes(&graph, &terms);

    assert!(scores.len() >= 2);
}

#[test]
fn test_score_nodes_no_match() {
    let mut extraction = ExtractionResult::new();
    extraction.add_node(node("hello", "test.py"));

    let graph = build_graph(vec![extraction]);
    let terms = vec!["nonexistent".to_string()];
    let scores = score_nodes(&graph, &terms);

    assert!(scores.is_empty());
}

#[test]
fn test_score_nodes_case_insensitive() {
    let mut extraction = ExtractionResult::new();
    extraction.add_node(node("HELLO", "test.py"));

    let graph = build_graph(vec![extraction]);
    let terms = vec!["hello".to_string()];
    let scores = score_nodes(&graph, &terms);

    assert!(!scores.is_empty());
}

// ===== Shortest Path Tests =====

#[test]
fn test_find_path_direct() {
    let mut extraction = ExtractionResult::new();
    extraction.add_node(node("A", "test.rs"));
    extraction.add_node(node("B", "test.rs"));
    extraction.add_edge(edge("A", "B"));

    let graph = build_graph(vec![extraction]);
    let path = find_shortest_path(&graph, "A", "B", 10);

    assert!(path.is_some());
    assert_eq!(path.unwrap().len(), 2);
}

#[test]
fn test_find_path_chain() {
    let mut extraction = ExtractionResult::new();
    extraction.add_node(node("A", "test.rs"));
    extraction.add_node(node("B", "test.rs"));
    extraction.add_node(node("C", "test.rs"));
    extraction.add_node(node("D", "test.rs"));

    extraction.add_edge(edge("A", "B"));
    extraction.add_edge(edge("B", "C"));
    extraction.add_edge(edge("C", "D"));

    let graph = build_graph(vec![extraction]);
    let path = find_shortest_path(&graph, "A", "D", 10);

    assert!(path.is_some());
    assert_eq!(path.unwrap().len(), 4);
}

#[test]
fn test_find_path_no_path() {
    let mut extraction = ExtractionResult::new();
    extraction.add_node(node("A", "f1.rs"));
    extraction.add_node(node("B", "f2.rs"));

    let graph = build_graph(vec![extraction]);
    let path = find_shortest_path(&graph, "A", "B", 10);

    assert!(path.is_none());
}

#[test]
fn test_find_path_not_exists() {
    let mut extraction = ExtractionResult::new();
    extraction.add_node(node("A", "test.rs"));

    let graph = build_graph(vec![extraction]);
    let path = find_shortest_path(&graph, "A", "NonExistent", 10);

    assert!(path.is_none());
}

// ===== God Nodes Tests =====

#[test]
fn test_find_god_nodes_basic() {
    let mut extraction = ExtractionResult::new();
    extraction.add_node(node("A", "test.rs"));
    extraction.add_node(node("B", "test.rs"));
    extraction.add_node(node("C", "test.rs"));

    extraction.add_edge(edge("A", "B"));
    extraction.add_edge(edge("A", "C"));
    extraction.add_edge(edge("B", "C"));

    let graph = build_graph(vec![extraction]);
    let gods = find_god_nodes(&graph, 2);

    assert!(!gods.is_empty());
}

#[test]
fn test_find_god_nodes_hub() {
    let mut extraction = ExtractionResult::new();

    extraction.add_node(node("Hub", "test.rs"));
    for i in 1..=5 {
        extraction.add_node(node(&format!("Leaf{}", i), "test.rs"));
        extraction.add_edge(edge("Hub", &format!("Leaf{}", i)));
    }

    let graph = build_graph(vec![extraction]);
    let gods = find_god_nodes(&graph, 1);

    assert_eq!(gods[0].node.label, "Hub");
}

#[test]
fn test_find_god_nodes_none() {
    let mut extraction = ExtractionResult::new();
    extraction.add_node(node("A", "test.rs"));

    let graph = build_graph(vec![extraction]);
    let gods = find_god_nodes(&graph, 5);

    assert!(gods.is_empty() || gods[0].degree == 0);
}

// ===== Analyze Tests =====

#[test]
fn test_analyze_confidence_stats() {
    let mut extraction = ExtractionResult::new();
    extraction.add_node(node("A", "test.rs"));
    extraction.add_node(node("B", "test.rs"));

    extraction.add_edge(edge("A", "B"));
    extraction.add_edge(Edge::new(
        "A".into(),
        "B".into(),
        "imports".into(),
        Confidence::Inferred,
    ));
    extraction.add_edge(Edge::new(
        "A".into(),
        "B".into(),
        "uses".into(),
        Confidence::Ambiguous,
    ));

    let graph = build_graph(vec![extraction]);
    let analysis = analyze(&graph);

    assert_eq!(analysis.confidence_stats.extracted, 1);
    assert_eq!(analysis.confidence_stats.inferred, 1);
    assert_eq!(analysis.confidence_stats.ambiguous, 1);
}

#[test]
fn test_analyze_community_stats() {
    let mut extraction = ExtractionResult::new();
    extraction.add_node(node("A", "f1.rs"));
    extraction.add_node(node("B", "f1.rs"));
    extraction.add_node(node("C", "f2.rs"));
    extraction.add_edge(edge("A", "B"));

    let graph = build_graph(vec![extraction]);
    let analysis = analyze(&graph);

    assert!(analysis.community_sizes.len() >= 1);
}

#[test]
fn test_analyze_empty_graph() {
    let graph = garfield::GraphData::new(vec![], vec![], 0);
    let analysis = analyze(&graph);

    assert_eq!(analysis.confidence_stats.extracted, 0);
    assert_eq!(analysis.confidence_stats.inferred, 0);
}

#[test]
fn test_analyze_node_stats() {
    let mut extraction = ExtractionResult::new();
    extraction.add_node(node("A", "test.rs"));
    extraction.add_node(node("B", "test.rs"));
    extraction.add_edge(edge("A", "B"));

    let graph = build_graph(vec![extraction]);
    let analysis = analyze(&graph);

    assert!(graph.nodes.len() >= 2);
}

// ===== Surprising Connections Tests =====

#[test]
fn test_find_surprising_connections_cross_file() {
    let mut extraction = ExtractionResult::new();
    extraction.add_node(node("A", "f1.rs"));
    extraction.add_node(node("B", "f2.rs"));
    extraction.add_edge(Edge::new(
        "A".into(),
        "B".into(),
        "calls".into(),
        Confidence::Inferred,
    ));

    let graph = build_graph(vec![extraction]);
    let _surprises = find_surprising_connections(&graph);
}

#[test]
fn test_find_surprising_connections_same_file() {
    let mut extraction = ExtractionResult::new();
    extraction.add_node(node("A", "test.rs"));
    extraction.add_node(node("B", "test.rs"));
    extraction.add_edge(edge("A", "B"));

    let graph = build_graph(vec![extraction]);
    let _surprises = find_surprising_connections(&graph);
}
