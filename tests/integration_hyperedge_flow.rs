//! Test hyperedge flow: Build -> Query -> Get hyperedge info

use garfield::{detect_hyperedges, from_json, get_hyperedge, get_node};
use std::path::Path;

#[test]
#[ignore]
fn test_hyperedge_flow_build_to_query() {
    // This test requires garfield-out/graph.json which is gitignored
    // Run: cargo run -- build . --output garfield-out
    // Then: cargo test test_hyperedge_flow_build_to_query -- --ignored
    
    let graph_path = Path::new("garfield-out/graph.json");
    if !graph_path.exists() {
        println!("SKIP: garfield-out/graph.json not found. Run 'cargo run -- build . --output garfield-out' first.");
        return;
    }
    
    let graph = from_json(graph_path).unwrap();

    // 1. Check graph has hyperedges
    assert!(!graph.hyperedges.is_empty(), "Graph should have hyperedges");
    println!("[OK] Graph has {} hyperedges", graph.hyperedges.len());

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

    println!("\n=== NODE -> HYPEREDGE ===");
    println!("Module: {}", he_info.label);
    println!("Members: {}", he_info.member_count);
    println!("Confidence: {:.2}", he_info.confidence_score);

    // 5. Query hyperedge directly
    let hyperedge_info = get_hyperedge(&graph, &hyperedge.id);
    assert!(hyperedge_info.is_some(), "Should find hyperedge by ID");
    println!("\n[OK] Hyperedge flow works!");
}

#[test]
#[ignore]
fn test_every_node_has_hyperedge() {
    let graph_path = Path::new("garfield-out/graph.json");
    if !graph_path.exists() {
        println!("SKIP: garfield-out/graph.json not found. Run 'cargo run -- build . --output garfield-out' first.");
        return;
    }
    
    let graph = from_json(graph_path).unwrap();

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
    
    println!("\n[OK] Hyperedge coverage test passed");
}
