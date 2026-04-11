//! Community detection module

use crate::types::{GraphData, CommunityResult};
use std::collections::{HashMap, HashSet};

/// Detect communities using label propagation
pub fn cluster(graph: &GraphData) -> CommunityResult {
    let n = graph.nodes.len();
    
    if n == 0 {
        return CommunityResult {
            assignments: vec![],
            cohesion_scores: HashMap::new(),
            community_sizes: HashMap::new(),
        };
    }
    
    // Build adjacency list
    let mut adj: HashMap<usize, HashSet<usize>> = HashMap::new();
    let mut node_index: HashMap<String, usize> = HashMap::new();
    
    for (i, node) in graph.nodes.iter().enumerate() {
        node_index.insert(node.id.clone(), i);
        adj.insert(i, HashSet::new());
    }
    
    for edge in &graph.edges {
        if let (Some(&src), Some(&tgt)) = (
            node_index.get(&edge.source),
            node_index.get(&edge.target),
        ) {
            adj.entry(src).or_default().insert(tgt);
            adj.entry(tgt).or_default().insert(src);
        }
    }
    
    // Simple label propagation (fast approximation)
    let assignments = label_propagation(&adj, n);
    
    // Calculate statistics
    let mut community_sizes: HashMap<u32, usize> = HashMap::new();
    for &c in &assignments {
        *community_sizes.entry(c).or_insert(0) += 1;
    }
    
    // Calculate cohesion
    let cohesion_scores = calculate_cohesion(&adj, &assignments);
    
    CommunityResult {
        assignments,
        cohesion_scores,
        community_sizes,
    }
}

/// Label propagation algorithm
fn label_propagation(adj: &HashMap<usize, HashSet<usize>>, n: usize) -> Vec<u32> {
    let mut labels: Vec<u32> = (0..n as u32).collect();
    let mut changed = true;
    let mut iterations = 0;
    let max_iterations = 100;
    
    while changed && iterations < max_iterations {
        changed = false;
        iterations += 1;
        
        for node in 0..n {
            let neighbors = adj.get(&node).cloned().unwrap_or_default();
            if neighbors.is_empty() {
                continue;
            }
            
            // Count label frequencies among neighbors
            let mut label_counts: HashMap<u32, usize> = HashMap::new();
            for &neighbor in &neighbors {
                let label = labels[neighbor];
                *label_counts.entry(label).or_insert(0) += 1;
            }
            
            // Find most common label
            let most_common = label_counts
                .into_iter()
                .max_by_key(|(_, count)| *count)
                .map(|(label, _)| label);
            
            if let Some(new_label) = most_common {
                if new_label != labels[node] {
                    labels[node] = new_label;
                    changed = true;
                }
            }
        }
    }
    
    // Renumber labels to 0..k
    let mut unique_labels: Vec<u32> = labels.iter().cloned().collect::<HashSet<_>>().into_iter().collect();
    unique_labels.sort();
    
    labels
        .iter()
        .map(|l| {
            unique_labels
                .iter()
                .position(|&u| u == *l)
                .unwrap() as u32
        })
        .collect()
}

/// Calculate cohesion score for each community
fn calculate_cohesion(
    adj: &HashMap<usize, HashSet<usize>>,
    assignments: &[u32],
) -> HashMap<u32, f64> {
    let mut cohesion: HashMap<u32, Vec<f64>> = HashMap::new();
    
    for (node, &community) in assignments.iter().enumerate() {
        let neighbors = adj.get(&node).cloned().unwrap_or_default();
        if neighbors.is_empty() {
            continue;
        }
        
        let community_neighbors: usize = neighbors
            .iter()
            .filter(|&&n| assignments[n] == community)
            .count();
        
        let degree = neighbors.len();
        let score = if degree > 0 {
            community_neighbors as f64 / degree as f64
        } else {
            0.0
        };
        
        cohesion.entry(community).or_default().push(score);
    }
    
    // Average cohesion per community
    cohesion
        .into_iter()
        .map(|(c, scores)| {
            let avg = scores.iter().sum::<f64>() / scores.len() as f64;
            (c, avg)
        })
        .collect()
}

/// Split oversized communities (>25 nodes)
pub fn split_oversized(graph: &mut GraphData, max_size: usize) {
    let mut community_sizes: HashMap<u32, Vec<usize>> = HashMap::new();
    
    for (i, node) in graph.nodes.iter().enumerate() {
        if let Some(c) = node.community {
            community_sizes.entry(c).or_default().push(i);
        }
    }
    
    // Split large communities
    let mut new_community_id = community_sizes.keys().max().unwrap_or(&0) + 1;
    
    for (_community, indices) in &mut community_sizes {
        if indices.len() > max_size {
            // Split into smaller groups
            let chunks: Vec<_> = indices.chunks(max_size).collect();
            for (i, chunk) in chunks.iter().enumerate() {
                if i == 0 {
                    // First chunk keeps original community ID
                    continue;
                }
                for &idx in *chunk {
                    if let Some(node) = graph.nodes.get_mut(idx) {
                        node.community = Some(new_community_id);
                    }
                }
                new_community_id += 1;
            }
        }
    }
    
    // Update metadata
    let unique_communities = graph.nodes.iter()
        .filter_map(|n| n.community)
        .collect::<HashSet<_>>()
        .len();
    graph.metadata.communities = unique_communities;
}

/// Add community info to nodes
pub fn add_communities(graph: &mut GraphData, communities: &[u32]) {
    for (i, community) in communities.iter().enumerate() {
        if i < graph.nodes.len() {
            graph.nodes[i].community = Some(*community);
        }
    }
    graph.metadata.communities = communities.iter().cloned().collect::<HashSet<_>>().len();
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
            Node::new("b.py:E".into(), "E".into(), "b.py".into(), "L1".into()),
        ];
        
        let edges = vec![
            Edge::new("a.py:A".into(), "a.py:B".into(), "calls".into(), Confidence::Extracted),
            Edge::new("a.py:B".into(), "a.py:C".into(), "calls".into(), Confidence::Extracted),
            Edge::new("b.py:D".into(), "b.py:E".into(), "calls".into(), Confidence::Extracted),
        ];
        
        GraphData {
            nodes,
            edges,
            metadata: GraphMetadata::new(5, 3, 0),
        }
    }

    #[test]
    fn test_cluster() {
        let graph = create_test_graph();
        let result = cluster(&graph);
        
        assert_eq!(result.assignments.len(), 5);
        assert!(!result.community_sizes.is_empty());
    }

    #[test]
    fn test_split_oversized() {
        let mut graph = create_test_graph();
        add_communities(&mut graph, &[0, 0, 0, 1, 1]); // First 3 in community 0, last 2 in community 1
        
        split_oversized(&mut graph, 2); // Split communities larger than 2
        
        // Should have more communities now
        let unique: HashSet<_> = graph.nodes.iter().filter_map(|n| n.community).collect();
        assert!(unique.len() > 2);
    }
}
