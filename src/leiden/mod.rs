//! Leiden community detection algorithm
//! 
//! Leiden = Louvain + Refinement
//! Paper: https://www.nature.com/articles/s41598-019-41695-z

use std::collections::HashMap;

/// Leiden community detection
pub fn leiden_communities(n: usize, edges: &[(usize, usize, f64)]) -> Vec<u32> {
    if n == 0 {
        return vec![];
    }

    let m: f64 = edges.iter().map(|(_, _, w)| w).sum::<f64>();
    if m == 0.0 {
        return (0..n as u32).collect();
    }

    // Build adjacency list
    let mut neighbors: HashMap<usize, Vec<(usize, f64)>> = HashMap::new();
    let mut node_weights: Vec<f64> = vec![0.0; n];
    
    for &(src, tgt, weight) in edges {
        neighbors.entry(src).or_default().push((tgt, weight));
        neighbors.entry(tgt).or_default().push((src, weight));
        node_weights[src] += weight;
        node_weights[tgt] += weight;
    }

    // Initialize: each node in own community
    let mut community: Vec<u32> = (0..n as u32).collect();
    let mut community_weights: Vec<f64> = node_weights.clone();

    // Phase 1: Local move (Louvain-style)
    let max_iterations = 10;
    for _ in 0..max_iterations {
        let mut moved = false;
        
        for node in 0..n {
            let current_c = community[node];
            let k_i = node_weights[node];
            
            if k_i == 0.0 {
                continue;
            }

            // Calculate sum of weights to each neighbor community
            let mut comm_sums: HashMap<u32, f64> = HashMap::new();
            let mut current_comm_sum = 0.0;
            
            if let Some(nbrs) = neighbors.get(&node) {
                for &(nbr, weight) in nbrs {
                    let nbr_c = community[nbr];
                    if nbr_c == current_c {
                        current_comm_sum += weight;
                    }
                    *comm_sums.entry(nbr_c).or_insert(0.0) += weight;
                }
            }

            // Calculate modularity gain for each community
            // delta_Q = (weight_to_c - weight_from_c) / m
            //        - k_i * (community_weight_c - k_i) / m^2
            let mut best_c = current_c;
            let mut best_gain = 0.0;

            for (c, &weight_to_c) in &comm_sums {
                if *c == current_c {
                    continue;
                }
                
                let weight_from_c = current_comm_sum;
                let community_weight_c = community_weights[*c as usize];
                
                // Modularity gain
                let gain = (weight_to_c - weight_from_c) / m 
                    - k_i * (community_weight_c - k_i) / (m * m);

                if gain > best_gain {
                    best_gain = gain;
                    best_c = *c;
                }
            }

            // Move if positive gain
            if best_c != current_c && best_gain > 1e-10 {
                // Update community weights
                community_weights[current_c as usize] -= k_i;
                community_weights[best_c as usize] += k_i;
                community[node] = best_c;
                moved = true;
            }
        }
        
        if !moved {
            break;
        }
    }

    // Phase 2: Refinement (simplified - check if communities are well-connected)
    // Skip for now - will be added in future

    // Renumber communities
    let mut comm_counts: HashMap<u32, usize> = HashMap::new();
    for &c in &community {
        *comm_counts.entry(c).or_insert(0) += 1;
    }

    let mut comms: Vec<(u32, usize)> = comm_counts.into_iter().collect();
    comms.sort_by(|a, b| b.1.cmp(&a.1));

    let mut comm_map: HashMap<u32, u32> = HashMap::new();
    for (new_id, (old_id, _)) in comms.iter().enumerate() {
        comm_map.insert(*old_id, new_id as u32);
    }

    community.iter().map(|c| comm_map[c]).collect()
}
