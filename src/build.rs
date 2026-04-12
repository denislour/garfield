//! Graph building module

use crate::types::{ExtractionResult, GraphData, Node, Edge};
use crate::cluster::{cluster, add_communities, split_oversized};
use std::collections::HashSet;

/// Deduplicate edges by source+target (keep first occurrence)
fn dedup_edges(edges: Vec<Edge>) -> Vec<Edge> {
    let mut seen: HashSet<(String, String)> = HashSet::new();
    let mut result = Vec::new();
    
    for edge in edges {
        let key = (edge.source.clone(), edge.target.clone());
        if seen.insert(key) {
            result.push(edge);
        }
    }
    
    result
}

/// Build graph từ extraction results
pub fn build_graph(extractions: Vec<ExtractionResult>) -> GraphData {
    let mut nodes: Vec<Node> = Vec::new();
    let mut edges: Vec<Edge> = Vec::new();
    
    // Collect all nodes and edges
    for extraction in &extractions {
        nodes.extend(extraction.nodes.clone());
        edges.extend(extraction.links.clone());
    }
    
    // Deduplicate nodes by ID
    nodes.sort_by_key(|n| n.id.clone());
    nodes.dedup_by_key(|n| n.id.clone());
    
    // Deduplicate edges by source+target (keep first/Extracted)
    edges = dedup_edges(edges);
    
    // Create graph without communities first
    let mut graph = GraphData::new(nodes, edges, 0);
    
    // Run community detection
    let community_result = cluster(&graph);
    
    // Add communities to nodes
    add_communities(&mut graph, &community_result.assignments);
    
    // Split oversized communities (>25 nodes)
    split_oversized(&mut graph, 25);
    
    graph
}

/// Merge extractions from multiple files
pub fn merge_extractions(results: Vec<ExtractionResult>) -> ExtractionResult {
    let mut all_nodes = Vec::new();
    let mut all_edges = Vec::new();
    
    for result in results {
        all_nodes.extend(result.nodes);
        all_edges.extend(result.links);
    }
    
    // Deduplicate nodes by ID
    all_nodes.sort_by_key(|n| n.id.clone());
    all_nodes.dedup_by_key(|n| n.id.clone());
    
    // Deduplicate edges by source+target
    all_edges = dedup_edges(all_edges);
    
    ExtractionResult {
        nodes: all_nodes,
        links: all_edges,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Node, Edge, Confidence};

    #[test]
    fn test_build_graph() {
        let mut extraction1 = ExtractionResult::new();
        extraction1.add_node(Node::new("a.py:foo".into(), "foo".into(), "a.py".into(), "L1".into()));
        extraction1.add_node(Node::new("a.py:bar".into(), "bar".into(), "a.py".into(), "L1".into()));
        extraction1.add_edge(Edge::new("a.py:foo".into(), "a.py:bar".into(), "calls".into(), Confidence::Extracted));
        
        let mut extraction2 = ExtractionResult::new();
        extraction2.add_node(Node::new("b.py:baz".into(), "baz".into(), "b.py".into(), "L1".into()));
        extraction2.add_edge(Edge::new("a.py:foo".into(), "b.py:baz".into(), "imports".into(), Confidence::Inferred));
        
        let graph = build_graph(vec![extraction1, extraction2]);
        
        assert_eq!(graph.nodes.len(), 3);
        assert_eq!(graph.links.len(), 2);
        assert!(graph.metadata.communities > 0);
    }

    #[test]
    fn test_merge_extractions() {
        let mut extraction1 = ExtractionResult::new();
        extraction1.add_node(Node::new("a.py:foo".into(), "foo".into(), "a.py".into(), "L1".into()));
        
        let mut extraction2 = ExtractionResult::new();
        extraction2.add_node(Node::new("a.py:foo".into(), "foo".into(), "a.py".into(), "L2".into())); // Duplicate
        extraction2.add_node(Node::new("b.py:bar".into(), "bar".into(), "b.py".into(), "L1".into()));
        
        let merged = merge_extractions(vec![extraction1, extraction2]);
        
        // Should have only 2 nodes (foo deduplicated)
        assert_eq!(merged.nodes.len(), 2);
    }
}
