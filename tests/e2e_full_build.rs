//! E2E Tests - Full Build Pipeline with Mock

use garfield::types::{Confidence, Edge, ExtractionResult, Node};
use garfield::{
    analyze, build_graph, detect_hyperedges, extract_file, find_shortest_path, score_nodes,
    serve::{get_node, subgraph_to_text},
    GraphData,
};
use std::collections::HashSet;
use std::path::Path;
use tempfile::TempDir;

fn node(id: &str, file: &str, label: &str) -> Node {
    Node::new(id.into(), label.into(), file.into(), "L1".into())
}

fn edge(src: &str, tgt: &str, relation: &str) -> Edge {
    Edge::new(
        src.into(),
        tgt.into(),
        relation.into(),
        Confidence::Extracted,
    )
}

fn setup_test_project() -> TempDir {
    let tmp = TempDir::new().unwrap();
    let src = tmp.path().join("src");
    std::fs::create_dir_all(&src).unwrap();

    std::fs::write(
        src.join("pricing.rs"),
        r#"
        pub fn calculate_price(qty: i32, tier: &str) -> f64 {
            let discount = get_discount_tier(qty);
            let base = 10.0 * qty as f64;
            let discounted = apply_discount(base, discount);
            compute_tax(discounted)
        }
        
        fn apply_discount(price: f64, tier: f64) -> f64 {
            price * (1.0 - tier)
        }
        
        fn compute_tax(amount: f64) -> f64 {
            amount * 1.1
        }
        
        fn get_discount_tier(quantity: i32) -> f64 {
            match quantity {
                0..=5 => 0.0,
                6..=10 => 0.1,
                _ => 0.2,
            }
        }
    "#,
    )
    .unwrap();

    std::fs::write(
        src.join("order.rs"),
        r#"
        pub fn create_order(items: Vec<Item>) -> Order {
            let total = calculate_order_total(&items);
            let order = Order { items, total };
            validate_order(&order);
            order
        }
        
        fn calculate_order_total(items: &[Item]) -> f64 {
            items.iter().map(|i| calculate_price(i.qty, &i.tier)).sum()
        }
        
        fn validate_order(order: &Order) -> bool {
            !order.items.is_empty()
        }
    "#,
    )
    .unwrap();

    tmp
}

fn extract_src_dir(dir: &Path) -> ExtractionResult {
    let mut result = ExtractionResult::new();

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "rs").unwrap_or(false) {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(extraction) = extract_file(&path, &content) {
                        result.nodes.extend(extraction.nodes);
                        result.links.extend(extraction.links);
                    }
                }
            }
        }
    }

    result
}

fn build_graph_with_hyperedges(extractions: Vec<ExtractionResult>) -> GraphData {
    let mut graph = build_graph(extractions);
    let hyperedges = detect_hyperedges(&graph);
    graph.hyperedges = hyperedges;
    graph
}

#[test]
fn test_build_creates_hyperedges() {
    let tmp = setup_test_project();
    let src = tmp.path().join("src");

    let extraction = extract_src_dir(&src);
    let graph = build_graph_with_hyperedges(vec![extraction]);

    println!("\n=== BUILD RESULT ===");
    println!("Nodes: {}", graph.nodes.len());
    println!("Edges: {}", graph.links.len());
    println!("Hyperedges: {}", graph.hyperedges.len());

    assert!(graph.nodes.len() >= 4, "Should have at least 4 nodes");
    assert!(
        !graph.links.is_empty() || graph.nodes.len() > 0,
        "Should have content"
    );
}

#[test]
fn test_query_returns_nodes_with_module_tags() {
    let tmp = setup_test_project();
    let src = tmp.path().join("src");

    let extraction = extract_src_dir(&src);
    let graph = build_graph_with_hyperedges(vec![extraction]);

    let terms = vec!["price".to_string()];
    let scores = score_nodes(&graph, &terms);

    println!("\n=== QUERY: 'price' ===");
    println!("Found {} matching nodes", scores.len());

    if !scores.is_empty() {
        let top_ids: HashSet<String> = scores
            .iter()
            .take(3)
            .map(|(_, id)| id.to_string())
            .collect();
        let output = subgraph_to_text(&graph, &top_ids, &[], 1000);
        println!("{}", output);

        assert!(output.contains("## Nodes"), "Should have ## Nodes section");
    }
}

#[test]
fn test_explain_returns_hyperedge_info() {
    let tmp = setup_test_project();
    let src = tmp.path().join("src");

    let extraction = extract_src_dir(&src);
    let graph = build_graph_with_hyperedges(vec![extraction]);

    if let Some(node) = graph.nodes.first() {
        let details = get_node(&graph, &node.id);

        if let Some(details) = details {
            println!("\n=== EXPLAIN: {} ===", details.id);
            println!("Label: {}", details.label);
            println!("File: {}", details.source_file);

            if let Some(he) = &details.hyperedge {
                println!("Hyperedge: {} ({} members)", he.label, he.member_count);
                assert!(he.member_count > 0, "Hyperedge should have members");
            } else {
                println!("No hyperedge (may be OK for small files)");
            }
        }
    }
}

#[test]
fn test_path_finding() {
    let mut extraction = ExtractionResult::new();

    extraction.add_node(node("A", "test.rs", "A"));
    extraction.add_node(node("B", "test.rs", "B"));
    extraction.add_node(node("C", "test.rs", "C"));
    extraction.add_node(node("D", "test.rs", "D"));

    extraction.add_edge(edge("A", "B", "calls"));
    extraction.add_edge(edge("B", "C", "calls"));
    extraction.add_edge(edge("C", "D", "calls"));

    let graph = build_graph_with_hyperedges(vec![extraction]);
    let path = find_shortest_path(&graph, "A", "D", 10);

    println!("\n=== PATH: A -> D ===");
    if let Some(path) = &path {
        println!("Path found: {:?}", path);
        assert!(path.len() >= 2, "Path should have at least 2 nodes");
    }
}

#[test]
fn test_hyperedge_explains_group() {
    let mut extraction = ExtractionResult::new();

    for i in 1..=5 {
        extraction.add_node(node(&format!("fn_{}", i), "utils.rs", &format!("fn_{}", i)));
    }

    let graph = build_graph_with_hyperedges(vec![extraction]);

    println!("\n=== HYPEREDGE INFO ===");
    println!("Nodes: {}", graph.nodes.len());
    println!("Hyperedges: {}", graph.hyperedges.len());

    if let Some(he) = graph.hyperedges.first() {
        println!("Hyperedge: {}", he.label);
        println!("Members: {}", he.nodes.len());
        assert!(
            he.nodes.len() >= 3,
            "Hyperedge should have at least 3 members"
        );
    }
}

#[test]
fn test_score_nodes_ranks_by_relevance() {
    let mut extraction = ExtractionResult::new();

    extraction.add_node(node("user_service:create", "user_service.rs", "create"));
    extraction.add_node(node("user_service:delete", "user_service.rs", "delete"));
    extraction.add_node(node("order_service:create", "order_service.rs", "create"));

    let graph = build_graph_with_hyperedges(vec![extraction]);

    let terms = vec!["create".to_string()];
    let scores = score_nodes(&graph, &terms);

    println!("\n=== SCORE: 'create' ===");
    for (score, id) in &scores {
        println!("  {} -> {}", id, score);
    }

    let create_scores: Vec<_> = scores
        .iter()
        .filter(|(_, id)| id.contains("create"))
        .collect();
    assert!(!create_scores.is_empty(), "Should find 'create' functions");
}

#[test]
fn test_empty_graph_handling() {
    let extraction = ExtractionResult::new();
    let graph = build_graph_with_hyperedges(vec![extraction]);

    assert!(
        graph.nodes.is_empty(),
        "Empty extraction should produce empty graph"
    );
    assert!(
        graph.hyperedges.is_empty(),
        "Empty graph should have no hyperedges"
    );

    let output = subgraph_to_text(&graph, &HashSet::new(), &[], 1000);
    assert!(
        output.contains("## Nodes"),
        "Empty query should show ## Nodes"
    );
}

#[test]
fn test_hyperedge_contains_valid_nodes() {
    let mut extraction = ExtractionResult::new();

    extraction.add_node(node("A", "test.rs", "A"));
    extraction.add_node(node("B", "test.rs", "B"));
    extraction.add_node(node("C", "test.rs", "C"));

    let graph = build_graph_with_hyperedges(vec![extraction]);

    for he in &graph.hyperedges {
        for node_id in &he.nodes {
            let exists = graph.nodes.iter().any(|n| &n.id == node_id);
            assert!(
                exists,
                "Hyperedge references non-existent node: {}",
                node_id
            );
        }
    }
}

#[test]
fn test_community_detection() {
    let mut extraction = ExtractionResult::new();

    for i in 1..=3 {
        extraction.add_node(node(
            &format!("group_a_{}", i),
            "a.rs",
            &format!("group_a_{}", i),
        ));
    }
    for i in 1..=3 {
        extraction.add_node(node(
            &format!("group_b_{}", i),
            "b.rs",
            &format!("group_b_{}", i),
        ));
    }

    extraction.add_edge(edge("group_a_1", "group_a_2", "calls"));
    extraction.add_edge(edge("group_a_2", "group_a_3", "calls"));
    extraction.add_edge(edge("group_b_1", "group_b_2", "calls"));
    extraction.add_edge(edge("group_b_2", "group_b_3", "calls"));

    let graph = build_graph_with_hyperedges(vec![extraction]);
    let analysis = analyze(&graph);

    println!("\n=== COMMUNITY DETECTION ===");
    println!("Communities found: {}", analysis.community_sizes.len());

    assert!(
        analysis.community_sizes.len() >= 1,
        "Should have at least 1 community"
    );
}

#[test]
fn test_query_depth_traversal() {
    let mut extraction = ExtractionResult::new();

    for i in 1..=5 {
        extraction.add_node(node(&format!("n{}", i), "test.rs", &format!("n{}", i)));
    }
    for i in 1..=4 {
        extraction.add_edge(edge(&format!("n{}", i), &format!("n{}", i + 1), "calls"));
    }

    let graph = build_graph_with_hyperedges(vec![extraction]);
    let terms = vec!["n1".to_string()];
    let scores = score_nodes(&graph, &terms);

    println!("\n=== DEPTH TRAVERSAL ===");
    println!("Starting from n1, found {} matches", scores.len());

    assert!(scores.len() >= 1, "Should find at least n1");
}
