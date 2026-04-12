//! Community detection module using Leiden algorithm

use crate::leiden::leiden_communities;
use crate::types::{CommunityResult, Confidence, GraphData};
use std::collections::{HashMap, HashSet};

/// Detect communities using Leiden algorithm
pub fn cluster(graph: &GraphData) -> CommunityResult {
    let n = graph.nodes.len();

    if n == 0 {
        return CommunityResult {
            assignments: vec![],
            cohesion_scores: HashMap::new(),
            community_sizes: HashMap::new(),
        };
    }

    // Build node index
    let mut node_index: HashMap<String, usize> = HashMap::new();
    for (i, node) in graph.nodes.iter().enumerate() {
        node_index.insert(node.id.clone(), i);
    }

    // Build edge list with weights
    // EXTRACTED edges = weight 1.0, INFERRED edges = weight 0.5
    let mut edges: Vec<(usize, usize, f64)> = Vec::new();
    for edge in &graph.links {
        if let (Some(&src), Some(&tgt)) =
            (node_index.get(&edge.source), node_index.get(&edge.target))
        {
            let weight = match edge.confidence {
                Confidence::Extracted => 1.0,
                _ => 0.5, // Inferred or Ambiguous get lower weight
            };
            edges.push((src, tgt, weight));
        }
    }

    // Run Leiden clustering
    let assignments = leiden_communities(n, &edges);

    // Calculate community sizes
    let mut community_sizes: HashMap<u32, usize> = HashMap::new();
    for &c in &assignments {
        *community_sizes.entry(c).or_insert(0) += 1;
    }

    // Build adjacency list for cohesion calculation
    let mut adj: HashMap<usize, HashSet<usize>> = HashMap::new();
    for i in 0..n {
        adj.insert(i, HashSet::new());
    }
    for (src, tgt, _) in &edges {
        adj.entry(*src).or_default().insert(*tgt);
        adj.entry(*tgt).or_default().insert(*src);
    }

    // Calculate cohesion scores (same formula as Graphify)
    let cohesion_scores = calculate_cohesion(&adj, &assignments);

    CommunityResult {
        assignments,
        cohesion_scores,
        community_sizes,
    }
}

/// Calculate cohesion score for each community
/// Formula: actual_intra_community_edges / possible_intra_community_edges
/// This matches Graphify's cohesion_score() in cluster.py
fn calculate_cohesion(
    adj: &HashMap<usize, HashSet<usize>>,
    assignments: &[u32],
) -> HashMap<u32, f64> {
    let n = assignments.len();
    if n == 0 {
        return HashMap::new();
    }

    // Group nodes by community
    let mut communities: HashMap<u32, Vec<usize>> = HashMap::new();
    for (node, &comm) in assignments.iter().enumerate() {
        communities.entry(comm).or_default().push(node);
    }

    let mut cohesion: HashMap<u32, f64> = HashMap::new();

    for (comm, nodes) in &communities {
        let comm_size = nodes.len();
        if comm_size <= 1 {
            cohesion.insert(*comm, 1.0);
            continue;
        }

        // Count actual intra-community edges
        let mut actual_edges = 0usize;
        for &node in nodes {
            if let Some(neighbors) = adj.get(&node) {
                for &neighbor in neighbors {
                    // Count each edge once (only when neighbor > node to avoid double counting)
                    if neighbor > node && assignments[neighbor] == *comm {
                        actual_edges += 1;
                    }
                }
            }
        }

        // Calculate possible edges: n * (n-1) / 2
        let possible_edges = (comm_size * (comm_size - 1)) as f64 / 2.0;

        let score = if possible_edges > 0.0 {
            actual_edges as f64 / possible_edges
        } else {
            0.0
        };

        // Round to 2 decimal places (same as Graphify)
        cohesion.insert(*comm, (score * 100.0).round() / 100.0);
    }

    cohesion
}

/// Split oversized communities recursively using Louvain
/// 
/// For communities larger than max_size:
/// 1. Extract the subgraph for that community
/// 2. Re-run Louvain on the subgraph
/// 3. Assign new sub-community IDs to nodes
/// 
/// This is equivalent to Graphify's _split_community() using Leiden/Louvain.
pub fn split_oversized(graph: &mut GraphData, max_size: usize) {
    // Group nodes by community
    let mut community_groups: HashMap<u32, Vec<usize>> = HashMap::new();
    for (i, node) in graph.nodes.iter().enumerate() {
        if let Some(c) = node.community {
            community_groups.entry(c).or_default().push(i);
        }
    }

    // Track new community IDs
    let mut new_community_id = community_groups.keys().max().copied().unwrap_or(0) + 1;

    for (_community, node_indices) in &community_groups {
        if node_indices.len() <= max_size {
            continue;
        }

        // Extract subgraph node IDs (clone strings to avoid lifetime issues)
        let subgraph_node_ids: Vec<String> = node_indices
            .iter()
            .filter_map(|&i| graph.nodes.get(i).map(|n| n.id.clone()))
            .collect();
        
        // Build node index mapping (old ID -> new index in subgraph)
        let node_to_subgraph: HashMap<&str, usize> = subgraph_node_ids
            .iter()
            .enumerate()
            .map(|(i, id)| (id.as_str(), i))
            .collect();

        // Extract edges within this community
        let subgraph_node_ids_str: HashSet<&str> = subgraph_node_ids.iter().map(|s| s.as_str()).collect();
        let subgraph_edges: Vec<(usize, usize, f64)> = graph
            .links
            .iter()
            .filter(|e| {
                subgraph_node_ids_str.contains(e.source.as_str())
                    && subgraph_node_ids_str.contains(e.target.as_str())
            })
            .filter_map(|e| {
                let src_idx = node_to_subgraph.get(e.source.as_str())?;
                let tgt_idx = node_to_subgraph.get(e.target.as_str())?;
                let weight = match e.confidence {
                    Confidence::Extracted => 1.0,
                    _ => 0.5,
                };
                Some((*src_idx, *tgt_idx, weight))
            })
            .collect();

        // Run Leiden on subgraph
        let n = subgraph_node_ids.len();
        if n == 0 || subgraph_edges.is_empty() {
            continue;
        }

        let sub_assignments = leiden_communities(n, &subgraph_edges);

        // Count sub-communities
        let sub_communities: HashSet<u32> = sub_assignments.iter().copied().collect();

        // Assign new community IDs to each sub-community
        let mut sub_id_map: HashMap<u32, u32> = HashMap::new();
        for &sc in &sub_communities {
            if !sub_id_map.contains_key(&sc) {
                sub_id_map.insert(sc, new_community_id);
                new_community_id += 1;
            }
        }

        // Update node community assignments
        // Use indices collected earlier to avoid borrow conflicts
        for &node_idx in node_indices {
            // Get node ID for this index
            let node_id = match graph.nodes.get(node_idx) {
                Some(n) => n.id.clone(),
                None => continue,
            };
            
            // Find subgraph index
            let sub_idx = match node_to_subgraph.get(node_id.as_str()) {
                Some(&idx) => idx,
                None => continue,
            };
            
            // Get old and new community
            let sub_comm = sub_assignments[sub_idx];
            let new_id = match sub_id_map.get(&sub_comm) {
                Some(&id) => id,
                None => continue,
            };
            
            // Update the node (we do this separately to avoid nested borrows)
            if sub_comm > 0 || sub_communities.len() == 1 {
                if let Some(node) = graph.nodes.get_mut(node_idx) {
                    node.community = Some(new_id);
                }
            }
        }
    }

    // Update metadata
    let unique_communities = graph
        .nodes
        .iter()
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
