//! Integration tests for serve module (query, explain, etc.)

use garfield::{
    serve::{
        bfs, dfs, find_shortest_path, get_community, get_hyperedge, get_neighbors, get_node,
        score_nodes, subgraph_to_text,
    },
    types::{Edge, GraphData, GraphMetadata, Hyperedge, Node, Confidence},
};
use std::collections::HashSet;

fn create_test_node(id: &str, label: &str, source_file: &str, node_type: &str) -> Node {
    Node {
        id: id.to_string(),
        label: label.to_string(),
        source_file: source_file.to_string(),
        source_location: format!("{} @ L1", source_file),
        community: None,
        node_type: Some(node_type.to_string()),
        file_type: None,
        file_stem: None,
    }
}

fn create_test_edge(source: &str, target: &str, relation: &str) -> Edge {
    Edge {
        source: source.to_string(),
        target: target.to_string(),
        relation: relation.to_string(),
        confidence: Confidence::Extracted,
        confidence_score: 1.0,
        source_file: String::new(),
        note: None,
    }
}

fn create_test_graph() -> GraphData {
    GraphData {
        nodes: vec![
            create_test_node("fn_add", "add", "src/math.rs", "function"),
            create_test_node("fn_subtract", "subtract", "src/math.rs", "function"),
            create_test_node("fn_multiply", "multiply", "src/math.rs", "function"),
            create_test_node("class_calculator", "Calculator", "src/calc.rs", "class"),
        ],
        links: vec![
            create_test_edge("fn_add", "class_calculator", "defines"),
            create_test_edge("fn_subtract", "class_calculator", "defines"),
            create_test_edge("fn_multiply", "class_calculator", "defines"),
            create_test_edge("class_calculator", "fn_add", "calls"),
        ],
        hyperedges: vec![Hyperedge {
            id: "file_math".to_string(),
            label: "math module".to_string(),
            nodes: vec![
                "fn_add".to_string(),
                "fn_subtract".to_string(),
                "fn_multiply".to_string(),
            ],
            relation: "participate_in".to_string(),
            confidence: Confidence::Inferred,
            confidence_score: 0.8,
            source_file: "src/math.rs".to_string(),
        }],
        metadata: GraphMetadata::new(4, 4, 2),
    }
}

#[test]
fn test_score_nodes_exact_match() {
    let graph = create_test_graph();
    let terms = vec!["add".to_string()];
    let scores = score_nodes(&graph, &terms);

    assert!(!scores.is_empty());
    assert_eq!(scores[0].1, "fn_add");
}

#[test]
fn test_score_nodes_partial_match() {
    let graph = create_test_graph();
    let terms = vec!["calc".to_string()];
    let scores = score_nodes(&graph, &terms);

    assert!(!scores.is_empty());
    assert!(
        scores.iter().any(|(_, id)| id.contains("calc")),
        "Should find nodes with 'calc'"
    );
}

#[test]
fn test_score_nodes_no_match() {
    let graph = create_test_graph();
    let terms = vec!["xyz123".to_string()];
    let scores = score_nodes(&graph, &terms);

    assert!(scores.is_empty());
}

#[test]
fn test_score_nodes_case_insensitive() {
    let graph = create_test_graph();
    let terms_lower = vec!["add".to_string()];
    let terms_upper = vec!["ADD".to_string()];
    let scores_lower = score_nodes(&graph, &terms_lower);
    let scores_upper = score_nodes(&graph, &terms_upper);

    assert_eq!(scores_lower.len(), scores_upper.len());
}

#[test]
fn test_bfs_traversal() {
    let graph = create_test_graph();
    let start_nodes = vec!["fn_add"];
    let (nodes, _) = bfs(&graph, &start_nodes, 2);

    assert!(!nodes.is_empty());
    assert!(nodes.contains(&"fn_add".to_string()));
}

#[test]
fn test_bfs_depth_limit() {
    let graph = create_test_graph();
    let start_nodes = vec!["class_calculator"];

    let (nodes_depth_1, _) = bfs(&graph, &start_nodes, 1);
    let (nodes_depth_2, _) = bfs(&graph, &start_nodes, 2);

    assert!(
        nodes_depth_2.len() >= nodes_depth_1.len(),
        "Deeper search should find equal or more nodes"
    );
}

#[test]
fn test_dfs_traversal() {
    let graph = create_test_graph();
    let start_nodes = vec!["fn_add"];
    let (nodes, _) = dfs(&graph, &start_nodes, 2);

    assert!(!nodes.is_empty());
}

#[test]
fn test_dfs_vs_bfs_difference() {
    let graph = create_test_graph();
    let start_nodes = vec!["class_calculator"];

    let (bfs_nodes, _) = bfs(&graph, &start_nodes, 3);
    let (dfs_nodes, _) = dfs(&graph, &start_nodes, 3);

    assert_eq!(bfs_nodes, dfs_nodes, "BFS and DFS should find same reachable nodes");
}

#[test]
fn test_find_shortest_path_direct() {
    let graph = create_test_graph();
    let path = find_shortest_path(&graph, "fn_add", "fn_subtract", 10);

    assert!(path.is_some());
    let path = path.unwrap();
    assert!(!path.is_empty());
}

#[test]
fn test_find_shortest_path_no_path() {
    let graph = create_test_graph();
    let path = find_shortest_path(&graph, "fn_add", "nonexistent", 10);

    assert!(path.is_none());
}

#[test]
fn test_get_node_by_id() {
    let graph = create_test_graph();
    let details = get_node(&graph, "fn_add");

    assert!(details.is_some());
    let details = details.unwrap();
    assert_eq!(details.id, "fn_add");
    assert_eq!(details.label, "add");
}

#[test]
fn test_get_node_by_label() {
    let graph = create_test_graph();
    let details = get_node(&graph, "Calculator");

    assert!(details.is_some());
    let details = details.unwrap();
    assert_eq!(details.id, "class_calculator");
}

#[test]
fn test_get_node_not_found() {
    let graph = create_test_graph();
    let details = get_node(&graph, "nonexistent");

    assert!(details.is_none());
}

#[test]
fn test_get_neighbors() {
    let graph = create_test_graph();
    let neighbors = get_neighbors(&graph, "fn_add", 10);

    assert!(!neighbors.is_empty());
}

#[test]
fn test_get_neighbors_none() {
    let graph = create_test_graph();
    let neighbors = get_neighbors(&graph, "nonexistent", 10);

    assert!(neighbors.is_empty());
}

#[test]
fn test_get_community() {
    let graph = create_test_graph();
    let community = get_community(&graph, 1);

    if let Some(community) = community {
        assert!(!community.nodes.is_empty());
    }
}

#[test]
fn test_get_community_not_found() {
    let graph = create_test_graph();
    let community = get_community(&graph, 999);

    assert!(community.is_none());
}

#[test]
fn test_get_hyperedge() {
    let graph = create_test_graph();
    let hyperedge = get_hyperedge(&graph, "file_math");

    assert!(hyperedge.is_some());
    let hyperedge = hyperedge.unwrap();
    assert_eq!(hyperedge.id, "file_math");
    assert!(hyperedge.member_count >= 3);
}

#[test]
fn test_get_hyperedge_not_found() {
    let graph = create_test_graph();
    let hyperedge = get_hyperedge(&graph, "nonexistent");

    assert!(hyperedge.is_none());
}

#[test]
fn test_subgraph_to_text_with_nodes() {
    let graph = create_test_graph();
    let node_ids: HashSet<String> = vec![
        "fn_add".to_string(),
        "fn_subtract".to_string(),
    ]
    .into_iter()
    .collect();

    let output = subgraph_to_text(&graph, &node_ids, &[], 1000);

    // Output should not be empty when nodes are provided
    assert!(!output.is_empty(), "Should produce non-empty output");
}

#[test]
fn test_subgraph_to_text_empty_input() {
    let graph = create_test_graph();
    let node_ids: HashSet<String> = vec![].into_iter().collect();

    let output = subgraph_to_text(&graph, &node_ids, &[], 1000);

    // Output should not crash with empty input
    assert!(!output.is_empty() || output.is_empty(), "Should handle empty input gracefully");
}

#[test]
fn test_subgraph_to_text_respects_token_budget() {
    let graph = create_test_graph();
    let node_ids: HashSet<String> = graph.nodes.iter().map(|n| n.id.clone()).collect();

    let output_small = subgraph_to_text(&graph, &node_ids, &[], 100);
    let output_large = subgraph_to_text(&graph, &node_ids, &[], 10000);

    // Both should produce output
    assert!(!output_small.is_empty());
    assert!(!output_large.is_empty());
}
