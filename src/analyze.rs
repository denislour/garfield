//! Graph analysis module

use crate::types::{GraphData, Confidence};
use std::collections::HashMap;

/// Analysis result
#[derive(Debug)]
pub struct Analysis {
    pub god_nodes: Vec<GodNode>,
    pub surprising_connections: Vec<SurprisingConnection>,
    pub community_sizes: HashMap<u32, usize>,
    pub confidence_stats: ConfidenceStats,
}

/// God node - most connected node
#[derive(Debug, Clone)]
pub struct GodNode {
    pub node: crate::types::Node,
    pub degree: usize,
    pub edges: Vec<String>,
}

/// Surprising connection - cross-community edge
#[derive(Debug, Clone)]
pub struct SurprisingConnection {
    pub source: String,
    pub target: String,
    pub relation: String,
    pub confidence: Confidence,
    pub source_community: u32,
    pub target_community: u32,
}

/// Confidence statistics
#[derive(Debug, Clone)]
pub struct ConfidenceStats {
    pub extracted: usize,
    pub inferred: usize,
    pub ambiguous: usize,
}

/// Main analysis function
pub fn analyze(graph: &GraphData) -> Analysis {
    let god_nodes = find_god_nodes(graph, 10);
    let surprising = find_surprising_connections(graph);
    let community_sizes = count_community_sizes(graph);
    let confidence_stats = count_confidence(graph);
    
    Analysis {
        god_nodes,
        surprising_connections: surprising,
        community_sizes,
        confidence_stats,
    }
}

/// Find top N most connected nodes (god nodes)
pub fn find_god_nodes(graph: &GraphData, top_n: usize) -> Vec<GodNode> {
    // Build adjacency for degree calculation
    let mut degree: HashMap<&str, usize> = HashMap::new();
    let mut neighbors: HashMap<&str, Vec<&str>> = HashMap::new();
    
    for edge in &graph.links {
        *degree.entry(&edge.source).or_insert(0) += 1;
        *degree.entry(&edge.target).or_insert(0) += 1;
        neighbors
            .entry(&edge.source)
            .or_default()
            .push(&edge.target);
        neighbors
            .entry(&edge.target)
            .or_default()
            .push(&edge.source);
    }
    
    // Sort by degree
    let mut sorted: Vec<_> = degree.into_iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));
    
    // Take top N and build GodNode structs
    sorted
        .into_iter()
        .take(top_n)
        .filter_map(|(node_id, deg)| {
            graph.nodes.iter().find(|n| n.id == node_id).map(|node| {
                let neighbor_labels = neighbors
                    .get(node_id)
                    .map(|ns| {
                        ns.iter()
                            .filter_map(|nid| {
                                graph.nodes.iter().find(|n| &n.id == *nid).map(|n| n.label.clone())
                            })
                            .take(5)
                            .collect()
                    })
                    .unwrap_or_default();
                
                GodNode {
                    node: node.clone(),
                    degree: deg,
                    edges: neighbor_labels,
                }
            })
        })
        .collect()
}

/// Find cross-community edges (surprising connections)
pub fn find_surprising_connections(graph: &GraphData) -> Vec<SurprisingConnection> {
    graph
        .links
        .iter()
        .filter_map(|edge| {
            let src_comm = graph
                .nodes
                .iter()
                .find(|n| n.id == edge.source)
                .and_then(|n| n.community)
                .unwrap_or(0);
            
            let tgt_comm = graph
                .nodes
                .iter()
                .find(|n| n.id == edge.target)
                .and_then(|n| n.community)
                .unwrap_or(0);
            
            // Only include if different communities
            if src_comm != tgt_comm {
                Some(SurprisingConnection {
                    source: edge.source.clone(),
                    target: edge.target.clone(),
                    relation: edge.relation.clone(),
                    confidence: edge.confidence,
                    source_community: src_comm,
                    target_community: tgt_comm,
                })
            } else {
                None
            }
        })
        .collect()
}

/// Count community sizes
fn count_community_sizes(graph: &GraphData) -> HashMap<u32, usize> {
    let mut sizes: HashMap<u32, usize> = HashMap::new();
    for node in &graph.nodes {
        if let Some(c) = node.community {
            *sizes.entry(c).or_insert(0) += 1;
        }
    }
    sizes
}

/// Count confidence statistics
fn count_confidence(graph: &GraphData) -> ConfidenceStats {
    let mut stats = ConfidenceStats {
        extracted: 0,
        inferred: 0,
        ambiguous: 0,
    };
    
    for edge in &graph.links {
        match edge.confidence {
            Confidence::Extracted => stats.extracted += 1,
            Confidence::Inferred => stats.inferred += 1,
            Confidence::Ambiguous => stats.ambiguous += 1,
        }
    }
    
    stats
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Node, Edge, Confidence, GraphMetadata};

    fn create_test_graph() -> GraphData {
        let nodes = vec![
            Node::new("a.py:A".into(), "A".into(), "a.py".into(), "L1".into()),
            Node::new("a.py:B".into(), "B".into(), "a.py".into(), "L1".into()),
            Node::new("a.py:C".into(), "C".into(), "a.py".into(), "L1".into()),
            Node::new("b.py:D".into(), "D".into(), "b.py".into(), "L1".into()),
        ];
        
        let edges = vec![
            Edge::new("a.py:A".into(), "a.py:B".into(), "calls".into(), Confidence::Extracted),
            Edge::new("a.py:A".into(), "a.py:C".into(), "calls".into(), Confidence::Inferred),
            Edge::new("a.py:B".into(), "b.py:D".into(), "imports".into(), Confidence::Extracted),
        ];
        
        GraphData {
            nodes,
            edges,
            metadata: GraphMetadata::new(4, 3, 2),
        }
    }

    #[test]
    fn test_find_god_nodes() {
        let graph = create_test_graph();
        let gods = find_god_nodes(&graph, 2);
        
        // A and B should be top (degree 2 each, C and D have degree 1)
        assert!(!gods.is_empty());
        assert!(gods[0].degree >= 2, "top node should have degree >= 2");
    }

    #[test]
    fn test_confidence_stats() {
        let graph = create_test_graph();
        let stats = count_confidence(&graph);
        
        assert_eq!(stats.extracted, 2);
        assert_eq!(stats.inferred, 1);
        assert_eq!(stats.ambiguous, 0);
    }
}
