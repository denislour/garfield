//! Graph building module

use crate::types::{ExtractionResult, GraphData, Node, Edge};
use crate::cluster::{cluster, add_communities, split_oversized};

/// Build graph từ extraction results
pub fn build_graph(extractions: Vec<ExtractionResult>) -> GraphData {
    let mut nodes: Vec<Node> = Vec::new();
    let mut edges: Vec<Edge> = Vec::new();
    
    // Collect all nodes and edges
    for extraction in &extractions {
        nodes.extend(extraction.nodes.clone());
        edges.extend(extraction.edges.clone());
    }
    
    // Deduplicate nodes by ID
    nodes.sort_by_key(|n| n.id.clone());
    nodes.dedup_by_key(|n| n.id.clone());
    
    // Deduplicate edges
    edges.sort();
    edges.dedup();
    
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
        all_edges.extend(result.edges);
    }
    
    // Deduplicate nodes by ID
    all_nodes.sort_by_key(|n| n.id.clone());
    all_nodes.dedup_by_key(|n| n.id.clone());
    
    // Deduplicate edges
    all_edges.sort();
    all_edges.dedup();
    
    ExtractionResult {
        nodes: all_nodes,
        edges: all_edges,
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
        assert_eq!(graph.edges.len(), 2);
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
