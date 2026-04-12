//! E2E tests - Full pipeline

use garfield::{analyze, build_graph, detect, extract_file, find_shortest_path, score_nodes, validate_graph};
use std::path::Path;

#[test]
fn test_e2e_full_pipeline() {
    let dir = Path::new("../graphify");
    if !dir.exists() { println!("SKIP"); return; }
    let files = detect::detect(dir).unwrap();
    let code: Vec<_> = files.files.into_iter().filter(|f| f.file_type == garfield::FileType::Code).collect();
    assert!(!code.is_empty());
    let mut ext = None;
    for f in &code {
        if let Ok(src) = std::fs::read_to_string(&f.path) {
            if let Ok(r) = extract_file(&f.path, &src) {
                if !r.nodes.is_empty() { ext = Some(r); break; }
            }
        }
    }
    let ext = match ext { Some(e) => e, None => return };
    let g = build_graph(vec![ext]);
    assert!(g.nodes.len() > 0);
    let _ = validate_graph(&g);
    let _ = analyze(&g);
}

#[test]
fn test_e2e_query() {
    let mut e = garfield::ExtractionResult::new();
    e.add_node(garfield::Node::new("t:hello".into(), "hello".into(), "t.py".into(), "L1".into()));
    e.add_node(garfield::Node::new("t:world".into(), "world".into(), "t.py".into(), "L2".into()));
    let g = build_graph(vec![e]);
    let terms = vec!["hello".to_string()];
    let scores = score_nodes(&g, &terms);
    assert!(!scores.is_empty());
    assert_eq!(scores[0].1, "t:hello");
}

#[test]
fn test_e2e_path() {
    let mut e = garfield::ExtractionResult::new();
    e.add_node(garfield::Node::new("a:A".into(), "A".into(), "a.rs".into(), "L1".into()));
    e.add_node(garfield::Node::new("a:B".into(), "B".into(), "a.rs".into(), "L2".into()));
    e.add_node(garfield::Node::new("a:C".into(), "C".into(), "a.rs".into(), "L3".into()));
    e.add_edge(garfield::Edge::new("a:A".into(), "a:B".into(), "calls".into(), garfield::Confidence::Extracted));
    e.add_edge(garfield::Edge::new("a:B".into(), "a:C".into(), "calls".into(), garfield::Confidence::Extracted));
    let g = build_graph(vec![e]);
    let p = find_shortest_path(&g, "a:A", "a:C", 10);
    assert!(p.is_some());
    assert!(p.unwrap().len() <= 3);
}

#[test]
fn test_e2e_confidence() {
    let mut e = garfield::ExtractionResult::new();
    e.add_node(garfield::Node::new("a:A".into(), "A".into(), "a.rs".into(), "L1".into()));
    e.add_node(garfield::Node::new("a:B".into(), "B".into(), "a.rs".into(), "L1".into()));
    e.add_edge(garfield::Edge::new("a:A".into(), "a:B".into(), "calls".into(), garfield::Confidence::Extracted));
    e.add_edge(garfield::Edge::new("a:A".into(), "a:B".into(), "imports".into(), garfield::Confidence::Inferred));
    let g = build_graph(vec![e]);
    let a = analyze(&g);
    assert_eq!(a.confidence_stats.extracted, 1);
    assert_eq!(a.confidence_stats.inferred, 1);
}

#[test]
fn test_e2e_grid() {
    let mut e = garfield::ExtractionResult::new();
    for i in 0..9 {
        let n = (b'A' + i) as char;
        e.add_node(garfield::Node::new(format!("g:{}", n).into(), n.to_string(), "g.rs".into(), format!("L{}", i)));
    }
    // Row edges
    e.add_edge(garfield::Edge::new("g:A".into(), "g:B".into(), "calls".into(), garfield::Confidence::Extracted));
    e.add_edge(garfield::Edge::new("g:B".into(), "g:C".into(), "calls".into(), garfield::Confidence::Extracted));
    e.add_edge(garfield::Edge::new("g:D".into(), "g:E".into(), "calls".into(), garfield::Confidence::Extracted));
    e.add_edge(garfield::Edge::new("g:E".into(), "g:F".into(), "calls".into(), garfield::Confidence::Extracted));
    e.add_edge(garfield::Edge::new("g:G".into(), "g:H".into(), "calls".into(), garfield::Confidence::Extracted));
    e.add_edge(garfield::Edge::new("g:H".into(), "g:I".into(), "calls".into(), garfield::Confidence::Extracted));
    // Col edges
    e.add_edge(garfield::Edge::new("g:A".into(), "g:D".into(), "calls".into(), garfield::Confidence::Extracted));
    e.add_edge(garfield::Edge::new("g:D".into(), "g:G".into(), "calls".into(), garfield::Confidence::Extracted));
    e.add_edge(garfield::Edge::new("g:B".into(), "g:E".into(), "calls".into(), garfield::Confidence::Extracted));
    e.add_edge(garfield::Edge::new("g:E".into(), "g:H".into(), "calls".into(), garfield::Confidence::Extracted));
    e.add_edge(garfield::Edge::new("g:C".into(), "g:F".into(), "calls".into(), garfield::Confidence::Extracted));
    e.add_edge(garfield::Edge::new("g:F".into(), "g:I".into(), "calls".into(), garfield::Confidence::Extracted));
    let g = build_graph(vec![e]);
    let p = find_shortest_path(&g, "g:A", "g:I", 10);
    assert!(p.is_some());
}
