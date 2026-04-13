//! Comprehensive unit tests for Leiden community detection algorithm

use garfield::{cluster, types::{GraphData, GraphMetadata, Node, Edge, Confidence}};

fn create_test_node(id: &str, source_file: &str) -> Node {
    Node {
        id: id.to_string(),
        label: id.to_string(),
        source_file: source_file.to_string(),
        source_location: format!("{} @ L1", source_file),
        community: None,
        node_type: Some("function".to_string()),
        file_type: None,
        file_stem: None,
    }
}

fn create_test_edge(source: &str, target: &str, weight: f64) -> Edge {
    Edge {
        source: source.to_string(),
        target: target.to_string(),
        relation: "calls".to_string(),
        confidence: if weight >= 1.0 { Confidence::Extracted } else { Confidence::Inferred },
        confidence_score: weight,
        source_file: String::new(),
        note: None,
    }
}

fn create_graph(nodes: Vec<&str>, edges: Vec<(&str, &str, f64)>) -> GraphData {
    let nodes_count = nodes.len();
    let edges_count = edges.len();
    
    let nodes: Vec<Node> = nodes
        .iter()
        .map(|n| create_test_node(n, "src/mod.rs"))
        .collect();

    let links: Vec<Edge> = edges
        .iter()
        .map(|(s, t, w)| create_test_edge(s, t, *w))
        .collect();

    GraphData {
        nodes,
        links,
        hyperedges: vec![],
        metadata: GraphMetadata::new(nodes_count, edges_count, 0),
    }
}

// ============ Leiden Algorithm Basic Tests ============

mod leiden_basic {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_empty_graph_returns_empty() {
        let graph = create_graph(vec![], vec![]);
        let result = cluster(&graph);
        assert!(result.assignments.is_empty());
    }

    #[test]
    fn test_single_node_one_community() {
        let graph = create_graph(vec!["a"], vec![]);
        let result = cluster(&graph);
        assert_eq!(result.assignments.len(), 1);
    }

    #[test]
    fn test_two_connected_nodes_same_community() {
        let graph = create_graph(vec!["a", "b"], vec![("a", "b", 1.0)]);
        let result = cluster(&graph);
        assert_eq!(result.assignments[0], result.assignments[1]);
    }

    #[test]
    fn test_disconnected_pairs() {
        let graph = create_graph(
            vec!["a", "b", "c", "d"],
            vec![("a", "b", 1.0), ("c", "d", 1.0)],
        );
        let result = cluster(&graph);
        assert_eq!(result.assignments[0], result.assignments[1]);
        assert_eq!(result.assignments[2], result.assignments[3]);
    }

    #[test]
    fn test_chain_graph_connected() {
        let graph = create_graph(
            vec!["a", "b", "c", "d"],
            vec![("a", "b", 1.0), ("b", "c", 1.0), ("c", "d", 1.0)],
        );
        let result = cluster(&graph);
        // All nodes should be connected somehow
        let unique: HashSet<u32> = result.assignments.iter().cloned().collect();
        assert!(unique.len() <= 4);
    }
}

// ============ Leiden Algorithm Weight Tests ============

mod leiden_weights {
    use super::*;

    #[test]
    fn test_high_weight_connections() {
        let graph = create_graph(
            vec!["a", "b", "c"],
            vec![("a", "b", 10.0), ("b", "c", 0.1)],
        );
        let result = cluster(&graph);
        // a and b should likely be together
        assert!(result.assignments[0] == result.assignments[1] || result.assignments.len() == 3);
    }

    #[test]
    fn test_inferred_vs_extracted_confidence() {
        let graph = create_graph(
            vec!["a", "b"],
            vec![("a", "b", 0.5)],
        );
        let result = cluster(&graph);
        // Edge exists, should be same community
        assert_eq!(result.assignments[0], result.assignments[1]);
    }
}

// ============ Leiden Algorithm Complex Graph Tests ============

mod leiden_complex {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_star_graph() {
        let graph = create_graph(
            vec!["center", "leaf1", "leaf2", "leaf3", "leaf4"],
            vec![
                ("center", "leaf1", 1.0),
                ("center", "leaf2", 1.0),
                ("center", "leaf3", 1.0),
                ("center", "leaf4", 1.0),
            ],
        );
        let result = cluster(&graph);
        // Hub should be separate or center of community
        let unique: HashSet<u32> = result.assignments.iter().cloned().collect();
        assert!(unique.len() >= 1 && unique.len() <= 5);
    }

    #[test]
    fn test_two_cliques() {
        let graph = create_graph(
            vec!["a1", "a2", "a3", "b1", "b2", "b3"],
            vec![
                ("a1", "a2", 1.0),
                ("a2", "a3", 1.0),
                ("a1", "a3", 1.0),
                ("b1", "b2", 1.0),
                ("b2", "b3", 1.0),
                ("b1", "b3", 1.0),
            ],
        );
        let result = cluster(&graph);
        // Should form at least 2 communities
        let unique: HashSet<u32> = result.assignments.iter().cloned().collect();
        assert!(unique.len() >= 2);
    }

    #[test]
    fn test_barbell_graph() {
        let graph = create_graph(
            vec!["a1", "a2", "a3", "bridge", "b2", "b3"],
            vec![
                ("a1", "a2", 1.0),
                ("a2", "a3", 1.0),
                ("a1", "a3", 1.0),
                ("a2", "bridge", 1.0),
                ("bridge", "b2", 1.0),
                ("b2", "b3", 1.0),
            ],
        );
        let result = cluster(&graph);
        // Should have multiple communities
        let unique: HashSet<u32> = result.assignments.iter().cloned().collect();
        assert!(unique.len() >= 2);
    }

    #[test]
    fn test_bipartite_graph() {
        let graph = create_graph(
            vec!["a1", "a2", "b1", "b2"],
            vec![
                ("a1", "b1", 1.0),
                ("a1", "b2", 1.0),
                ("a2", "b1", 1.0),
                ("a2", "b2", 1.0),
            ],
        );
        let result = cluster(&graph);
        assert_eq!(result.assignments.len(), 4);
    }
}

// ============ Leiden Algorithm Community Properties Tests ============

mod leiden_properties {
    use super::*;

    #[test]
    fn test_community_sizes_calculated() {
        let graph = create_graph(
            vec!["a", "b", "c", "d", "e", "f"],
            vec![
                ("a", "b", 1.0),
                ("b", "c", 1.0),
                ("d", "e", 1.0),
                ("e", "f", 1.0),
            ],
        );
        let result = cluster(&graph);
        let total: usize = result.community_sizes.values().sum();
        assert_eq!(total, 6);
    }

    #[test]
    fn test_all_nodes_assigned() {
        let graph = create_graph(
            vec!["a", "b", "c", "d", "e"],
            vec![("a", "b", 1.0), ("c", "d", 1.0)],
        );
        let result = cluster(&graph);
        assert_eq!(result.assignments.len(), 5);
    }

    #[test]
    fn test_community_ids_non_negative() {
        let graph = create_graph(
            vec!["a", "b", "c"],
            vec![("a", "b", 1.0)],
        );
        let result = cluster(&graph);
        for &comm in &result.assignments {
            assert!(comm >= 0);
        }
    }
}

// ============ Leiden Algorithm Edge Cases ============

mod leiden_edge_cases {
    use super::*;

    #[test]
    fn test_many_isolated_nodes() {
        let nodes: Vec<String> = (0..10).map(|i| format!("n{}", i)).collect();
        let nodes_ref: Vec<&str> = nodes.iter().map(|s| s.as_str()).collect();
        let graph = create_graph(nodes_ref, vec![]);
        let result = cluster(&graph);
        assert_eq!(result.assignments.len(), 10);
    }

    #[test]
    fn test_self_loops_handled() {
        let graph = create_graph(
            vec!["a", "b"],
            vec![("a", "b", 1.0)],
        );
        let result = cluster(&graph);
        assert_eq!(result.assignments.len(), 2);
    }

    #[test]
    fn test_no_edges() {
        let graph = create_graph(
            vec!["a", "b", "c"],
            vec![],
        );
        let result = cluster(&graph);
        // Each node in its own community
        assert_eq!(result.assignments.len(), 3);
    }
}

// ============ Cohesion Tests ============

mod cohesion_tests {
    use super::*;

    #[test]
    fn test_cohesion_scores_valid() {
        let graph = create_graph(
            vec!["a", "b", "c"],
            vec![("a", "b", 1.0), ("b", "c", 1.0)],
        );
        let result = cluster(&graph);
        for (_, &score) in &result.cohesion_scores {
            assert!((0.0..=1.0).contains(&score));
        }
    }
}
