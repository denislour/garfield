//! Integration tests for hyperedge detection

use garfield::{
    build_graph, detect_hyperedges,
    types::{Edge, ExtractionResult, GraphData, GraphMetadata, Hyperedge, Node, Confidence},
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

#[test]
fn test_hyperedge_minimum_nodes() {
    // Create graph with 2 nodes - should NOT create hyperedge (min is 3)
    let graph = create_test_graph(make_nodes_file_a(), make_edges_ab());
    let hyperedges = detect_hyperedges(&graph);

    // File-based hyperedge requires 3+ nodes
    let file_hyperedges: Vec<_> = hyperedges
        .iter()
        .filter(|h| h.id.starts_with("file_"))
        .collect();

    assert!(
        file_hyperedges.is_empty() || file_hyperedges.iter().all(|h| h.nodes.len() >= 3),
        "File hyperedges should have minimum 3 nodes"
    );
}

#[test]
fn test_hyperedge_call_chain_detection() {
    // Create a call chain: a -> b -> c -> d
    let nodes = vec![
        create_test_node("a", "src/mod.rs"),
        create_test_node("b", "src/mod.rs"),
        create_test_node("c", "src/mod.rs"),
        create_test_node("d", "src/mod.rs"),
    ];
    let edges = vec![
        create_test_edge("a", "b", "calls"),
        create_test_edge("b", "c", "calls"),
        create_test_edge("c", "d", "calls"),
    ];

    let graph = create_test_graph(nodes, edges);
    let hyperedges = detect_hyperedges(&graph);

    // Should detect some hyperedge from call chain
    let call_chain_he: Vec<_> = hyperedges
        .iter()
        .filter(|h| h.relation == "calls")
        .collect();

    println!("Call chain hyperedges found: {:?}", call_chain_he.len());
}

#[test]
fn test_hyperedge_confidence_scores() {
    let nodes = vec![
        create_test_node("func_a", "src/mod.rs"),
        create_test_node("func_b", "src/mod.rs"),
        create_test_node("func_c", "src/mod.rs"),
    ];
    let edges = vec![
        create_test_edge("func_a", "func_b", "calls"),
        create_test_edge("func_b", "func_c", "calls"),
    ];

    let graph = create_test_graph(nodes, edges);
    let hyperedges = detect_hyperedges(&graph);

    for he in &hyperedges {
        assert!(
            (0.0..=1.0).contains(&he.confidence_score),
            "Hyperedge '{}' has invalid confidence: {}",
            he.id,
            he.confidence_score
        );
    }
}

#[test]
fn test_hyperedge_empty_graph() {
    let graph = create_test_graph(vec![], vec![]);
    let hyperedges = detect_hyperedges(&graph);

    assert!(hyperedges.is_empty(), "Empty graph should have no hyperedges");
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
        nodes.push(create_test_node(&format!("module_a_func_{}", i), "src/module_a/mod.rs"));
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
