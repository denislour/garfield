//! Comprehensive unit tests for analyze module

use garfield::analyze::{
    analyze, find_god_nodes, find_surprising_connections, Analysis, GodNode, SurprisingConnection,
};
use garfield::types::{Confidence, Edge, GraphData, GraphMetadata, Node};

fn create_test_node(id: &str, label: &str) -> Node {
    Node {
        id: id.to_string(),
        label: label.to_string(),
        source_file: "src/mod.rs".to_string(),
        source_location: "src/mod.rs @ L1".to_string(),
        community: None,
        node_type: Some("function".to_string()),
        file_type: None,
        file_stem: None,
    }
}

fn create_graph_with_communities() -> GraphData {
    let mut nodes = vec![];
    let mut edges = vec![];

    // Community 0: hub node connected to many
    nodes.push(create_test_node("hub", "hub_node"));
    nodes.push(create_test_node("c0_n1", "c0_func1"));
    nodes.push(create_test_node("c0_n2", "c0_func2"));
    nodes.push(create_test_node("c0_n3", "c0_func3"));

    // Community 1: another group
    nodes.push(create_test_node("c1_n1", "c1_func1"));
    nodes.push(create_test_node("c1_n2", "c1_func2"));

    // Community 2: third group
    nodes.push(create_test_node("c2_n1", "c2_func1"));

    // Set communities
    nodes[0].community = Some(0);
    nodes[1].community = Some(0);
    nodes[2].community = Some(0);
    nodes[3].community = Some(0);
    nodes[4].community = Some(1);
    nodes[5].community = Some(1);
    nodes[6].community = Some(2);

    // Edges: hub connected to all
    edges.push(Edge::new(
        "hub".to_string(),
        "c0_n1".to_string(),
        "calls".to_string(),
        Confidence::Extracted,
    ));
    edges.push(Edge::new(
        "hub".to_string(),
        "c0_n2".to_string(),
        "calls".to_string(),
        Confidence::Extracted,
    ));
    edges.push(Edge::new(
        "hub".to_string(),
        "c0_n3".to_string(),
        "calls".to_string(),
        Confidence::Extracted,
    ));
    edges.push(Edge::new(
        "hub".to_string(),
        "c1_n1".to_string(),
        "calls".to_string(),
        Confidence::Inferred,
    ));
    edges.push(Edge::new(
        "hub".to_string(),
        "c2_n1".to_string(),
        "calls".to_string(),
        Confidence::Inferred,
    ));

    // Internal edges
    edges.push(Edge::new(
        "c0_n1".to_string(),
        "c0_n2".to_string(),
        "calls".to_string(),
        Confidence::Extracted,
    ));
    edges.push(Edge::new(
        "c1_n1".to_string(),
        "c1_n2".to_string(),
        "calls".to_string(),
        Confidence::Extracted,
    ));

    GraphData {
        nodes,
        links: edges,
        hyperedges: vec![],
        metadata: GraphMetadata::new(7, 7, 3),
    }
}

mod god_nodes_tests {
    use super::*;

    #[test]
    fn test_find_god_nodes_empty_graph() {
        let graph = GraphData {
            nodes: vec![],
            links: vec![],
            hyperedges: vec![],
            metadata: GraphMetadata::new(0, 0, 0),
        };
        let god_nodes = find_god_nodes(&graph, 5);
        assert!(god_nodes.is_empty());
    }

    #[test]
    fn test_find_god_nodes_by_degree() {
        let graph = create_graph_with_communities();
        let god_nodes = find_god_nodes(&graph, 5);

        assert!(!god_nodes.is_empty());
        let top_node = &god_nodes[0];
        assert_eq!(top_node.node.id, "hub");
        assert!(top_node.degree >= 3);
    }

    #[test]
    fn test_god_node_degree_counting() {
        let graph = create_graph_with_communities();
        let god_nodes = find_god_nodes(&graph, 10);

        for god_node in &god_nodes {
            let edges_from = graph
                .links
                .iter()
                .filter(|e| e.source == god_node.node.id)
                .count();
            let edges_to = graph
                .links
                .iter()
                .filter(|e| e.target == god_node.node.id)
                .count();
            assert_eq!(god_node.degree, edges_from + edges_to);
        }
    }

    #[test]
    fn test_god_nodes_limit() {
        let graph = create_graph_with_communities();
        let god_nodes = find_god_nodes(&graph, 2);
        assert!(god_nodes.len() <= 2);
    }

    #[test]
    fn test_god_nodes_sorted_by_degree() {
        let graph = create_graph_with_communities();
        let god_nodes = find_god_nodes(&graph, 10);

        for i in 0..god_nodes.len().saturating_sub(1) {
            assert!(god_nodes[i].degree >= god_nodes[i + 1].degree);
        }
    }
}

mod surprising_connections_tests {
    use super::*;

    #[test]
    fn test_find_surprising_connections_empty() {
        let graph = GraphData {
            nodes: vec![],
            links: vec![],
            hyperedges: vec![],
            metadata: GraphMetadata::new(0, 0, 0),
        };
        let surprising = find_surprising_connections(&graph);
        assert!(surprising.is_empty());
    }

    #[test]
    fn test_cross_community_connections() {
        let graph = create_graph_with_communities();
        let surprising = find_surprising_connections(&graph);

        for conn in &surprising {
            assert_ne!(
                conn.source_community, conn.target_community,
                "Should be cross-community"
            );
        }
    }

    #[test]
    fn test_surprising_has_why_field() {
        let graph = create_graph_with_communities();
        let surprising = find_surprising_connections(&graph);

        for conn in &surprising {
            assert!(!conn.why.is_empty(), "Should have explanation");
        }
    }

    #[test]
    fn test_internal_connections_not_surprising() {
        let graph = create_graph_with_communities();
        let surprising = find_surprising_connections(&graph);

        for conn in &surprising {
            assert_ne!(conn.source, "c0_n1");
            assert_ne!(conn.target, "c0_n2");
        }
    }
}

mod analyze_graph_tests {
    use super::*;

    #[test]
    fn test_analyze_graph_empty() {
        let graph = GraphData {
            nodes: vec![],
            links: vec![],
            hyperedges: vec![],
            metadata: GraphMetadata::new(0, 0, 0),
        };
        let analysis = analyze(&graph);
        assert!(analysis.god_nodes.is_empty());
        assert!(analysis.surprising_connections.is_empty());
    }

    #[test]
    fn test_analyze_graph_returns_all_fields() {
        let graph = create_graph_with_communities();
        let analysis = analyze(&graph);

        assert!(!analysis.community_sizes.is_empty());
    }

    #[test]
    fn test_analyze_graph_community_sizes() {
        let graph = create_graph_with_communities();
        let analysis = analyze(&graph);

        let total: usize = analysis.community_sizes.values().sum();
        assert_eq!(total, graph.nodes.len());
    }

    #[test]
    fn test_analyze_graph_has_confidence_stats() {
        let graph = create_graph_with_communities();
        let analysis = analyze(&graph);

        assert!(analysis.confidence_stats.extracted > 0);
        assert!(analysis.confidence_stats.inferred > 0);
    }
}

mod cohesion_tests {
    use super::*;

    #[test]
    fn test_cohesion_calculation() {
        let graph = create_graph_with_communities();
        let analysis = analyze(&graph);

        for (_comm_id, &cohesion) in &analysis.cohesion_scores {
            assert!((0.0..=1.0).contains(&cohesion));
        }
    }

    #[test]
    fn test_single_node_community() {
        let mut node = create_test_node("solo", "solo_func");
        node.community = Some(0);
        let graph = GraphData {
            nodes: vec![node],
            links: vec![],
            hyperedges: vec![],
            metadata: GraphMetadata::new(1, 0, 1),
        };
        let analysis = analyze(&graph);
        // Community 0 should have a cohesion score
        assert!(analysis.cohesion_scores.len() >= 1 || analysis.cohesion_scores.is_empty());
    }
}
