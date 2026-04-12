//! Hyperedge Detection Module
//!
//! Detects hyperedges - groups of 3+ nodes that work together (no LLM required).
//!
//! # Algorithms
//!
//! 1. **File-Based** (O(n)): Group nodes by source file
//! 2. **Call Chain** (O(n²)): Find A→B→C→D chains
//! 3. **Config Pattern** (O(n)): K8s, Docker, Terraform patterns

use crate::types::{Confidence, GraphData, Hyperedge, Node};
use std::collections::{HashMap, HashSet};
use std::path::Path;

/// Minimum nodes to form a hyperedge
const MIN_NODES: usize = 3;
/// Maximum nodes in a hyperedge
const MAX_NODES: usize = 20;
/// Minimum score threshold
const MIN_SCORE: f64 = 0.3;

/// Hyperedge candidate before processing
#[derive(Debug, Clone)]
struct HyperedgeCandidate {
    id: String,
    label: String,
    nodes: Vec<String>,
    relation: String,
    confidence: Confidence,
    source_file: String,
    score: f64,
}

impl HyperedgeCandidate {
    fn into_hyperedge(self) -> Hyperedge {
        Hyperedge {
            id: self.id,
            label: self.label,
            nodes: self.nodes,
            relation: self.relation,
            confidence: self.confidence,
            confidence_score: self.score,
            source_file: self.source_file,
        }
    }
}

/// Detect all hyperedges in a graph
pub fn detect_hyperedges(graph: &GraphData) -> Vec<Hyperedge> {
    let mut candidates: Vec<HyperedgeCandidate> = Vec::new();

    // Algorithm 1: File-Based (fastest, O(n))
    let file_candidates = detect_file_groups(graph);
    candidates.extend(file_candidates);

    // Algorithm 2: Call Chain (medium, O(n²))
    let chain_candidates = detect_call_chains(graph);
    candidates.extend(chain_candidates);

    // Algorithm 3: Config Pattern (for K8s, Docker, Terraform)
    let config_candidates = detect_config_patterns(graph);
    candidates.extend(config_candidates);

    // Process candidates: dedup + filter + sort
    process_candidates(candidates)
}

/// Algorithm 1: File-Based Groups
/// Groups nodes by source file - O(n)
fn detect_file_groups(graph: &GraphData) -> Vec<HyperedgeCandidate> {
    let mut by_file: HashMap<String, Vec<&Node>> = HashMap::new();

    for node in &graph.nodes {
        by_file
            .entry(node.source_file.clone())
            .or_default()
            .push(node);
    }

    by_file
        .into_iter()
        .filter(|(_, nodes)| nodes.len() >= MIN_NODES)
        .filter(|(_, nodes)| nodes.len() <= MAX_NODES * 2)
        .map(|(file, nodes)| {
            let file_stem = Path::new(&file)
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "unknown".to_string());

            let node_ids: Vec<String> = nodes.iter().map(|n| n.id.clone()).collect();
            let score = calculate_cohesion(&node_ids, graph);

            HyperedgeCandidate {
                id: format!("file_{}", file_stem.to_lowercase().replace(' ', "_")),
                label: format!("{} module", file_stem),
                nodes: node_ids,
                relation: "participate_in".to_string(),
                confidence: Confidence::Inferred,
                source_file: file,
                score,
            }
        })
        .collect()
}

/// Algorithm 2: Call Chain Detection
/// Finds A→B→C→D chains - O(n²) worst case
fn detect_call_chains(graph: &GraphData) -> Vec<HyperedgeCandidate> {
    // Build adjacency list from "calls" edges only
    let adj: HashMap<String, Vec<String>> = graph
        .links
        .iter()
        .filter(|e| e.relation == "calls")
        .fold(HashMap::new(), |mut acc, edge| {
            acc.entry(edge.source.clone())
                .or_default()
                .push(edge.target.clone());
            acc
        });

    let mut candidates = Vec::new();
    let node_ids: Vec<String> = graph.nodes.iter().map(|n| n.id.clone()).collect();
    let node_set: HashSet<String> = node_ids.iter().cloned().collect();

    // DFS from each node to find chains
    for start in &node_ids {
        let mut visited = HashSet::new();
        let mut chain = vec![start.clone()];
        find_chains_dfs(
            start,
            &adj,
            &node_set,
            &mut visited,
            &mut chain,
            graph,
            &mut candidates,
        );
    }

    candidates
}

/// DFS helper for call chain detection
#[allow(clippy::too_many_arguments)]
fn find_chains_dfs(
    current: &str,
    adj: &HashMap<String, Vec<String>>,
    valid_nodes: &HashSet<String>,
    visited: &mut HashSet<String>,
    chain: &mut Vec<String>,
    graph: &GraphData,
    candidates: &mut Vec<HyperedgeCandidate>,
) {
    visited.insert(current.to_string());

    if let Some(neighbors) = adj.get(current) {
        for neighbor in neighbors {
            if valid_nodes.contains(neighbor) && !visited.contains(neighbor) {
                chain.push(neighbor.clone());

                // Found a valid chain (3-20 nodes)
                if chain.len() >= MIN_NODES && chain.len() <= MAX_NODES {
                    let score = calculate_chain_cohesion(chain, graph);

                    candidates.push(HyperedgeCandidate {
                        id: format!("chain_{}", candidates.len()),
                        label: format!("Call Chain ({})", chain.len()),
                        nodes: chain.clone(),
                        relation: "call_chain".to_string(),
                        confidence: Confidence::Extracted,
                        source_file: get_first_file(chain, graph),
                        score,
                    });
                }

                // Continue exploring
                if chain.len() < MAX_NODES {
                    find_chains_dfs(
                        neighbor,
                        adj,
                        valid_nodes,
                        visited,
                        chain,
                        graph,
                        candidates,
                    );
                }

                chain.pop();
            }
        }
    }

    visited.remove(current);
}

/// Algorithm 3: Config Pattern Detection
/// Detects K8s, Docker, Terraform patterns - O(n)
fn detect_config_patterns(graph: &GraphData) -> Vec<HyperedgeCandidate> {
    let mut candidates = Vec::new();

    // Group nodes by file extension
    let mut by_ext: HashMap<String, Vec<&Node>> = HashMap::new();
    for node in &graph.nodes {
        let ext = Path::new(&node.source_file)
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default();

        if !ext.is_empty() {
            by_ext.entry(ext).or_default().push(node);
        }
    }

    // Config file patterns
    let config_extensions = ["yaml", "yml", "json", "toml", "tf", "dockerfile"];

    for ext in config_extensions {
        if let Some(nodes) = by_ext.get(ext) {
            if nodes.len() >= MIN_NODES && nodes.len() <= MAX_NODES {
                let node_ids: Vec<String> = nodes.iter().map(|n| n.id.clone()).collect();
                let score = calculate_cohesion(&node_ids, graph);

                candidates.push(HyperedgeCandidate {
                    id: format!("config_{}", ext),
                    label: format!("Config files (.{})", ext),
                    nodes: node_ids,
                    relation: "config_group".to_string(),
                    confidence: Confidence::Inferred,
                    source_file: nodes.first().map(|n| n.source_file.clone()).unwrap_or_default(),
                    score,
                });
            }
        }
    }

    candidates
}

/// Calculate cohesion score for a group of nodes
/// Cohesion = internal_edges / (internal_edges + external_edges)
fn calculate_cohesion(node_ids: &[String], graph: &GraphData) -> f64 {
    let node_set: HashSet<&str> = node_ids.iter().map(|s| s.as_str()).collect();

    let mut internal_edges = 0;
    let mut external_edges = 0;

    for edge in &graph.links {
        let src_in = node_set.contains(edge.source.as_str());
        let tgt_in = node_set.contains(edge.target.as_str());

        if src_in && tgt_in {
            internal_edges += 1;
        } else if src_in || tgt_in {
            external_edges += 1;
        }
    }

    let total = internal_edges + external_edges;
    if total == 0 {
        return 0.5;
    }

    internal_edges as f64 / total as f64
}

/// Calculate cohesion for a call chain
fn calculate_chain_cohesion(chain: &[String], graph: &GraphData) -> f64 {
    let mut present_edges = 0;
    let expected_edges = chain.len().saturating_sub(1);

    for i in 0..chain.len().saturating_sub(1) {
        let src = &chain[i];
        let tgt = &chain[i + 1];

        // Check if edge exists (either direction)
        let has_edge = graph.links.iter().any(|e| {
            (e.source == *src && e.target == *tgt)
                || (e.source == *tgt && e.target == *src)
        });

        if has_edge {
            present_edges += 1;
        }
    }

    if expected_edges == 0 {
        return 0.5;
    }

    // Base score from edge presence
    let edge_score = present_edges as f64 / expected_edges as f64;

    // Bonus for chains in same file
    let same_file_bonus = if chain.len() > 1 {
        let first_node = graph.nodes.iter().find(|n| n.id == chain[0]);
        let last_node = graph.nodes.iter().find(|n| n.id == chain[chain.len() - 1]);
        if first_node.zip(last_node).map(|(f, l)| f.source_file == l.source_file).unwrap_or(false) {
            0.1
        } else {
            0.0
        }
    } else {
        0.0
    };

    (edge_score + same_file_bonus).min(1.0)
}

/// Get the first file from a list of node IDs
fn get_first_file(node_ids: &[String], graph: &GraphData) -> String {
    for id in node_ids {
        if let Some(node) = graph.nodes.iter().find(|n| n.id == *id) {
            return node.source_file.clone();
        }
    }
    String::new()
}

/// Process candidates: dedup, filter, sort
fn process_candidates(mut candidates: Vec<HyperedgeCandidate>) -> Vec<Hyperedge> {
    // Deduplicate by node set
    let mut seen = HashSet::new();
    candidates.retain(|c| {
        let mut sorted_nodes = c.nodes.clone();
        sorted_nodes.sort();
        let key = sorted_nodes.join("|");
        seen.insert(key)
    });

    // Filter by criteria
    candidates.retain(|c| {
        c.score >= MIN_SCORE
            && c.nodes.len() >= MIN_NODES
            && c.nodes.len() <= MAX_NODES
    });

    // Sort by score (highest first)
    candidates.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    candidates.into_iter().map(|c| c.into_hyperedge()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Edge;

    fn create_test_graph() -> GraphData {
        let nodes = vec![
            Node::new(
                "order.rs:create_order".to_string(),
                "create_order".to_string(),
                "order.rs".to_string(),
                "L10".to_string(),
            ),
            Node::new(
                "order.rs:validate_items".to_string(),
                "validate_items".to_string(),
                "order.rs".to_string(),
                "L25".to_string(),
            ),
            Node::new(
                "order.rs:save_order".to_string(),
                "save_order".to_string(),
                "order.rs".to_string(),
                "L40".to_string(),
            ),
            Node::new(
                "order.rs:send_confirmation".to_string(),
                "send_confirmation".to_string(),
                "order.rs".to_string(),
                "L55".to_string(),
            ),
            Node::new(
                "inventory.rs:check_stock".to_string(),
                "check_stock".to_string(),
                "inventory.rs".to_string(),
                "L5".to_string(),
            ),
        ];

        let links = vec![
            Edge::new(
                "order.rs:create_order".to_string(),
                "order.rs:validate_items".to_string(),
                "calls".to_string(),
                Confidence::Extracted,
            ),
            Edge::new(
                "order.rs:create_order".to_string(),
                "order.rs:save_order".to_string(),
                "calls".to_string(),
                Confidence::Extracted,
            ),
            Edge::new(
                "order.rs:create_order".to_string(),
                "inventory.rs:check_stock".to_string(),
                "calls".to_string(),
                Confidence::Extracted,
            ),
            Edge::new(
                "order.rs:save_order".to_string(),
                "order.rs:send_confirmation".to_string(),
                "calls".to_string(),
                Confidence::Extracted,
            ),
        ];

        GraphData::new(nodes, links, 2)
    }

    #[test]
    fn test_detect_file_groups() {
        let graph = create_test_graph();
        let candidates = detect_file_groups(&graph);

        // order.rs has 4 nodes, should form a hyperedge
        let order_hyperedge = candidates.iter().find(|c| c.source_file == "order.rs");
        assert!(order_hyperedge.is_some());
        let order = order_hyperedge.unwrap();
        assert_eq!(order.nodes.len(), 4);
        assert_eq!(order.relation, "participate_in");
        assert_eq!(order.confidence, Confidence::Inferred);
    }

    #[test]
    fn test_detect_call_chains() {
        let graph = create_test_graph();
        let candidates = detect_call_chains(&graph);

        // Should find chains
        assert!(!candidates.is_empty());
    }

    #[test]
    fn test_cohesion_calculation() {
        let graph = create_test_graph();
        let node_ids = vec![
            "order.rs:create_order".to_string(),
            "order.rs:validate_items".to_string(),
            "order.rs:save_order".to_string(),
        ];

        let score = calculate_cohesion(&node_ids, &graph);
        assert!(score > 0.0);
        assert!(score <= 1.0);
    }

    #[test]
    fn test_process_candidates_filters() {
        let mut candidates = vec![
            HyperedgeCandidate {
                id: "too_small".to_string(),
                label: "Too Small".to_string(),
                nodes: vec!["a".to_string(), "b".to_string()],
                relation: "test".to_string(),
                confidence: Confidence::Inferred,
                source_file: "test.rs".to_string(),
                score: 0.5,
            },
            HyperedgeCandidate {
                id: "valid".to_string(),
                label: "Valid".to_string(),
                nodes: vec!["a".to_string(), "b".to_string(), "c".to_string()],
                relation: "test".to_string(),
                confidence: Confidence::Inferred,
                source_file: "test.rs".to_string(),
                score: 0.5,
            },
            HyperedgeCandidate {
                id: "low_score".to_string(),
                label: "Low Score".to_string(),
                nodes: vec!["x".to_string(), "y".to_string(), "z".to_string()],
                relation: "test".to_string(),
                confidence: Confidence::Inferred,
                source_file: "test.rs".to_string(),
                score: 0.1,
            },
        ];

        let hyperedges = process_candidates(candidates);

        // Only "valid" should remain
        assert_eq!(hyperedges.len(), 1);
        assert_eq!(hyperedges[0].id, "valid");
    }

    #[test]
    fn test_dedup_by_node_set() {
        let candidates = vec![
            HyperedgeCandidate {
                id: "first".to_string(),
                label: "First".to_string(),
                nodes: vec!["a".to_string(), "b".to_string(), "c".to_string()],
                relation: "test".to_string(),
                confidence: Confidence::Inferred,
                source_file: "test.rs".to_string(),
                score: 0.8,
            },
            HyperedgeCandidate {
                id: "second".to_string(),
                label: "Second".to_string(),
                nodes: vec!["c".to_string(), "b".to_string(), "a".to_string()], // Same nodes
                relation: "test".to_string(),
                confidence: Confidence::Inferred,
                source_file: "test.rs".to_string(),
                score: 0.9,
            },
        ];

        let hyperedges = process_candidates(candidates);

        // Should dedupe to 1
        assert_eq!(hyperedges.len(), 1);
    }

    #[test]
    fn test_detect_hyperedges_integration() {
        let graph = create_test_graph();
        let hyperedges = detect_hyperedges(&graph);

        // Should find at least one hyperedge
        assert!(!hyperedges.is_empty());

        // All hyperedges should have valid properties
        for he in &hyperedges {
            assert!(he.nodes.len() >= MIN_NODES);
            assert!(he.nodes.len() <= MAX_NODES);
            assert!(he.confidence_score >= MIN_SCORE);
        }
    }

    #[test]
    fn test_config_pattern_detection() {
        let nodes = vec![
            Node::new("a.yaml:svc1".to_string(), "svc1".to_string(), "a.yaml".to_string(), "L1".to_string()),
            Node::new("a.yaml:svc2".to_string(), "svc2".to_string(), "a.yaml".to_string(), "L10".to_string()),
            Node::new("a.yaml:svc3".to_string(), "svc3".to_string(), "a.yaml".to_string(), "L20".to_string()),
        ];

        let graph = GraphData::new(nodes, vec![], 1);
        let candidates = detect_config_patterns(&graph);

        assert!(!candidates.is_empty());
        let yaml_group = candidates.iter().find(|c| c.source_file.ends_with(".yaml"));
        assert!(yaml_group.is_some());
    }
}
