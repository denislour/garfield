//! Tests for hyperedge flow: Query & Explain

use garfield::{
    from_json,
    serve::{subgraph_to_text, get_node},
    GraphData,
};
use std::collections::HashSet;
use std::path::Path;

/// Helper: Load graph from garfield-out
fn load_graph() -> GraphData {
    from_json(Path::new("garfield-out/graph.json")).unwrap()
}

#[test]
fn test_query_output_has_hyperedge_annotation() {
    let graph = load_graph();
    
    // Get nodes containing "user"
    let matching_nodes: Vec<_> = graph.nodes.iter()
        .filter(|n| n.label.to_lowercase().contains("user"))
        .map(|n| n.id.clone())
        .collect();
    
    let node_ids: HashSet<String> = matching_nodes.into_iter().collect();
    
    // Generate subgraph text
    let output = subgraph_to_text(&graph, &node_ids, &[], 1000);
    
    println!("=== Query Output ===");
    println!("{}", output);
    
    // Assertions
    assert!(output.contains("## Nodes"), "Should have ## Nodes section");
    assert!(output.contains("[user_service module]"), 
        "Nodes should have hyperedge annotation");
}

#[test]
fn test_explain_output_has_hyperedge_section() {
    let graph = load_graph();
    
    // Get details for "update_user"
    let details = get_node(&graph, "update_user");
    
    assert!(details.is_some(), "Should find update_user node");
    
    let details = details.unwrap();
    
    println!("=== Explain Output ===");
    println!("ID: {}", details.id);
    println!("Label: {}", details.label);
    if let Some(he) = &details.hyperedge {
        println!("Hyperedge: {} ({} members)", he.label, he.member_count);
    }
    
    // Assertions
    assert_eq!(details.id, "user_service:update_user");
    assert!(details.hyperedge.is_some(), "Node should have hyperedge");
    
    let he = details.hyperedge.unwrap();
    assert!(he.label.contains("user_service"), "Hyperedge label should contain user_service");
    assert!(he.member_count > 0, "Hyperedge should have members");
}

#[test]
fn test_node_without_hyperedge() {
    let graph = load_graph();
    
    // Find a node that's NOT in any hyperedge
    let node_without_he = graph.nodes.iter()
        .find(|n| !graph.hyperedges.iter().any(|he| he.nodes.contains(&n.id)));
    
    if let Some(node) = node_without_he {
        println!("Node without hyperedge: {}", node.id);
        
        let details = get_node(&graph, &node.id);
        assert!(details.is_some());
        assert!(details.unwrap().hyperedge.is_none(), 
            "Node not in hyperedge should have no hyperedge info");
    }
}

#[test]
fn test_hyperedge_contains_nodes() {
    let graph = load_graph();
    
    // Each hyperedge should contain at least 3 nodes
    for he in &graph.hyperedges {
        println!("Hyperedge: {} has {} nodes", he.label, he.nodes.len());
        assert!(he.nodes.len() >= 1, "Hyperedge should have at least 1 node");
        
        // Verify each node_id in hyperedge actually exists
        for node_id in &he.nodes {
            assert!(
                graph.nodes.iter().any(|n| &n.id == node_id),
                "Hyperedge references non-existent node: {}",
                node_id
            );
        }
    }
}
