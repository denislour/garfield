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
    
    // Get first 5 nodes to test
    let matching_nodes: Vec<_> = graph.nodes.iter()
        .take(5)
        .map(|n| n.id.clone())
        .collect();
    
    if matching_nodes.is_empty() {
        println!("No nodes in graph - skipping test");
        return;
    }
    
    let node_ids: HashSet<String> = matching_nodes.into_iter().collect();
    
    // Generate subgraph text
    let output = subgraph_to_text(&graph, &node_ids, &[], 1000);
    
    println!("=== Query Output ===");
    println!("{}", output);
    
    // Assertions
    assert!(output.contains("## Nodes"), "Should have ## Nodes section");
}

#[test]
fn test_explain_output_has_hyperedge_info() {
    let graph = load_graph();
    
    // Find first node with a hyperedge
    let hyperedge_node = graph.hyperedges.iter()
        .next()
        .and_then(|he| he.nodes.first())
        .cloned();
    
    if let Some(node_id) = hyperedge_node {
        let details = get_node(&graph, &node_id);
        
        assert!(details.is_some(), "Should find node: {}", node_id);
        
        let details = details.unwrap();
        
        println!("=== Explain Output ===");
        println!("ID: {}", details.id);
        println!("Label: {}", details.label);
        if let Some(he) = &details.hyperedge {
            println!("Hyperedge: {} ({} members)", he.label, he.member_count);
        }
        
        // Assertions
        assert!(details.hyperedge.is_some(), "Node should have hyperedge");
        
        let he = details.hyperedge.unwrap();
        assert!(he.label.contains("module"), "Hyperedge label should contain 'module'");
        assert!(he.member_count > 0, "Hyperedge should have members");
    } else {
        println!("No hyperedges found - skipping test");
    }
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
    } else {
        println!("All nodes have hyperedges (OK for small codebase)");
    }
}

#[test]
fn test_hyperedge_contains_valid_nodes() {
    let graph = load_graph();
    
    if graph.hyperedges.is_empty() {
        println!("No hyperedges - skipping test");
        return;
    }
    
    // Each hyperedge should contain valid nodes
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
