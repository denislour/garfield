//! Graph building module

use crate::community::{add_communities, cluster, split_oversized};
use crate::types::{Edge, ExtractionResult, GraphData, Hyperedge, Node};
use std::collections::HashSet;

fn dedup_edges(edges: Vec<Edge>) -> Vec<Edge> {
    let mut seen: HashSet<(String, String, String)> = HashSet::new();
    let mut result = Vec::new();

    for edge in edges {
        let key = (
            edge.source.clone(),
            edge.target.clone(),
            edge.relation.clone(),
        );
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
    let mut hyperedges: Vec<Hyperedge> = Vec::new();

    // Collect all nodes, edges, and hyperedges
    for extraction in &extractions {
        nodes.extend(extraction.nodes.clone());
        edges.extend(extraction.links.clone());
        hyperedges.extend(extraction.hyperedges.clone());
    }

    // Deduplicate nodes by ID
    nodes.sort_by_key(|n| n.id.clone());
    nodes.dedup_by_key(|n| n.id.clone());

    // Deduplicate edges by source+target (keep first/Extracted)
    edges = dedup_edges(edges);

    // Create graph without communities first
    let mut graph = GraphData::new(nodes, edges, 0);
    graph.hyperedges = hyperedges;

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
    let mut all_hyperedges = Vec::new();

    for result in results {
        all_nodes.extend(result.nodes);
        all_edges.extend(result.links);
        all_hyperedges.extend(result.hyperedges);
    }

    // Deduplicate nodes by ID
    all_nodes.sort_by_key(|n| n.id.clone());
    all_nodes.dedup_by_key(|n| n.id.clone());

    // Deduplicate edges by source+target
    all_edges = dedup_edges(all_edges);

    ExtractionResult {
        nodes: all_nodes,
        links: all_edges,
        hyperedges: all_hyperedges,
    }
}

/// Merge new extraction into existing graph (for --update flow)
/// New nodes/edges are added, existing ones are kept
pub fn merge_into_graph(existing: &mut GraphData, new_extraction: ExtractionResult) {
    let mut new_nodes: Vec<Node> = Vec::new();
    let mut new_edges: Vec<Edge> = Vec::new();
    let mut new_hyperedges: Vec<Hyperedge> = Vec::new();

    // Collect from new extraction
    new_nodes.extend(new_extraction.nodes);
    new_edges.extend(new_extraction.links);
    new_hyperedges.extend(new_extraction.hyperedges);

    // Deduplicate new nodes against existing
    let existing_ids: HashSet<&str> = existing.nodes.iter().map(|n| n.id.as_str()).collect();
    let mut unique_new_nodes: Vec<Node> = Vec::new();
    for node in new_nodes {
        if !existing_ids.contains(node.id.as_str()) {
            unique_new_nodes.push(node);
        }
    }

    // Deduplicate new edges against existing
    let existing_edge_keys: HashSet<(String, String)> = existing
        .links
        .iter()
        .map(|e| (e.source.clone(), e.target.clone()))
        .collect();
    let mut unique_new_edges: Vec<Edge> = Vec::new();
    for edge in new_edges {
        let key = (edge.source.clone(), edge.target.clone());
        if !existing_edge_keys.contains(&key) {
            unique_new_edges.push(edge);
        }
    }

    // Add unique new nodes and edges to existing graph
    existing.nodes.extend(unique_new_nodes);
    existing.links.extend(unique_new_edges);
    existing.hyperedges.extend(new_hyperedges);

    // Update metadata
    existing.metadata.total_nodes = existing.nodes.len();
    existing.metadata.total_edges = existing.links.len();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Confidence, Edge, Node};

    #[test]
    fn test_build_graph() {
        let mut extraction1 = ExtractionResult::new();
        extraction1.add_node(Node::new(
            "a.py:foo".into(),
            "foo".into(),
            "a.py".into(),
            "L1".into(),
        ));
        extraction1.add_node(Node::new(
            "a.py:bar".into(),
            "bar".into(),
            "a.py".into(),
            "L1".into(),
        ));
        extraction1.add_edge(Edge::new(
            "a.py:foo".into(),
            "a.py:bar".into(),
            "calls".into(),
            Confidence::Extracted,
        ));

        let mut extraction2 = ExtractionResult::new();
        extraction2.add_node(Node::new(
            "b.py:baz".into(),
            "baz".into(),
            "b.py".into(),
            "L1".into(),
        ));
        extraction2.add_edge(Edge::new(
            "a.py:foo".into(),
            "b.py:baz".into(),
            "imports".into(),
            Confidence::Inferred,
        ));

        let graph = build_graph(vec![extraction1, extraction2]);

        assert_eq!(graph.nodes.len(), 3);
        assert_eq!(graph.links.len(), 2);
        assert!(graph.metadata.communities > 0);
    }

    #[test]
    fn test_merge_extractions() {
        let mut extraction1 = ExtractionResult::new();
        extraction1.add_node(Node::new(
            "a.py:foo".into(),
            "foo".into(),
            "a.py".into(),
            "L1".into(),
        ));

        let mut extraction2 = ExtractionResult::new();
        extraction2.add_node(Node::new(
            "a.py:foo".into(),
            "foo".into(),
            "a.py".into(),
            "L2".into(),
        )); // Duplicate
        extraction2.add_node(Node::new(
            "b.py:bar".into(),
            "bar".into(),
            "b.py".into(),
            "L1".into(),
        ));

        let merged = merge_extractions(vec![extraction1, extraction2]);

        // Should have only 2 nodes (foo deduplicated)
        assert_eq!(merged.nodes.len(), 2);
    }
}
