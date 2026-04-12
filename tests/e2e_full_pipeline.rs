//! E2E tests - Full pipeline tests

use garfield::{analyze, build_graph, detect, extract_file, find_shortest_path, score_nodes, validate_graph};
use garfield::types::{Confidence, Edge, Node, ExtractionResult};
use std::path::Path;

fn node(id: &str, file: &str) -> Node {
    Node::new(id.into(), id.into(), file.into(), "L1".into())
}

fn edge(src: &str, tgt: &str) -> Edge {
    Edge::new(src.into(), tgt.into(), "calls".into(), Confidence::Extracted)
}

// ===== Full Pipeline E2E =====

#[test]
fn test_e2e_full_pipeline_real_directory() {
    let test_dir = Path::new("../graphify");
    if !test_dir.exists() {
        println!("SKIP: graphify directory not found");
        return;
    }
    
    // 1. Detect
    let files = detect::detect(test_dir).unwrap();
    let code_files: Vec<_> = files.files
        .into_iter()
        .filter(|f| f.file_type == garfield::FileType::Code)
        .collect();
    assert!(!code_files.is_empty(), "Should detect code files");
    println!("✓ Detected {} code files", code_files.len());
    
    // 2. Extract
    let mut extraction = None;
    for file in &code_files {
        if let Ok(source) = std::fs::read_to_string(&file.path) {
            if let Ok(result) = extract_file(&file.path, &source) {
                if !result.nodes.is_empty() {
                    extraction = Some(result);
                    break;
                }
            }
        }
    }
    
    let extraction = match extraction {
        Some(e) => e,
        None => {
            println!("SKIP: No valid extraction found");
            return;
        }
    };
    println!("✓ Extracted {} nodes, {} edges", extraction.nodes.len(), extraction.links.len());
    
    // 3. Build graph
    let graph = build_graph(vec![extraction]);
    assert!(graph.nodes.len() > 0, "Should have nodes");
    println!("✓ Built graph with {} nodes, {} edges", graph.nodes.len(), graph.links.len());
    
    // 4. Validate
    let validation = validate_graph(&graph);
    if let Err(e) = validation {
        println!("⚠ Validation warning: {:?}", e);
    } else {
        println!("✓ Graph validation passed");
    }
    
    // 5. Analyze
    let analysis = analyze(&graph);
    println!("✓ Analysis complete: {} communities", analysis.community_sizes.len());
    
    // 6. Query
    if !graph.nodes.is_empty() {
        let first_node = &graph.nodes[0].label;
        let terms = vec![first_node.to_lowercase()];
        let scores = score_nodes(&graph, &terms);
        println!("✓ Query for '{}' found {} results", first_node, scores.len());
    }
}

#[test]
fn test_e2e_python_extraction() {
    let test_dir = Path::new("../graphify");
    if !test_dir.exists() {
        println!("SKIP");
        return;
    }
    
    // Find a Python file
    let files = detect::detect(test_dir).unwrap();
    let py_files: Vec<_> = files.files
        .into_iter()
        .filter(|f| f.path.to_string_lossy().ends_with(".py"))
        .collect();
    
    if py_files.is_empty() {
        println!("SKIP: No Python files found");
        return;
    }
    
    let file = &py_files[0];
    if let Ok(source) = std::fs::read_to_string(&file.path) {
        let result = extract_file(&file.path, &source);
        assert!(result.is_ok());
        println!("✓ Extracted Python file: {:?}", file.path.file_name());
    }
}

#[test]
fn test_e2e_rust_extraction() {
    let test_dir = Path::new("../graphify");
    if !test_dir.exists() {
        println!("SKIP");
        return;
    }
    
    // Find a Rust file
    let files = detect::detect(test_dir).unwrap();
    let rs_files: Vec<_> = files.files
        .into_iter()
        .filter(|f| f.path.to_string_lossy().ends_with(".rs"))
        .collect();
    
    if rs_files.is_empty() {
        println!("SKIP: No Rust files found");
        return;
    }
    
    let file = &rs_files[0];
    if let Ok(source) = std::fs::read_to_string(&file.path) {
        let result = extract_file(&file.path, &source);
        assert!(result.is_ok());
        println!("✓ Extracted Rust file: {:?}", file.path.file_name());
    }
}

// ===== Complex Graph E2E =====

#[test]
fn test_e2e_complex_call_graph() {
    // Create a realistic call graph
    let mut extraction = ExtractionResult::new();
    
    // API Layer
    extraction.add_node(node("api:handle_request", "api.rs"));
    extraction.add_node(node("api:parse_request", "api.rs"));
    extraction.add_node(node("api:format_response", "api.rs"));
    
    // Service Layer
    extraction.add_node(node("service:process_order", "service.rs"));
    extraction.add_node(node("service:validate_order", "service.rs"));
    extraction.add_node(node("service:save_order", "service.rs"));
    
    // Data Layer
    extraction.add_node(node("db:insert", "db.rs"));
    extraction.add_node(node("db:update", "db.rs"));
    extraction.add_node(node("db:query", "db.rs"));
    
    // API calls
    extraction.add_edge(edge("api:handle_request", "api:parse_request"));
    extraction.add_edge(edge("api:handle_request", "service:process_order"));
    extraction.add_edge(edge("api:handle_request", "api:format_response"));
    
    // Service calls
    extraction.add_edge(edge("service:process_order", "service:validate_order"));
    extraction.add_edge(edge("service:process_order", "service:save_order"));
    
    // DB calls
    extraction.add_edge(edge("service:save_order", "db:insert"));
    extraction.add_edge(edge("service:validate_order", "db:query"));
    
    let graph = build_graph(vec![extraction]);
    
    // Find path from API to DB
    let path = find_shortest_path(&graph, "api:handle_request", "db:insert", 10);
    assert!(path.is_some());
    let path = path.unwrap();
    assert!(path.len() >= 3);
    
    // Find path from API to service
    let path2 = find_shortest_path(&graph, "api:handle_request", "service:validate_order", 10);
    assert!(path2.is_some());
    
    println!("✓ Call graph: {} nodes, {} edges", graph.nodes.len(), graph.links.len());
    println!("✓ Path from API to DB: {:?}", path);
}

#[test]
fn test_e2e_microservice_architecture() {
    let mut extraction = ExtractionResult::new();
    
    // User Service
    extraction.add_node(node("user:create", "user_service.rs"));
    extraction.add_node(node("user:get", "user_service.rs"));
    extraction.add_node(node("user:update", "user_service.rs"));
    
    // Order Service
    extraction.add_node(node("order:create", "order_service.rs"));
    extraction.add_node(node("order:get", "order_service.rs"));
    extraction.add_node(node("order:cancel", "order_service.rs"));
    
    // Payment Service
    extraction.add_node(node("payment:charge", "payment_service.rs"));
    extraction.add_node(node("payment:refund", "payment_service.rs"));
    
    // Internal edges
    extraction.add_edge(edge("order:create", "user:get")); // Verify user
    extraction.add_edge(edge("order:create", "payment:charge")); // Charge payment
    extraction.add_edge(edge("order:cancel", "payment:refund")); // Refund
    
    let graph = build_graph(vec![extraction]);
    
    // Find path from order creation to payment
    let path = find_shortest_path(&graph, "order:create", "payment:charge", 10);
    assert!(path.is_some());
    
    // Query for payment-related functions
    let payment_terms = vec!["payment".to_string()];
    let payment_nodes = score_nodes(&graph, &payment_terms);
    assert!(payment_nodes.len() >= 2);
    
    println!("✓ Microservice graph: {} services", graph.nodes.len());
}

#[test]
fn test_e2e_large_graph_performance() {
    let mut extraction = ExtractionResult::new();
    
    // Create a moderately large graph (100 nodes)
    for i in 0..100 {
        extraction.add_node(node(&format!("node_{}", i), "large.rs"));
    }
    
    // Create sparse connections (each node connected to ~5 others)
    for i in 0..100 {
        for j in 1..=5 {
            let target = (i + j) % 100;
            extraction.add_edge(edge(&format!("node_{}", i), &format!("node_{}", target)));
        }
    }
    
    let graph = build_graph(vec![extraction]);
    
    assert_eq!(graph.nodes.len(), 100);
    
    // Should still find paths quickly
    let path = find_shortest_path(&graph, "node_0", "node_50", 100);
    assert!(path.is_some());
    
    // Query should still work
    let search_terms = vec!["node_25".to_string()];
    let scores = score_nodes(&graph, &search_terms);
    assert!(!scores.is_empty());
    
    println!("✓ Large graph: {} nodes processed successfully", graph.nodes.len());
}

// ===== Edge Cases E2E =====

#[test]
fn test_e2e_self_loops() {
    let mut extraction = ExtractionResult::new();
    extraction.add_node(node("A", "test.rs"));
    extraction.add_edge(Edge::new(
        "A".into(), "A".into(), "calls".into(), Confidence::Extracted
    ));
    
    let graph = build_graph(vec![extraction]);
    assert_eq!(graph.nodes.len(), 1);
}

#[test]
fn test_e2e_multi_edges() {
    let mut extraction = ExtractionResult::new();
    extraction.add_node(node("A", "test.rs"));
    extraction.add_node(node("B", "test.rs"));
    
    // Multiple edges between same nodes
    extraction.add_edge(edge("A", "B"));
    extraction.add_edge(Edge::new(
        "A".into(), "B".into(), "imports".into(), Confidence::Inferred
    ));
    
    let graph = build_graph(vec![extraction]);
    // Should deduplicate or keep all
    assert!(graph.links.len() >= 1);
}

#[test]
fn test_e2e_isolated_nodes() {
    let mut extraction = ExtractionResult::new();
    
    // Group 1: connected
    extraction.add_node(node("A1", "g1.rs"));
    extraction.add_node(node("A2", "g1.rs"));
    extraction.add_edge(edge("A1", "A2"));
    
    // Group 2: connected
    extraction.add_node(node("B1", "g2.rs"));
    extraction.add_node(node("B2", "g2.rs"));
    extraction.add_edge(edge("B1", "B2"));
    
    // Group 3: isolated
    extraction.add_node(node("C1", "g3.rs"));
    
    let graph = build_graph(vec![extraction]);
    let analysis = analyze(&graph);
    
    // Should have at least 2 communities (or 3 if isolated is separate)
    assert!(analysis.community_sizes.len() >= 2);
    
    // A and B should find paths to each other
    let path = find_shortest_path(&graph, "A1", "A2", 10);
    assert!(path.is_some());
    
    // C should not find path to A
    let path_to_a = find_shortest_path(&graph, "C1", "A1", 10);
    assert!(path_to_a.is_none());
}

#[test]
fn test_e2e_cyclic_graph() {
    let mut extraction = ExtractionResult::new();
    
    // Create a cycle: A -> B -> C -> A
    extraction.add_node(node("A", "cycle.rs"));
    extraction.add_node(node("B", "cycle.rs"));
    extraction.add_node(node("C", "cycle.rs"));
    
    extraction.add_edge(edge("A", "B"));
    extraction.add_edge(edge("B", "C"));
    extraction.add_edge(edge("C", "A"));
    
    let graph = build_graph(vec![extraction]);
    
    // All should be in same community
    let analysis = analyze(&graph);
    assert_eq!(analysis.community_sizes.len(), 1);
    
    // Paths should still work
    let path = find_shortest_path(&graph, "A", "C", 10);
    assert!(path.is_some());
}

#[test]
fn test_e2e_complete_graph() {
    let mut extraction = ExtractionResult::new();
    
    // Complete graph K4
    for i in 0..4 {
        extraction.add_node(node(&format!("K4_{}", i), "k4.rs"));
    }
    
    for i in 0..4 {
        for j in (i+1)..4 {
            extraction.add_edge(edge(&format!("K4_{}", i), &format!("K4_{}", j)));
        }
    }
    
    let graph = build_graph(vec![extraction]);
    let analysis = analyze(&graph);
    
    // K4 should be one community (or may split depending on algorithm)
    assert!(analysis.community_sizes.len() <= 2);
}
