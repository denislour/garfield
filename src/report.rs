//! Report generation module

use std::fs;
use std::path::Path;
use crate::types::{GraphData, Confidence};
use crate::analyze::analyze;

/// Generate GRAPH_REPORT.md
pub fn generate_report(graph: &GraphData, output_path: &Path) -> anyhow::Result<()> {
    let analysis = analyze(graph);
    
    let mut content = String::new();
    
    // Header
    content.push_str("# Graph Report\n\n");
    
    // Corpus check
    content.push_str(&format!(
        "## Corpus Check\n- {} files · ~{} nodes\n- Verdict: {}\n\n",
        graph.metadata.total_nodes,
        graph.metadata.total_edges,
        corpus_verdict(graph)
    ));
    
    // Summary
    content.push_str(&format!(
        "## Summary\n- {} nodes · {} edges · {} communities detected\n",
        graph.metadata.total_nodes,
        graph.metadata.total_edges,
        graph.metadata.communities
    ));
    
    // Confidence breakdown
    let total = graph.edges.len();
    if total > 0 {
        let extracted = analysis.confidence_stats.extracted;
        let inferred = analysis.confidence_stats.inferred;
        let ambiguous = analysis.confidence_stats.ambiguous;
        
        content.push_str(&format!(
            "- Extraction: {}% EXTRACTED · {}% INFERRED · {}% AMBIGUOUS\n\n",
            (extracted * 100) / total,
            (inferred * 100) / total,
            (ambiguous * 100) / total
        ));
    } else {
        content.push('\n');
    }
    
    // God nodes
    content.push_str("## God Nodes (most connected - your core abstractions)\n");
    for (i, god) in analysis.god_nodes.iter().enumerate().take(10) {
        content.push_str(&format!(
            "{}. `{}` - {} edges\n",
            i + 1,
            god.node.label,
            god.degree
        ));
    }
    content.push('\n');
    
    // Surprising connections
    content.push_str("## Surprising Connections (you probably didn't know these)\n");
    for conn in analysis.surprising_connections.iter().take(5) {
        let src_label = graph
            .nodes
            .iter()
            .find(|n| n.id == conn.source)
            .map(|n| n.label.as_str())
            .unwrap_or(&conn.source);
        
        let tgt_label = graph
            .nodes
            .iter()
            .find(|n| n.id == conn.target)
            .map(|n| n.label.as_str())
            .unwrap_or(&conn.target);
        
        content.push_str(&format!(
            "- `{}` --{}--> `{}` [{}]\n",
            src_label,
            conn.relation,
            tgt_label,
            match conn.confidence {
                Confidence::Extracted => "EXTRACTED",
                Confidence::Inferred => "INFERRED",
                Confidence::Ambiguous => "AMBIGUOUS",
            }
        ));
    }
    content.push('\n');
    
    // Community summary
    content.push_str("## Communities\n\n");
    let mut sorted_communities: Vec<_> = analysis.community_sizes.iter().collect();
    sorted_communities.sort_by(|a, b| b.1.cmp(a.1));
    
    for (_i, (cid, size)) in sorted_communities.iter().take(10).enumerate() {
        content.push_str(&format!("### Community {} ({} nodes)\n", cid, size));
        
        // List some nodes in this community
        let nodes_in_comm: Vec<_> = graph
            .nodes
            .iter()
            .filter(|n| n.community == Some(**cid))
            .take(5)
            .map(|n| n.label.as_str())
            .collect();
        
        content.push_str(&format!("Nodes: {}\n\n", nodes_in_comm.join(", ")));
    }
    
    // Ensure directory exists
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }
    
    // Write report
    fs::write(output_path, content)?;
    
    println!("Report generated: {}", output_path.display());
    Ok(())
}

/// Determine corpus verdict based on size
fn corpus_verdict(graph: &GraphData) -> String {
    let nodes = graph.metadata.total_nodes;
    if nodes < 10 {
        "corpus too small for meaningful graph".to_string()
    } else if nodes < 100 {
        "corpus is small but graph structure adds value".to_string()
    } else if nodes < 1000 {
        "corpus is large enough that graph structure adds value".to_string()
    } else {
        "corpus is very large - graph is essential for navigation".to_string()
    }
}

/// Print report to stdout
pub fn print_report(graph: &GraphData) {
    let analysis = analyze(graph);
    
    println!("=== GRAPH REPORT ===");
    println!("Nodes: {}", graph.metadata.total_nodes);
    println!("Edges: {}", graph.metadata.total_edges);
    println!("Communities: {}", graph.metadata.communities);
    
    println!("\n--- GOD NODES ---");
    for (i, god) in analysis.god_nodes.iter().enumerate().take(5) {
        println!("{}. {} ({} edges)", i + 1, god.node.label, god.degree);
    }
    
    println!("\n--- SURPRISING CONNECTIONS ---");
    for conn in analysis.surprising_connections.iter().take(3) {
        println!("{} --{}--> {}", conn.source, conn.relation, conn.target);
    }
    
    println!("\n--- CONFIDENCE ---");
    let stats = &analysis.confidence_stats;
    println!("EXTRACTED: {}", stats.extracted);
    println!("INFERRED: {}", stats.inferred);
    println!("AMBIGUOUS: {}", stats.ambiguous);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Node, Edge, Confidence, GraphMetadata};

    fn create_test_graph() -> GraphData {
        let nodes = vec![
            Node::new("a.py:A".into(), "A".into(), "a.py".into(), "L1".into()),
            Node::new("a.py:B".into(), "B".into(), "a.py".into(), "L1".into()),
            Node::new("b.py:C".into(), "C".into(), "b.py".into(), "L1".into()),
        ];
        
        let edges = vec![
            Edge::new("a.py:A".into(), "a.py:B".into(), "calls".into(), Confidence::Extracted),
            Edge::new("a.py:B".into(), "b.py:C".into(), "imports".into(), Confidence::Inferred),
        ];
        
        GraphData {
            nodes,
            edges,
            metadata: GraphMetadata::new(3, 2, 2),
        }
    }

    #[test]
    fn test_corpus_verdict() {
        let small = GraphData::new(vec![], vec![], 0);
        assert!(corpus_verdict(&small).contains("small"));
        
        let large = GraphData::new(
            (0..100).map(|i| Node::new(format!("n{}", i), format!("n{}", i), "test.py".into(), "L1".into())).collect(),
            vec![],
            10
        );
        assert!(corpus_verdict(&large).contains("large"));
    }
}
