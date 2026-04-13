//! Query engine module
//!
//! ## Serve Flow
//!
//! 1. score_nodes() - Keyword matching with label (1.0) + file (0.5) weights
//! 2. bfs() / dfs() - Graph traversal with depth limit
//! 3. find_shortest_path() - BFS-based path finding
//! 4. get_node() - Full node details
//! 5. get_neighbors() - Direct connections
//! 6. get_community() - All nodes in a community
//! 7. graph_stats() - Summary statistics
//!
//! ## Scoring Details (matching Graphify)
//!
//! - Label match: full weight (1.0 per match)
//! - Source file match: 0.5x weight (0.5 per match)
//! - Combined score determines traversal starting points

use crate::types::{Confidence, GraphData};
use std::collections::{HashMap, HashSet};

/// Query result
#[derive(Debug)]
pub struct QueryResult {
    pub nodes: HashSet<String>,
    pub edges: Vec<crate::types::Edge>,
    pub text: String,
}

/// Node details result
#[derive(Debug)]
pub struct NodeDetails {
    pub id: String,
    pub label: String,
    pub source_file: String,
    pub source_location: String,
    pub community: Option<u32>,
    pub node_type: Option<String>,
    pub incoming_edges: Vec<EdgeInfo>,
    pub outgoing_edges: Vec<EdgeInfo>,
    pub hyperedge: Option<HyperedgeInfo>,  // ✨ Module info
}

/// Hyperedge info for node details
#[derive(Debug, Clone)]
pub struct HyperedgeInfo {
    pub id: String,
    pub label: String,
    pub member_count: usize,
    pub relation: String,
    pub confidence_score: f64,
}

#[derive(Debug, Clone)]
pub struct EdgeInfo {
    pub source: String,
    pub target: String,
    pub source_label: String,
    pub target_label: String,
    pub relation: String,
    pub confidence: String,
}

/// Community info
#[derive(Debug)]
pub struct CommunityInfo {
    pub id: u32,
    pub size: usize,
    pub cohesion: f64,
    pub label: String,
    pub nodes: Vec<CommunityNode>,
}

#[derive(Debug, Clone)]
pub struct CommunityNode {
    pub id: String,
    pub label: String,
    pub degree: usize,
}

/// Graph statistics
#[derive(Debug)]
pub struct GraphStats {
    pub total_nodes: usize,
    pub total_edges: usize,
    pub communities: usize,
    pub confidence_breakdown: ConfidenceBreakdown,
    pub avg_degree: f64,
    pub most_connected: Vec<GodNodeInfo>,
}

#[derive(Debug)]
pub struct ConfidenceBreakdown {
    pub extracted: usize,
    pub inferred: usize,
    pub ambiguous: usize,
}

#[derive(Debug, Clone)]
pub struct GodNodeInfo {
    pub id: String,
    pub label: String,
    pub degree: usize,
    pub source_file: String,
}

/// Score nodes by keyword match (matching Graphify scoring)
/// 
/// Scoring rules:
/// - Label match = 1.0 per match
/// - Source file match = 0.5 per match
/// - Combined score = label_score + source_score
/// 
/// Returns sorted list of (score, node_id)
pub fn score_nodes<'a>(graph: &'a GraphData, terms: &'a [String]) -> Vec<(f64, &'a str)> {
    let mut scored = Vec::new();

    for node in &graph.nodes {
        let label_lower = node.label.to_lowercase();
        let source_lower = node.source_file.to_lowercase();

        // Label match: full weight (1.0)
        let label_score: f64 = terms
            .iter()
            .filter(|t| label_lower.contains(&t.to_lowercase()))
            .count() as f64;

        // Source file match: 0.5x weight
        let source_score: f64 = terms
            .iter()
            .filter(|t| source_lower.contains(&t.to_lowercase()))
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
    let mut stack: Vec<(&str, usize)> = start_nodes.iter().rev().map(|s| (*s, 0)).collect();

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

/// Find shortest path between two nodes (BFS-based)
pub fn find_shortest_path(
    graph: &GraphData,
    source: &str,
    target: &str,
    max_hops: usize,
) -> Option<Vec<String>> {
    let mut visited: HashSet<String> = HashSet::new();
    let mut queue: Vec<(String, Vec<String>)> =
        vec![(source.to_string(), vec![source.to_string()])];

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
        // For undirected traversal, also add reverse
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

/// Render subgraph as text with token budget
pub fn subgraph_to_text(
    graph: &GraphData,
    nodes: &HashSet<String>,
    edges: &[crate::types::Edge],
    token_budget: usize,
) -> String {
    let char_budget = token_budget * 4; // ~4 chars per token average
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

    // Render nodes with hyperedge info
    lines.push("## Nodes".to_string());
    for (node_id, _) in &node_degrees {
        if let Some(node) = graph.nodes.iter().find(|n| &n.id == node_id) {
            // Find hyperedge for this node
            let hyperedge_label = graph.hyperedges.iter()
                .find(|he| he.nodes.contains(&node.id))
                .map(|he| format!(" [{}]", he.label))
                .unwrap_or_default();

            lines.push(format!(
                "  • {}{} [{} @ {}] (community: {})",
                node.label,
                hyperedge_label,
                node.source_file,
                node.source_location,
                node.community.unwrap_or(0)
            ));
        }
    }

    // Render edges
    lines.push("\n## Edges".to_string());
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

            let conf_str = match edge.confidence {
                Confidence::Extracted => "EXTRACTED",
                Confidence::Inferred => "INFERRED",
                Confidence::Ambiguous => "AMBIGUOUS",
            };

            lines.push(format!(
                "  {} --[{}: {}]--> {}",
                src_label, edge.relation, conf_str, tgt_label
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

    let traversal = if use_dfs { "DFS" } else { "BFS" };
    
    let start_labels: Vec<_> = start_nodes
        .iter()
        .filter_map(|id| graph.nodes.iter().find(|n| &n.id == *id).map(|n| n.label.as_str()))
        .collect();

    let header = format!(
        "Query: \"{}\"\nTraversal: {} depth={} | Start: {} | {} nodes found\n\n",
        question,
        traversal,
        depth,
        start_labels.join(", "),
        nodes.len()
    );

    header + &subgraph_to_text(graph, &nodes, &edges, token_budget)
}

/// Get detailed node information
pub fn get_node(graph: &GraphData, identifier: &str) -> Option<NodeDetails> {
    // Find node by ID or label
    let node = graph.nodes.iter().find(|n| {
        n.id == identifier || n.label.to_lowercase() == identifier.to_lowercase()
    })?;

    // Collect incoming edges
    let incoming: Vec<_> = graph.links.iter()
        .filter(|e| e.target == node.id)
        .map(|e| {
            let src_label = graph.nodes.iter()
                .find(|n| n.id == e.source)
                .map(|n| n.label.clone())
                .unwrap_or_else(|| e.source.clone());
            EdgeInfo {
                source: e.source.clone(),
                target: e.target.clone(),
                source_label: src_label,
                target_label: node.label.clone(),
                relation: e.relation.clone(),
                confidence: format!("{:?}", e.confidence),
            }
        })
        .collect();

    // Collect outgoing edges
    let outgoing: Vec<_> = graph.links.iter()
        .filter(|e| e.source == node.id)
        .map(|e| {
            let tgt_label = graph.nodes.iter()
                .find(|n| n.id == e.target)
                .map(|n| n.label.clone())
                .unwrap_or_else(|| e.target.clone());
            EdgeInfo {
                source: e.source.clone(),
                target: e.target.clone(),
                source_label: node.label.clone(),
                target_label: tgt_label,
                relation: e.relation.clone(),
                confidence: format!("{:?}", e.confidence),
            }
        })
        .collect();

    // Find hyperedge containing this node
    let hyperedge = graph.hyperedges.iter()
        .find(|he| he.nodes.contains(&node.id))
        .map(|he| HyperedgeInfo {
            id: he.id.clone(),
            label: he.label.clone(),
            member_count: he.nodes.len(),
            relation: he.relation.clone(),
            confidence_score: he.confidence_score,
        });

    Some(NodeDetails {
        id: node.id.clone(),
        label: node.label.clone(),
        source_file: node.source_file.clone(),
        source_location: node.source_location.clone(),
        community: node.community,
        node_type: node.node_type.clone(),
        incoming_edges: incoming,
        outgoing_edges: outgoing,
        hyperedge,
    })
}

/// Get hyperedge by ID
pub fn get_hyperedge(graph: &GraphData, identifier: &str) -> Option<HyperedgeInfo> {
    graph.hyperedges.iter()
        .find(|he| he.id == identifier || he.label.to_lowercase().contains(&identifier.to_lowercase()))
        .map(|he| HyperedgeInfo {
            id: he.id.clone(),
            label: he.label.clone(),
            member_count: he.nodes.len(),
            relation: he.relation.clone(),
            confidence_score: he.confidence_score,
        })
}

/// Get neighbors of a node
pub fn get_neighbors(
    graph: &GraphData,
    identifier: &str,
    max_results: usize,
) -> Vec<EdgeInfo> {
    let node = match graph.nodes.iter().find(|n| {
        n.id == identifier || n.label.to_lowercase() == identifier.to_lowercase()
    }) {
        Some(n) => n,
        None => return Vec::new(),
    };
    
    let mut neighbors = Vec::new();
    
    for edge in &graph.links {
        if edge.source == node.id {
            let tgt_label = graph.nodes.iter()
                .find(|n| n.id == edge.target)
                .map(|n| n.label.clone())
                .unwrap_or_else(|| edge.target.clone());
            neighbors.push(EdgeInfo {
                source: edge.source.clone(),
                target: edge.target.clone(),
                source_label: node.label.clone(),
                target_label: tgt_label,
                relation: edge.relation.clone(),
                confidence: format!("{:?}", edge.confidence),
            });
        } else if edge.target == node.id {
            let src_label = graph.nodes.iter()
                .find(|n| n.id == edge.source)
                .map(|n| n.label.clone())
                .unwrap_or_else(|| edge.source.clone());
            neighbors.push(EdgeInfo {
                source: edge.source.clone(),
                target: edge.target.clone(),
                source_label: src_label,
                target_label: node.label.clone(),
                relation: edge.relation.clone(),
                confidence: format!("{:?}", edge.confidence),
            });
        }
    }
    
    neighbors.truncate(max_results);
    neighbors
}

/// Get all nodes in a community
pub fn get_community(graph: &GraphData, community_id: u32) -> Option<CommunityInfo> {
    let nodes: Vec<_> = graph.nodes.iter()
        .filter(|n| n.community == Some(community_id))
        .collect();
    
    if nodes.is_empty() {
        return None;
    }
    
    // Calculate degree for each node
    let community_nodes: Vec<CommunityNode> = nodes.iter()
        .map(|n| {
            let degree = graph.links.iter()
                .filter(|e| e.source == n.id || e.target == n.id)
                .count();
            CommunityNode {
                id: n.id.clone(),
                label: n.label.clone(),
                degree,
            }
        })
        .collect();
    
    // Calculate cohesion
    let size = nodes.len();
    let mut actual_edges = 0usize;
    for i in 0..size {
        for j in (i + 1)..size {
            let nid_i = &nodes[i].id;
            let nid_j = &nodes[j].id;
            if graph.links.iter().any(|e| 
                (e.source == *nid_i && e.target == *nid_j) ||
                (e.source == *nid_j && e.target == *nid_i)
            ) {
                actual_edges += 1;
            }
        }
    }
    let possible = (size * (size - 1)) as f64 / 2.0;
    let cohesion = if possible > 0.0 { actual_edges as f64 / possible } else { 1.0 };
    
    // Generate label
    let label = format!("Community {}", community_id);
    
    Some(CommunityInfo {
        id: community_id,
        size,
        cohesion: (cohesion * 100.0).round() / 100.0,
        label,
        nodes: community_nodes,
    })
}

/// Get graph statistics
pub fn graph_stats(graph: &GraphData) -> GraphStats {
    let total_nodes = graph.nodes.len();
    let total_edges = graph.links.len();
    
    // Confidence breakdown
    let mut extracted = 0usize;
    let mut inferred = 0usize;
    let mut ambiguous = 0usize;
    
    for edge in &graph.links {
        match edge.confidence {
            Confidence::Extracted => extracted += 1,
            Confidence::Inferred => inferred += 1,
            Confidence::Ambiguous => ambiguous += 1,
        }
    }
    
    // Calculate average degree
    let mut degree_sum = 0usize;
    for node in &graph.nodes {
        let degree = graph.links.iter()
            .filter(|e| e.source == node.id || e.target == node.id)
            .count();
        degree_sum += degree;
    }
    let avg_degree = if total_nodes > 0 { 
        degree_sum as f64 / total_nodes as f64 
    } else { 
        0.0 
    };
    
    // Find most connected nodes
    let mut node_degrees: Vec<_> = graph.nodes.iter()
        .map(|n| {
            let degree = graph.links.iter()
                .filter(|e| e.source == n.id || e.target == n.id)
                .count();
            GodNodeInfo {
                id: n.id.clone(),
                label: n.label.clone(),
                degree,
                source_file: n.source_file.clone(),
            }
        })
        .collect();
    node_degrees.sort_by(|a, b| b.degree.cmp(&a.degree));
    let most_connected: Vec<_> = node_degrees.into_iter().take(10).collect();
    
    GraphStats {
        total_nodes,
        total_edges,
        communities: graph.metadata.communities,
        confidence_breakdown: ConfidenceBreakdown {
            extracted,
            inferred,
            ambiguous,
        },
        avg_degree: (avg_degree * 100.0).round() / 100.0,
        most_connected,
    }
}

/// Format graph stats as readable text
pub fn format_graph_stats(stats: &GraphStats) -> String {
    let mut lines = Vec::new();
    
    lines.push("## Graph Statistics".to_string());
    lines.push(format!("  Total Nodes: {}", stats.total_nodes));
    lines.push(format!("  Total Edges: {}", stats.total_edges));
    lines.push(format!("  Communities: {}", stats.communities));
    lines.push(format!("  Avg Degree: {:.2}", stats.avg_degree));
    
    lines.push("\n## Confidence Breakdown".to_string());
    lines.push(format!("  EXTRACTED: {}", stats.confidence_breakdown.extracted));
    lines.push(format!("  INFERRED: {}", stats.confidence_breakdown.inferred));
    lines.push(format!("  AMBIGUOUS: {}", stats.confidence_breakdown.ambiguous));
    
    if !stats.most_connected.is_empty() {
        lines.push("\n## Most Connected Nodes".to_string());
        for (i, node) in stats.most_connected.iter().enumerate().take(5) {
            lines.push(format!("  {}. {} (degree: {})", i + 1, node.label, node.degree));
        }
    }
    
    lines.join("\n")
}

/// Get node body - reads directly from source files
/// 
/// Parse node ID to extract file_stem and function name,
/// then read the source file and extract the function body.
pub fn get_node_body(node_id: &str) -> Option<String> {
    // Parse ID: format is "file_stem:function_name" or "file_stem:ClassName::method"
    let parts: Vec<&str> = node_id.split(':').collect();
    if parts.len() < 2 {
        return None;
    }
    
    let file_stem = parts[0];
    let name = parts[1..].join(":"); // Handle "Class::method" format
    
    // Find source file by file_stem
    let source_file = find_source_file(file_stem)?;
    
    // Read source file
    let content = match std::fs::read_to_string(&source_file) {
        Ok(c) => c,
        Err(_) => return None,
    };
    
    // Extract function/method body
    extract_function_body(&content, &name)
}

/// Find source file path by file_stem
fn find_source_file(file_stem: &str) -> Option<std::path::PathBuf> {
    // Search in common locations
    let search_dirs = [
        "./src",
        ".",
        "src",
    ];
    
    let extensions = ["rs", "py", "ts", "js", "go", "java"];
    
    for dir in &search_dirs {
        for ext in &extensions {
            let path = std::path::Path::new(dir).join(format!("{}.{}", file_stem, ext));
            if path.exists() {
                return Some(path);
            }
            
            // Try with raw/ prefix (for graphify style)
            let raw_path = std::path::Path::new(dir).join("raw").join(format!("{}.{}", file_stem, ext));
            if raw_path.exists() {
                return Some(raw_path);
            }
        }
    }
    
    None
}

/// Extract function body from source content
fn extract_function_body(content: &str, fn_name: &str) -> Option<String> {
    let lines: Vec<&str> = content.lines().collect();
    
    // Try to find function definition
    let search_patterns = [
        format!("fn {}", fn_name),
        format!("pub fn {}", fn_name),
        format!("def {}", fn_name),
        format!("func {}", fn_name),
        format!("func ({}", fn_name), // TypeScript/Go style
    ];
    
    let mut start_line = None;
    for (i, line) in lines.iter().enumerate() {
        for pattern in &search_patterns {
            if line.contains(pattern.as_str()) {
                start_line = Some(i);
                break;
            }
        }
        if start_line.is_some() {
            break;
        }
    }
    
    let start = start_line?;
    
    // Find end of function (matching brace)
    let mut brace_count = 0;
    let mut in_function = false;
    let mut end_line = start;
    
    for i in start..lines.len() {
        let line = lines[i];
        for c in line.chars() {
            match c {
                '{' | '(' => {
                    brace_count += 1;
                    in_function = true;
                }
                '}' | ')' => {
                    brace_count -= 1;
                }
                _ => {}
            }
        }
        if in_function && brace_count <= 0 {
            end_line = i;
            break;
        }
        end_line = i;
    }
    
    // Extract and return function body
    let body: String = lines[start..=end_line.min(lines.len() - 1)]
        .iter()
        .enumerate()
        .map(|(_i, l)| format!("{:4}| {}", "", l))
        .collect::<Vec<_>>()
        .join("\n");
    
    Some(body)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Confidence, Edge, GraphMetadata, Node};

    fn create_test_graph() -> GraphData {
        let nodes = vec![
            Node::new("a.py:A".into(), "A".into(), "a.py".into(), "L1".into()),
            Node::new("a.py:B".into(), "B".into(), "a.py".into(), "L1".into()),
            Node::new("a.py:C".into(), "C".into(), "a.py".into(), "L1".into()),
            Node::new("b.py:D".into(), "D".into(), "b.py".into(), "L1".into()),
        ];

        let edges = vec![
            Edge::new(
                "a.py:A".into(),
                "a.py:B".into(),
                "calls".into(),
                Confidence::Extracted,
            ),
            Edge::new(
                "a.py:B".into(),
                "a.py:C".into(),
                "calls".into(),
                Confidence::Extracted,
            ),
            Edge::new(
                "a.py:C".into(),
                "b.py:D".into(),
                "imports".into(),
                Confidence::Inferred,
            ),
        ];

        GraphData {
            nodes,
            links: edges,
            metadata: GraphMetadata::new(4, 3, 2),
            hyperedges: Vec::new(),
        }
    }

    #[test]
    fn test_score_nodes() {
        let graph = create_test_graph();
        let terms = vec!["a".to_string()];
        let scores = score_nodes(&graph, &terms);

        assert!(!scores.is_empty());
        // a.py:A should score higher (label match)
        assert!(scores[0].1.contains("a.py"));
    }

    #[test]
    fn test_bfs() {
        let graph = create_test_graph();
        let start = vec!["a.py:A"];
        let (nodes, _edges) = bfs(&graph, &start, 2);

        assert!(!nodes.is_empty());
        assert!(nodes.contains(&"a.py:A".to_string()));
    }

    #[test]
    fn test_shortest_path() {
        let graph = create_test_graph();
        let path = find_shortest_path(&graph, "a.py:A", "b.py:D", 5);

        assert!(path.is_some());
        let path = path.unwrap();
        assert!(path.len() <= 5);
    }
    
    #[test]
    fn test_get_node() {
        let graph = create_test_graph();
        let details = get_node(&graph, "a.py:A");
        
        assert!(details.is_some());
        let details = details.unwrap();
        assert_eq!(details.label, "A");
        assert_eq!(details.outgoing_edges.len(), 1);
    }
    
    #[test]
    fn test_graph_stats() {
        let graph = create_test_graph();
        let stats = graph_stats(&graph);
        
        assert_eq!(stats.total_nodes, 4);
        assert_eq!(stats.total_edges, 3);
        assert_eq!(stats.confidence_breakdown.extracted, 2);
        assert_eq!(stats.confidence_breakdown.inferred, 1);
    }

    #[test]
    fn test_get_node_body() {
        // Test with actual function from serve.rs
        let body = get_node_body("serve:find_shortest_path");
        assert!(body.is_some());
        let body = body.unwrap();
        assert!(body.contains("fn find_shortest_path"));
        assert!(body.contains("pub fn"));
    }

    #[test]
    fn test_get_node_body_nonexistent() {
        let body = get_node_body("nonexistent:function");
        assert!(body.is_none());
    }

    #[test]
    fn test_find_source_file() {
        let path = find_source_file("serve");
        assert!(path.is_some());
        let path = path.unwrap();
        assert!(path.to_string_lossy().contains("serve"));
    }
}
