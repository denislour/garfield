//! Integration tests for hyperedge detection

use garfield::{
    build_graph, detect_hyperedges,
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

fn create_test_graph(nodes: Vec<Node>, edges: Vec<Edge>) -> GraphData {
    GraphData {
        nodes,
        links: edges,
        hyperedges: vec![],
        metadata: GraphMetadata::new(0, 0, 0),
    }
}

fn make_nodes_file_a() -> Vec<Node> {
    vec![
        create_test_node("a", "src/mod.rs"),
        create_test_node("b", "src/mod.rs"),
    ]
}

fn make_edges_ab() -> Vec<Edge> {
    vec![create_test_edge("a", "b", "calls")]
}

fn make_nodes_multiple_files() -> Vec<Node> {
    vec![
        create_test_node("func_a", "src/mod_a.rs"),
        create_test_node("func_b", "src/mod_b.rs"),
        create_test_node("func_c", "src/mod_c.rs"),
    ]
}

fn make_edges_abc() -> Vec<Edge> {
    vec![
        create_test_edge("func_a", "func_b", "calls"),
        create_test_edge("func_b", "func_c", "calls"),
    ]
}

// ============ Core Hyperedge Tests ============

#[test]
fn test_hyperedge_minimum_nodes() {
    let graph = create_test_graph(make_nodes_file_a(), make_edges_ab());
    let hyperedges = detect_hyperedges(&graph);

    assert!(
        hyperedges.is_empty(),
        "Fewer than 3 nodes in same file should not create hyperedge"
    );
}

#[test]
fn test_hyperedge_call_chain_detection() {
    let mut nodes = make_nodes_multiple_files();
    nodes.push(create_test_node("func_d", "src/mod_d.rs"));

    let mut edges = make_edges_abc();
    edges.push(create_test_edge("func_c", "func_d", "calls"));

    let graph = create_test_graph(nodes, edges);
    let hyperedges = detect_hyperedges(&graph);

    // Should detect call chain: a->b->c->d
    assert!(
        !hyperedges.is_empty(),
        "Call chain should create hyperedge"
    );

    // Should have a call chain hyperedge
    let chain_hyperedges: Vec<_> = hyperedges
        .iter()
        .filter(|h| h.relation.contains("chain") || h.relation.contains("call"))
        .collect();
    assert!(
        !chain_hyperedges.is_empty(),
        "Should detect call chain hyperedge"
    );
}

#[test]
fn test_hyperedge_confidence_scores() {
    let nodes = make_nodes_multiple_files();
    let edges = make_edges_abc();

    let graph = create_test_graph(nodes, edges);
    let hyperedges = detect_hyperedges(&graph);

    for hyperedge in &hyperedges {
        assert!(
            hyperedge.confidence_score >= 0.0 && hyperedge.confidence_score <= 1.0,
            "Confidence score should be between 0 and 1"
        );
    }
}

#[test]
fn test_hyperedge_empty_graph() {
    let graph = create_test_graph(vec![], vec![]);
    let hyperedges = detect_hyperedges(&graph);

    assert!(
        hyperedges.is_empty(),
        "Empty graph should have no hyperedges"
    );
}

#[test]
fn test_hyperedge_single_node() {
    let nodes = vec![create_test_node("only_one", "src/mod.rs")];
    let graph = create_test_graph(nodes, vec![]);
    let hyperedges = detect_hyperedges(&graph);

    let file_he: Vec<_> = hyperedges
        .iter()
        .filter(|h| h.id.starts_with("file_"))
        .collect();

    assert!(
        file_he.is_empty(),
        "Single node should not create file hyperedge"
    );
}

#[test]
fn test_hyperedge_cross_file_relationships() {
    let mut nodes = vec![];
    for i in 0..5 {
        nodes.push(create_test_node(
            &format!("module_a_func_{}", i),
            "src/module_a/mod.rs",
        ));
    }

    let mut edges = vec![];
    for i in 1..5 {
        edges.push(create_test_edge(
            &format!("module_a_func_{}", i - 1),
            &format!("module_a_func_{}", i),
            "calls",
        ));
    }

    let graph = create_test_graph(nodes, edges);
    let hyperedges = detect_hyperedges(&graph);

    assert!(
        !hyperedges.is_empty(),
        "Graph with 5 functions should create hyperedge"
    );
}

#[test]
fn test_build_graph_with_hyperedges() {
    let extraction = ExtractionResult {
        nodes: vec![],
        links: vec![],
        hyperedges: vec![],
    };

    let graph = build_graph(vec![extraction]);

    assert_eq!(graph.nodes.len(), 0);
    assert_eq!(graph.links.len(), 0);
}

#[test]
fn test_hyperedge_id_uniqueness() {
    let nodes = vec![
        create_test_node("func_1", "src/mod.rs"),
        create_test_node("func_2", "src/mod.rs"),
        create_test_node("func_3", "src/mod.rs"),
    ];
    let edges = vec![create_test_edge("func_1", "func_2", "calls")];

    let graph = create_test_graph(nodes, edges);
    let hyperedges = detect_hyperedges(&graph);
    let ids: Vec<_> = hyperedges.iter().map(|h| h.id.clone()).collect();

    let mut unique_ids = ids.clone();
    unique_ids.sort();
    unique_ids.dedup();

    assert_eq!(
        ids.len(),
        unique_ids.len(),
        "Hyperedge IDs should be unique"
    );
}

// ============ Directory-Based Algorithm Tests ============

#[test]
fn test_directory_based_hyperedges() {
    // Create nodes in directories with 6+ nodes each (MIN_NODES * 2 = 6)
    let mut nodes = vec![];

    // auth directory - 6 nodes
    for i in 0..6 {
        nodes.push(create_test_node(
            &format!("auth_func_{}", i),
            &format!("src/auth/user_{}.rs", i),
        ));
    }

    // Add internal edges (5 edges for 6 nodes chain)
    let mut edges = vec![];
    for i in 1..6 {
        edges.push(create_test_edge(
            &format!("auth_func_{}", i - 1),
            &format!("auth_func_{}", i),
            "calls",
        ));
    }

    let graph = create_test_graph(nodes, edges);
    let hyperedges = detect_hyperedges(&graph);

    // Should have hyperedges (file-based or directory-based)
    assert!(
        !hyperedges.is_empty(),
        "Graph with 6+ nodes should create hyperedges"
    );
}

#[test]
fn test_directory_groups_require_minimum_nodes() {
    // Test that directory groups need MIN_NODES * 2 = 6 nodes minimum
    let nodes = vec![
        create_test_node("single_1", "src/partial/func_1.rs"),
        create_test_node("single_2", "src/partial/func_2.rs"),
        create_test_node("single_3", "src/partial/func_3.rs"),
    ];

    let graph = create_test_graph(nodes, vec![]);
    let hyperedges = detect_hyperedges(&graph);

    // Less than 6 nodes in same directory should not create directory group
    let dir_hyperedges: Vec<_> = hyperedges
        .iter()
        .filter(|h| h.id.starts_with("dir_"))
        .collect();

    assert!(
        dir_hyperedges.is_empty(),
        "Less than 6 nodes should not create directory group"
    );
}

#[test]
fn test_directory_groups_skip_root_files() {
    // Files at root level should be skipped
    let nodes = vec![
        create_test_node("root_func_1", "main.rs"),
        create_test_node("root_func_2", "lib.rs"),
        create_test_node("root_func_3", "utils.rs"),
        create_test_node("root_func_4", "mod.rs"),
        create_test_node("root_func_5", "config.rs"),
        create_test_node("root_func_6", "types.rs"),
    ];

    let edges = vec![];
    let graph = create_test_graph(nodes, edges);
    let hyperedges = detect_hyperedges(&graph);

    // Root-level files should not create directory groups
    let dir_hyperedges: Vec<_> = hyperedges
        .iter()
        .filter(|h| h.id.starts_with("dir_"))
        .collect();

    assert!(
        dir_hyperedges.is_empty(),
        "Root files should not create directory groups"
    );
}

#[test]
fn test_directory_groups_have_valid_labels() {
    // Create a directory group and verify label format
    let mut nodes = vec![];
    for i in 0..8 {
        nodes.push(create_test_node(
            &format!("core_func_{}", i),
            &format!("src/core/util_{}.rs", i),
        ));
    }

    // Add internal edges
    let mut edges = vec![];
    for i in 1..8 {
        edges.push(create_test_edge(
            &format!("core_func_{}", i - 1),
            &format!("core_func_{}", i),
            "calls",
        ));
    }

    let graph = create_test_graph(nodes, edges);
    let hyperedges = detect_hyperedges(&graph);

    let dir_hyperedges: Vec<_> = hyperedges
        .iter()
        .filter(|h| h.relation.contains("directory"))
        .collect();

    if !dir_hyperedges.is_empty() {
        for he in &dir_hyperedges {
            // Label should contain "directory"
            assert!(
                he.label.to_lowercase().contains("directory") || he.label.to_lowercase().contains("core"),
                "Directory hyperedge label should mention the directory"
            );
            // Should have at least 6 nodes
            assert!(
                he.nodes.len() >= 6,
                "Directory hyperedge should have at least 6 nodes"
            );
        }
    }
}
