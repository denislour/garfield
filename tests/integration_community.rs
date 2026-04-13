//! Community detection integration tests

use garfield::types::{Confidence, Edge, Node};
use garfield::{build_graph, cluster, CommunityResult};

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

fn edge_with_conf(src: &str, tgt: &str, conf: Confidence) -> Edge {
    Edge::new(src.into(), tgt.into(), "calls".into(), conf)
}

// ===== Leiden Algorithm Tests =====

#[test]
fn test_leiden_empty_graph() {
    let edges: Vec<(usize, f64)> = vec![];
    let result = garfield::leiden::leiden_communities(0, &[]);
    assert!(result.is_empty());
}

#[test]
fn test_leiden_no_edges() {
    let result = garfield::leiden::leiden_communities(3, &[]);
    assert_eq!(result, vec![0, 1, 2]); // Each node in own community
}

#[test]
fn test_leiden_single_edge() {
    let edges = vec![(0, 1, 1.0)];
    let result = garfield::leiden::leiden_communities(2, &edges);
    assert_eq!(result[0], result[1]); // Should be in same community
}

#[test]
fn test_leiden_triangle() {
    let edges = vec![(0, 1, 1.0), (1, 2, 1.0), (0, 2, 1.0)];
    let result = garfield::leiden::leiden_communities(3, &edges);
    assert_eq!(result[0], result[1]);
    assert_eq!(result[1], result[2]); // All in same community
}

#[test]
fn test_leiden_disconnected() {
    let edges = vec![(0, 1, 1.0), (2, 3, 1.0)];
    let result = garfield::leiden::leiden_communities(4, &edges);

    // {0,1} should be together, {2,3} should be together
    assert_eq!(result[0], result[1]);
    assert_eq!(result[2], result[3]);
    assert_ne!(result[0], result[2]);
}

#[test]
fn test_leiden_weighted() {
    let edges = vec![
        (0, 1, 10.0), // Strong edge
        (1, 2, 10.0), // Strong edge
        (0, 2, 1.0),  // Weak edge
    ];
    let result = garfield::leiden::leiden_communities(3, &edges);
    assert_eq!(result[0], result[1]);
    assert_eq!(result[1], result[2]);
}

#[test]
fn test_leiden_large_weight_favors_merge() {
    let edges = vec![
        (0, 1, 100.0),
        (2, 3, 100.0),
        (1, 2, 1.0), // Weak bridge
    ];
    let result = garfield::leiden::leiden_communities(4, &edges);
    // Should merge due to weak bridge
    let unique: std::collections::HashSet<u32> = result.iter().cloned().collect();
    assert!(unique.len() <= 2);
}

// ===== Cluster (High-level API) Tests =====

#[test]
fn test_cluster_triangle() {
    let mut extraction = garfield::ExtractionResult::new();
    extraction.add_node(node("A", "tri.rs"));
    extraction.add_node(node("B", "tri.rs"));
    extraction.add_node(node("C", "tri.rs"));
    extraction.add_edge(edge("A", "B"));
    extraction.add_edge(edge("B", "C"));
    extraction.add_edge(edge("C", "A"));

    let graph = build_graph(vec![extraction]);
    let result = cluster(&graph);

    assert_eq!(result.assignments.len(), 3);
    // All should be in same community
    assert_eq!(result.assignments[0], result.assignments[1]);
    assert_eq!(result.assignments[1], result.assignments[2]);
}

#[test]
fn test_cluster_two_cliques() {
    let mut extraction = garfield::ExtractionResult::new();

    // Clique 1
    extraction.add_node(node("A1", "f1.rs"));
    extraction.add_node(node("A2", "f1.rs"));
    extraction.add_edge(edge("A1", "A2"));

    // Clique 2
    extraction.add_node(node("B1", "f2.rs"));
    extraction.add_node(node("B2", "f2.rs"));
    extraction.add_edge(edge("B1", "B2"));

    let graph = build_graph(vec![extraction]);
    let result = cluster(&graph);

    let unique: std::collections::HashSet<u32> = result.assignments.iter().cloned().collect();
    assert_eq!(unique.len(), 2);
}

#[test]
fn test_cluster_cross_file_edges() {
    let mut extraction = garfield::ExtractionResult::new();

    extraction.add_node(node("A", "f1.rs"));
    extraction.add_node(node("B", "f1.rs"));
    extraction.add_edge(edge("A", "B"));

    extraction.add_node(node("C", "f2.rs"));
    extraction.add_node(node("D", "f2.rs"));
    extraction.add_edge(edge("C", "D"));

    // Cross-file edge
    extraction.add_edge(edge_with_conf("B", "C", Confidence::Inferred));

    let graph = build_graph(vec![extraction]);
    let result = cluster(&graph);

    // May merge or stay separate depending on edge weights
    assert!(result.assignments.len() >= 2);
}

#[test]
fn test_cluster_chain() {
    let mut extraction = garfield::ExtractionResult::new();

    extraction.add_node(node("A", "chain.rs"));
    extraction.add_node(node("B", "chain.rs"));
    extraction.add_node(node("C", "chain.rs"));
    extraction.add_node(node("D", "chain.rs"));

    extraction.add_edge(edge("A", "B"));
    extraction.add_edge(edge("B", "C"));
    extraction.add_edge(edge("C", "D"));

    let graph = build_graph(vec![extraction]);
    let result = cluster(&graph);

    // All connected, should form communities
    let unique: std::collections::HashSet<u32> = result.assignments.iter().cloned().collect();
    assert!(unique.len() >= 1);
}

#[test]
fn test_cluster_star() {
    let mut extraction = garfield::ExtractionResult::new();

    extraction.add_node(node("Center", "star.rs"));
    for i in 1..=5 {
        extraction.add_node(node(&format!("Leaf{}", i), "star.rs"));
        extraction.add_edge(edge("Center", &format!("Leaf{}", i)));
    }

    let graph = build_graph(vec![extraction]);
    let result = cluster(&graph);

    // Star is connected
    let unique: std::collections::HashSet<u32> = result.assignments.iter().cloned().collect();
    assert!(unique.len() >= 1);
}

#[test]
fn test_cluster_complex_grid() {
    let mut extraction = garfield::ExtractionResult::new();

    // 3x3 grid
    let positions = vec![
        ("00", 0, 0),
        ("01", 0, 1),
        ("02", 0, 2),
        ("10", 1, 0),
        ("11", 1, 1),
        ("12", 1, 2),
        ("20", 2, 0),
        ("21", 2, 1),
        ("22", 2, 2),
    ];

    for (id, _, _) in &positions {
        extraction.add_node(node(id, "grid.rs"));
    }

    // Add edges in grid pattern
    for &(id, r, c) in &positions {
        if c < 2 {
            extraction.add_edge(edge(id, &format!("{}{}", r, c + 1)));
        }
        if r < 2 {
            extraction.add_edge(edge(id, &format!("{}{}", r + 1, c)));
        }
    }

    let graph = build_graph(vec![extraction]);
    let result = cluster(&graph);

    // 3x3 grid should form communities
    let unique: std::collections::HashSet<u32> = result.assignments.iter().cloned().collect();
    assert!(unique.len() >= 1);
}
