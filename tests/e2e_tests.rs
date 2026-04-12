//! Garfield E2E tests - Full pipeline tests

use garfield::{
    analyze, build_graph, detect, extract_file, find_shortest_path, 
    score_nodes, validate_graph,
};
use std::path::Path;

#[test]
fn test_e2e_full_pipeline() {
    let test_dir = Path::new("../graphify");
    if !test_dir.exists() {
        println!("SKIP: graphify not found");
        return;
    }

    // 1. Detect
    let detected = detect::detect(test_dir).expect("detect should work");
    let code_files: Vec<_> = detected.files
        .into_iter()
        .filter(|f| f.file_type == garfield::FileType::Code)
        .collect();
    assert!(!code_files.is_empty());

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
        None => return,
    };

    // 3. Build graph
    let graph = build_graph(vec![extraction]);
    assert!(graph.nodes.len() > 0);

    // 4. Validate
    let _ = validate_graph(&graph);

    // 5. Analyze
    let analysis = analyze(&graph);
    println!("Graph: {} nodes, {} edges, {} communities",
        graph.nodes.len(), graph.links.len(), analysis.community_sizes.len());
}

#[test]
fn test_e2e_query() {
    let mut extraction = garfield::ExtractionResult::new();
    extraction.add_node(garfield::Node::new("test.py:hello".into(), "hello".into(), "test.py".into(), "L1".into()));
    extraction.add_node(garfield::Node::new("test.py:world".into(), "world".into(), "test.py".into(), "L2".into()));

    let graph = build_graph(vec![extraction]);
    let terms = vec!["hello".to_string()];
    let scores = score_nodes(&graph, &terms);

    assert!(!scores.is_empty());
    assert_eq!(scores[0].1, "test.py:hello");
}

#[test]
fn test_e2e_path() {
    let mut extraction = garfield::ExtractionResult::new();
    extraction.add_node(garfield::Node::new("a:A".into(), "A".into(), "a.rs".into(), "L1".into()));
    extraction.add_node(garfield::Node::new("a:B".into(), "B".into(), "a.rs".into(), "L2".into()));
    extraction.add_node(garfield::Node::new("a:C".into(), "C".into(), "a.rs".into(), "L3".into()));

    extraction.add_edge(garfield::Edge::new("a:A".into(), "a:B".into(), "calls".into(), garfield::Confidence::Extracted));
    extraction.add_edge(garfield::Edge::new("a:B".into(), "a:C".into(), "calls".into(), garfield::Confidence::Extracted));

    let graph = build_graph(vec![extraction]);
    let path = find_shortest_path(&graph, "a:A", "a:C", 10);
    
    assert!(path.is_some());
    assert!(path.unwrap().len() <= 3);
}

#[test]
fn test_e2e_confidence() {
    let mut extraction = garfield::ExtractionResult::new();
    extraction.add_node(garfield::Node::new("a:A".into(), "A".into(), "a.rs".into(), "L1".into()));
    extraction.add_node(garfield::Node::new("a:B".into(), "B".into(), "a.rs".into(), "L1".into()));

    extraction.add_edge(garfield::Edge::new("a:A".into(), "a:B".into(), "calls".into(), garfield::Confidence::Extracted));
    extraction.add_edge(garfield::Edge::new("a:A".into(), "a:B".into(), "imports".into(), garfield::Confidence::Inferred));

    let graph = build_graph(vec![extraction]);
    let analysis = analyze(&graph);

    assert_eq!(analysis.confidence_stats.extracted, 1);
    assert_eq!(analysis.confidence_stats.inferred, 1);
}

#[test]
fn test_e2e_complex_grid() {
    let mut extraction = garfield::ExtractionResult::new();

    // 3x3 grid
    for i in 0..9 {
        let name = (b'A' + i) as char;
        extraction.add_node(garfield::Node::new(
            format!("grid:{}", name).into(),
            name.to_string(),
            "grid.rs".into(),
            format!("L{}", i),
        ));
    }

    // Row edges
    extraction.add_edge(garfield::Edge::new("grid:A".into(), "grid:B".into(), "calls".into(), garfield::Confidence::Extracted));
    extraction.add_edge(garfield::Edge::new("grid:B".into(), "grid:C".into(), "calls".into(), garfield::Confidence::Extracted));
    extraction.add_edge(garfield::Edge::new("grid:D".into(), "grid:E".into(), "calls".into(), garfield::Confidence::Extracted));
    extraction.add_edge(garfield::Edge::new("grid:E".into(), "grid:F".into(), "calls".into(), garfield::Confidence::Extracted));
    extraction.add_edge(garfield::Edge::new("grid:G".into(), "grid:H".into(), "calls".into(), garfield::Confidence::Extracted));
    extraction.add_edge(garfield::Edge::new("grid:H".into(), "grid:I".into(), "calls".into(), garfield::Confidence::Extracted));

    // Column edges
    extraction.add_edge(garfield::Edge::new("grid:A".into(), "grid:D".into(), "calls".into(), garfield::Confidence::Extracted));
    extraction.add_edge(garfield::Edge::new("grid:D".into(), "grid:G".into(), "calls".into(), garfield::Confidence::Extracted));
    extraction.add_edge(garfield::Edge::new("grid:B".into(), "grid:E".into(), "calls".into(), garfield::Confidence::Extracted));
    extraction.add_edge(garfield::Edge::new("grid:E".into(), "grid:H".into(), "calls".into(), garfield::Confidence::Extracted));
    extraction.add_edge(garfield::Edge::new("grid:C".into(), "grid:F".into(), "calls".into(), garfield::Confidence::Extracted));
    extraction.add_edge(garfield::Edge::new("grid:F".into(), "grid:I".into(), "calls".into(), garfield::Confidence::Extracted));

    let graph = build_graph(vec![extraction]);
    let path = find_shortest_path(&graph, "grid:A", "grid:I", 10);
    
    assert!(path.is_some());
    println!("Grid path A→I: {} hops", path.unwrap().len() - 1);
}
