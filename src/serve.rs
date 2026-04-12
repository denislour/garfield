//! Query engine module

use crate::types::{GraphData, Confidence};
use std::collections::{HashMap, HashSet};

/// Query result
#[derive(Debug)]
pub struct QueryResult {
    pub nodes: HashSet<String>,
    pub edges: Vec<crate::types::Edge>,
    pub text: String,
}

/// Score nodes by keyword match
/// Returns sorted list of (score, node_id)
pub fn score_nodes<'a>(graph: &'a GraphData, terms: &'a [String]) -> Vec<(f64, &'a str)> {
    let mut scored = Vec::new();
    
    for node in &graph.nodes {
        let label = node.label.to_lowercase();
        let source = node.source_file.to_lowercase();
        
        let label_score: f64 = terms
            .iter()
            .filter(|t| label.contains(&t.to_lowercase()))
            .count() as f64;
        
        let source_score: f64 = terms
            .iter()
            .filter(|t| source.contains(&t.to_lowercase()))
            .count() as f64
            * 0.5;
        
        let score = label_score + source_score;
        
        if score > 0.0 {
            scored.push((score, node.id.as_str()));
        }
    }
    
    // Sort by score descending
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    scored
}

/// Breadth-first search traversal
pub fn bfs(
    graph: &GraphData,
    start_nodes: &[&str],
    depth: usize,
) -> (HashSet<String>, Vec<crate::types::Edge>) {
    let mut visited: HashSet<String> = start_nodes.iter().map(|s| s.to_string()).collect();
    let mut frontier: HashSet<String> = visited.clone();
    let mut edges_seen: Vec<crate::types::Edge> = Vec::new();
    
    // Build adjacency map
    let adj = build_adjacency(graph);
    
    for _ in 0..depth {
        let mut next_frontier: HashSet<String> = HashSet::new();
        
        for node_id in &frontier {
            if let Some(neighbors) = adj.get(node_id) {
                for neighbor_id in neighbors {
                    if !visited.contains(neighbor_id) {
                        next_frontier.insert(neighbor_id.clone());
                        
                        // Find edge info
                        if let Some(edge) = find_edge(graph, node_id, neighbor_id) {
                            edges_seen.push(edge);
                        }
                    }
                }
            }
        }
        
        visited.extend(next_frontier.clone());
        frontier = next_frontier;
    }
    
    (visited, edges_seen)
}

/// Depth-first search traversal
pub fn dfs(
    graph: &GraphData,
    start_nodes: &[&str],
    depth: usize,
) -> (HashSet<String>, Vec<crate::types::Edge>) {
    let mut visited: HashSet<String> = HashSet::new();
    let mut edges_seen: Vec<crate::types::Edge> = Vec::new();
    
    // Build adjacency map
    let adj = build_adjacency(graph);
    
    // Stack: (node_id, current_depth)
    let mut stack: Vec<(&str, usize)> = start_nodes
        .iter()
        .rev()
        .map(|s| (*s, 0))
        .collect();
    
    while let Some((node_id, d)) = stack.pop() {
        if visited.contains(node_id) || d > depth {
            continue;
        }
        visited.insert(node_id.to_string());
        
        if let Some(neighbors) = adj.get(node_id) {
            for neighbor_id in neighbors {
                if !visited.contains(neighbor_id) {
                    stack.push((neighbor_id.as_str(), d + 1));
                    
                    if let Some(edge) = find_edge(graph, node_id, neighbor_id) {
                        edges_seen.push(edge);
                    }
                }
            }
        }
    }
    
    (visited, edges_seen)
}

/// Find shortest path between two nodes
pub fn find_shortest_path(
    graph: &GraphData,
    source: &str,
    target: &str,
    max_hops: usize,
) -> Option<Vec<String>> {
    // BFS from source to target
    let mut visited: HashSet<String> = HashSet::new();
    let mut queue: Vec<(String, Vec<String>)> = vec![(source.to_string(), vec![source.to_string()])];
    
    let adj = build_adjacency(graph);
    
    while !queue.is_empty() {
        let (current, path) = queue.remove(0);
        
        if current == target {
            return Some(path);
        }
        
        if path.len() > max_hops {
            continue;
        }
        
        if visited.contains(&current) {
            continue;
        }
        visited.insert(current.clone());
        
        if let Some(neighbors) = adj.get(&current) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    let mut new_path = path.clone();
                    new_path.push(neighbor.clone());
                    queue.push((neighbor.clone(), new_path));
                }
            }
        }
    }
    
    None
}

/// Build adjacency map from graph
fn build_adjacency(graph: &GraphData) -> HashMap<String, Vec<String>> {
    let mut adj: HashMap<String, Vec<String>> = HashMap::new();
    
    for node in &graph.nodes {
        adj.entry(node.id.clone()).or_default();
    }
    
    for edge in &graph.links {
        adj.entry(edge.source.clone())
            .or_default()
            .push(edge.target.clone());
        adj.entry(edge.target.clone())
            .or_default()
            .push(edge.source.clone());
    }
    
    adj
}

/// Find edge between two nodes
fn find_edge(graph: &GraphData, source: &str, target: &str) -> Option<crate::types::Edge> {
    graph
        .links
        .iter()
        .find(|e| e.source == source && e.target == target)
        .cloned()
}

/// Render subgraph as text
pub fn subgraph_to_text(
    graph: &GraphData,
    nodes: &HashSet<String>,
    edges: &[crate::types::Edge],
    token_budget: usize,
) -> String {
    let char_budget = token_budget * 3;
    let mut lines = Vec::new();
    
    // Sort nodes by degree (most connected first)
    let mut node_degrees: Vec<_> = nodes
        .iter()
        .map(|nid| {
            let degree = graph
                .links
                .iter()
                .filter(|e| e.source == *nid || e.target == *nid)
                .count();
            (nid.clone(), degree)
        })
        .collect();
    node_degrees.sort_by(|a, b| b.1.cmp(&a.1));
    
    // Render nodes
    for (node_id, _) in &node_degrees {
        if let Some(node) = graph.nodes.iter().find(|n| &n.id == node_id) {
            lines.push(format!(
                "NODE {} [src={} loc={} community={}]",
                node.label,
                node.source_file,
                node.source_location,
                node.community.unwrap_or(0)
            ));
        }
    }
    
    // Render edges
    for edge in edges {
        if nodes.contains(&edge.source) && nodes.contains(&edge.target) {
            let src_label = graph
                .nodes
                .iter()
                .find(|n| n.id == edge.source)
                .map(|n| n.label.as_str())
                .unwrap_or(&edge.source);
            
            let tgt_label = graph
                .nodes
                .iter()
                .find(|n| n.id == edge.target)
                .map(|n| n.label.as_str())
                .unwrap_or(&edge.target);
            
            lines.push(format!(
                "EDGE {} --{} [{}]--> {}",
                src_label,
                edge.relation,
                match edge.confidence {
                    Confidence::Extracted => "EXTRACTED",
                    Confidence::Inferred => "INFERRED",
                    Confidence::Ambiguous => "AMBIGUOUS",
                },
                tgt_label
            ));
        }
    }
    
    let output = lines.join("\n");
    if output.len() > char_budget {
        format!(
            "{}\n... (truncated to ~{} token budget)",
            &output[..char_budget],
            token_budget
        )
    } else {
        output
    }
}

/// Execute query
pub fn query(
    graph: &GraphData,
    question: &str,
    use_dfs: bool,
    depth: usize,
    token_budget: usize,
) -> String {
    let terms: Vec<String> = question
        .split_whitespace()
        .filter(|w| w.len() > 2)
        .map(|w| w.to_lowercase())
        .collect();
    
    let scored = score_nodes(graph, &terms);
    
    if scored.is_empty() {
        return "No matching nodes found.".to_string();
    }
    
    // Take top 3 starting nodes
    let start_nodes: Vec<&str> = scored.iter().take(3).map(|(_, id)| *id).collect();
    
    let (nodes, edges) = if use_dfs {
        dfs(graph, &start_nodes, depth)
    } else {
        bfs(graph, &start_nodes, depth)
    };
    
    let header = format!(
        "Traversal: {} depth={} | Start: {} | {} nodes found\n\n",
        if use_dfs { "DFS" } else { "BFS" },
        depth,
        start_nodes
            .iter()
            .filter_map(|id| graph.nodes.iter().find(|n| &n.id == *id).map(|n| n.label.as_str()))
            .collect::<Vec<_>>()
            .join(", "),
        nodes.len()
    );
    
    header + &subgraph_to_text(graph, &nodes, &edges, token_budget)
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
            Edge::new("a.py:B".into(), "a.py:C".into(), "calls".into(), Confidence::Extracted),
            Edge::new("a.py:C".into(), "b.py:D".into(), "imports".into(), Confidence::Inferred),
        ];
        
        GraphData {
            nodes,
            edges,
            metadata: GraphMetadata::new(4, 3, 2),
        }
    }

    #[test]
    fn test_score_nodes() {
        let graph = create_test_graph();
        let terms = vec!["a".to_string()];
        let scores = score_nodes(&graph, &terms);
        
        assert!(!scores.is_empty());
    }

    #[test]
    fn test_bfs() {
        let graph = create_test_graph();
        let start = vec!["a.py:A"];
        let (nodes, _edges) = bfs(&graph, &start, 2);
        
        assert!(!nodes.is_empty());
    }

    #[test]
    fn test_shortest_path() {
        let graph = create_test_graph();
        let path = find_shortest_path(&graph, "a.py:A", "b.py:D", 5);
        
        assert!(path.is_some());
        let path = path.unwrap();
        assert!(path.len() <= 5);
    }
}
