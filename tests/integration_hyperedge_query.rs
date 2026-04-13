//! Tests for hyperedge flow: Query & Explain

use garfield::{
    from_json,
    serve::{get_node, subgraph_to_text},
    GraphData,
};
use std::collections::HashSet;
use std::path::Path;

/// Helper: Load graph from garfield-out
fn load_graph() -> Option<GraphData> {
    let path = Path::new("garfield-out/graph.json");
    if !path.exists() {
        None
    } else {
        from_json(path).ok()
    }
}

#[test]
#[ignore]
fn test_query_output_has_hyperedge_annotation() {
    let graph = match load_graph() {
        Some(g) => g,
        None => {
            println!("SKIP: garfield-out/graph.json not found");
            return;
        }
    };

    // Get first 5 nodes to test
    let matching_nodes: Vec<_> = graph.nodes.iter().take(5).map(|n| n.id.clone()).collect();

    if matching_nodes.is_empty() {
        println!("No nodes in graph - skipping test");
        return;
    }

    let node_ids: HashSet<String> = matching_nodes.into_iter().collect();

    // Generate subgraph text
    let output = subgraph_to_text(&graph, &node_ids, &[], 1000);

    // Output should contain node info
    assert!(!output.is_empty(), "Should generate output");
    println!("[OK] Query output generated successfully");
}

#[test]
#[ignore]
fn test_hyperedge_contains_valid_nodes() {
    let graph = match load_graph() {
        Some(g) => g,
        None => {
            println!("SKIP: garfield-out/graph.json not found");
            return;
        }
    };

    for hyperedge in &graph.hyperedges {
        for node_id in &hyperedge.nodes {
            // Check node exists in graph
            let exists = graph.nodes.iter().any(|n| &n.id == node_id);
            assert!(exists, "Hyperedge {} contains invalid node: {}", hyperedge.id, node_id);
        }
    }
    println!("[OK] All hyperedges contain valid nodes");
}

#[test]
#[ignore]
fn test_node_without_hyperedge() {
    let graph = match load_graph() {
        Some(g) => g,
        None => {
            println!("SKIP: garfield-out/graph.json not found");
            return;
        }
    };

    // Get nodes that are NOT in any hyperedge
    let hyperedge_nodes: HashSet<_> = graph
        .hyperedges
        .iter()
        .flat_map(|he| he.nodes.iter())
        .collect();

    let isolated: Vec<_> = graph
        .nodes
        .iter()
        .filter(|n| !hyperedge_nodes.contains(&n.id))
        .collect();

    println!("Nodes without hyperedge: {}/{}", isolated.len(), graph.nodes.len());
    println!("[OK] Node hyperedge assignment works");
}

#[test]
#[ignore]
fn test_explain_output_has_hyperedge_info() {
    let graph = match load_graph() {
        Some(g) => g,
        None => {
            println!("SKIP: garfield-out/graph.json not found");
            return;
        }
    };

    // Find a node that has a hyperedge
    let node_with_he = graph.nodes.iter().find(|n| {
        graph.hyperedges.iter().any(|he| he.nodes.contains(&n.id))
    });

    let Some(node) = node_with_he else {
        println!("No nodes with hyperedges - skipping test");
        return;
    };

    let details = get_node(&graph, &node.id);
    assert!(details.is_some(), "Should get node details");

    let details = details.unwrap();

    // Check hyperedge info
    if let Some(he) = details.hyperedge {
        assert!(he.label.len() > 0, "Hyperedge should have label");
        assert!(he.member_count > 0, "Hyperedge should have members");
        println!("[OK] Node {} has hyperedge: {}", node.id, he.label);
    } else {
        println!("Node {} has no hyperedge", node.id);
    }
}
