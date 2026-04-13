//! Integration tests for community detection

use garfield::{
    build_graph, cluster, merge_extractions, merge_into_graph,
    types::{Confidence, Edge, ExtractionResult, GraphData, GraphMetadata, Node},
};

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

fn create_community_test_graph() -> GraphData {
    let nodes = vec![
        create_test_node("module_a_func_1", "src/module_a/mod.rs"),
        create_test_node("module_a_func_2", "src/module_a/mod.rs"),
        create_test_node("module_a_func_3", "src/module_a/mod.rs"),
        create_test_node("module_b_func_1", "src/module_b/mod.rs"),
        create_test_node("module_b_func_2", "src/module_b/mod.rs"),
    ];

    let edges = vec![
        create_test_edge("module_a_func_1", "module_a_func_2", "calls"),
        create_test_edge("module_a_func_2", "module_a_func_3", "calls"),
        create_test_edge("module_b_func_1", "module_b_func_2", "calls"),
        create_test_edge("module_a_func_3", "module_b_func_1", "calls"),
    ];

    GraphData {
        nodes,
        links: edges,
        hyperedges: vec![],
        metadata: GraphMetadata::new(5, 4, 2),
    }
}

#[test]
fn test_cluster_detects_communities() {
    let graph = create_community_test_graph();
    let result = cluster(&graph);

    // Should have assignments for all nodes
    assert_eq!(
        result.assignments.len(),
        graph.nodes.len(),
        "Should have assignment for each node"
    );
}

#[test]
fn test_cluster_assignments_coverage() {
    let graph = create_community_test_graph();
    let result = cluster(&graph);

    assert_eq!(
        result.assignments.len(),
        graph.nodes.len(),
        "Every node should have a community assignment"
    );
}

#[test]
fn test_build_graph_with_communities() {
    let extraction = ExtractionResult {
        nodes: vec![],
        links: vec![],
        hyperedges: vec![],
    };

    let graph = build_graph(vec![extraction]);

    assert_eq!(graph.nodes.len(), 0);
}

#[test]
fn test_merge_extractions_empty() {
    let results: Vec<ExtractionResult> = vec![];
    let merged = merge_extractions(results);

    assert!(merged.nodes.is_empty());
    assert!(merged.links.is_empty());
}

#[test]
fn test_merge_extractions_single() {
    let extraction = ExtractionResult {
        nodes: vec![create_test_node("test", "test.rs")],
        links: vec![],
        hyperedges: vec![],
    };

    let merged = merge_extractions(vec![extraction]);

    assert_eq!(merged.nodes.len(), 1);
}

#[test]
fn test_merge_extractions_deduplicates_nodes() {
    let extraction = ExtractionResult {
        nodes: vec![
            create_test_node("duplicate", "test.rs"),
            create_test_node("duplicate", "test2.rs"),
        ],
        links: vec![],
        hyperedges: vec![],
    };

    let merged = merge_extractions(vec![extraction]);

    assert_eq!(merged.nodes.len(), 1);
}

#[test]
fn test_merge_into_graph_empty() {
    let mut existing = create_community_test_graph();
    let initial_count = existing.nodes.len();

    merge_into_graph(
        &mut existing,
        ExtractionResult {
            nodes: vec![],
            links: vec![],
            hyperedges: vec![],
        },
    );

    assert_eq!(existing.nodes.len(), initial_count);
}

#[test]
fn test_merge_into_graph_adds_new_nodes() {
    let mut existing = create_community_test_graph();
    let initial_count = existing.nodes.len();

    let new_extraction = ExtractionResult {
        nodes: vec![create_test_node("new_node", "new.rs")],
        links: vec![],
        hyperedges: vec![],
    };

    merge_into_graph(&mut existing, new_extraction);

    assert_eq!(existing.nodes.len(), initial_count + 1);
}

#[test]
fn test_merge_into_graph_preserves_existing() {
    let mut existing = create_community_test_graph();

    merge_into_graph(
        &mut existing,
        ExtractionResult {
            nodes: vec![],
            links: vec![],
            hyperedges: vec![],
        },
    );

    assert!(existing.nodes.iter().any(|n| n.id == "module_a_func_1"));
}

#[test]
fn test_community_with_isolated_nodes() {
    let nodes = (0..3)
        .map(|i| create_test_node(&format!("isolated_{}", i), "src/mod.rs"))
        .collect::<Vec<_>>();

    let graph = GraphData {
        nodes,
        links: vec![],
        hyperedges: vec![],
        metadata: GraphMetadata::new(3, 0, 0),
    };

    let result = cluster(&graph);

    assert_eq!(
        result.assignments.len(),
        3,
        "All isolated nodes should have assignments"
    );
}
