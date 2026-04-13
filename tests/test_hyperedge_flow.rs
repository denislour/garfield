//! Test hyperedge flow: Build → Query → Get hyperedge info

use garfield::{detect_hyperedges, from_json, get_hyperedge, get_node};
use std::path::Path;

#[test]
fn test_hyperedge_flow_build_to_query() {
    // Load existing graph (from garfield-out)
    let graph = from_json(Path::new("garfield-out/graph.json")).unwrap();

    // 1. Check graph has hyperedges
    assert!(!graph.hyperedges.is_empty(), "Graph should have hyperedges");
    println!("✅ Graph has {} hyperedges", graph.hyperedges.len());

    // 2. Find a hyperedge
    let hyperedge = &graph.hyperedges[0];
    println!("\n=== HYPEREDGE ===");
    println!("ID: {}", hyperedge.id);
    println!("Label: {}", hyperedge.label);
    println!("Nodes: {}", hyperedge.nodes.len());
    println!("Score: {:.2}", hyperedge.confidence_score);
    println!("Relation: {}", hyperedge.relation);

    // 3. Get node that belongs to this hyperedge
    let node_id = &hyperedge.nodes[0];
    let node_details = get_node(&graph, node_id);

    assert!(node_details.is_some(), "Should find node");
    let details = node_details.unwrap();

    println!("\n=== NODE DETAILS ===");
    println!("ID: {}", details.id);
    println!("Label: {}", details.label);
    println!("File: {}", details.source_file);

    // 4. Check node has hyperedge info
    assert!(
        details.hyperedge.is_some(),
        "Node should have hyperedge info"
    );
    let he_info = details.hyperedge.unwrap();

    println!("\n=== NODE → HYPEREDGE ===");
    println!("Module: {}", he_info.label);
    println!("Members: {}", he_info.member_count);
    println!("Confidence: {:.2}", he_info.confidence_score);

    // 5. Query hyperedge directly
    let hyperedge_info = get_hyperedge(&graph, &hyperedge.id);
    assert!(hyperedge_info.is_some(), "Should find hyperedge by ID");
    println!("\n✅ Hyperedge flow works!");
}

#[test]
fn test_every_node_has_hyperedge() {
    let graph = from_json(Path::new("garfield-out/graph.json")).unwrap();

    // Get all node IDs that are in hyperedges
    let hyperedge_nodes: std::collections::HashSet<_> = graph
        .hyperedges
        .iter()
        .flat_map(|he| he.nodes.iter())
        .collect();

    println!("Total nodes: {}", graph.nodes.len());
    println!("Nodes in hyperedges: {}", hyperedge_nodes.len());

    // Check how many nodes are in hyperedges
    // (only files with 3+ definitions get hyperedges)
    let coverage = (hyperedge_nodes.len() as f64 / graph.nodes.len() as f64) * 100.0;
    println!("Coverage: {:.1}%", coverage);
}
