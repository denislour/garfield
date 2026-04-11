//! Core data structures for garfield

use serde::{Deserialize, Serialize};

/// Confidence level của một edge
#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Confidence {
    Extracted,
    Inferred,
    Ambiguous,
}

/// Một node trong graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub label: String,
    pub source_file: String,
    #[serde(default)]
    pub source_location: String,
    #[serde(default)]
    pub community: Option<u32>,
    #[serde(default)]
    pub node_type: Option<String>,
}

impl Node {
    /// Create a new node
    pub fn new(id: String, label: String, source_file: String, source_location: String) -> Self {
        Self {
            id,
            label,
            source_file,
            source_location,
            community: None,
            node_type: None,
        }
    }
}

/// Một edge trong graph
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Edge {
    pub source: String,
    pub target: String,
    pub relation: String,
    pub confidence: Confidence,
}

impl Edge {
    /// Create a new edge
    pub fn new(source: String, target: String, relation: String, confidence: Confidence) -> Self {
        Self {
            source,
            target,
            relation,
            confidence,
        }
    }
}

/// Kết quả extract từ một file
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExtractionResult {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

impl ExtractionResult {
    /// Create empty result
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a node
    pub fn add_node(&mut self, node: Node) {
        self.nodes.push(node);
    }

    /// Add an edge
    pub fn add_edge(&mut self, edge: Edge) {
        self.edges.push(edge);
    }

    /// Merge another extraction result into this one
    pub fn merge(&mut self, other: ExtractionResult) {
        self.nodes.extend(other.nodes);
        self.edges.extend(other.edges);
    }
}

/// Graph metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphMetadata {
    pub total_nodes: usize,
    pub total_edges: usize,
    pub communities: usize,
    pub created: String,
}

impl GraphMetadata {
    /// Create metadata with current timestamp
    pub fn new(total_nodes: usize, total_edges: usize, communities: usize) -> Self {
        Self {
            total_nodes,
            total_edges,
            communities,
            created: chrono_now(),
        }
    }
}

/// Graph structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphData {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
    pub metadata: GraphMetadata,
}

impl GraphData {
    /// Create new graph data
    pub fn new(nodes: Vec<Node>, edges: Vec<Edge>, communities: usize) -> Self {
        let total_nodes = nodes.len();
        let total_edges = edges.len();
        
        Self {
            metadata: GraphMetadata::new(total_nodes, total_edges, communities),
            nodes,
            edges,
        }
    }
}

/// Community assignment result
#[derive(Debug)]
pub struct CommunityResult {
    pub assignments: Vec<u32>,
    pub cohesion_scores: std::collections::HashMap<u32, f64>,
    pub community_sizes: std::collections::HashMap<u32, usize>,
}

/// Build summary
#[derive(Debug)]
pub struct BuildSummary {
    pub total_nodes: usize,
    pub total_edges: usize,
    pub communities: usize,
    pub changed_files: usize,
    pub cached_files: usize,
}

/// File type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    Code,
    Markdown,
    Binary,
}

/// Detected file
#[derive(Debug, Clone)]
pub struct DetectedFile {
    pub path: std::path::PathBuf,
    pub file_type: FileType,
    pub extension: String,
    pub size_bytes: u64,
}

/// Detection statistics
#[derive(Debug)]
pub struct DetectStats {
    pub total: usize,
    pub code: usize,
    pub markdown: usize,
    pub binary: usize,
}

/// Get current timestamp
pub fn chrono_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap();
    format!("{}", duration.as_secs())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_creation() {
        let node = Node::new(
            "test.py:main".to_string(),
            "main".to_string(),
            "test.py".to_string(),
            "L1".to_string(),
        );
        
        assert_eq!(node.id, "test.py:main");
        assert_eq!(node.label, "main");
        assert!(node.community.is_none());
    }

    #[test]
    fn test_edge_creation() {
        let edge = Edge::new(
            "a.py:foo".to_string(),
            "b.py:bar".to_string(),
            "calls".to_string(),
            Confidence::Extracted,
        );
        
        assert_eq!(edge.relation, "calls");
    }

    #[test]
    fn test_extraction_result() {
        let mut result = ExtractionResult::new();
        
        result.add_node(Node::new(
            "test.py:foo".to_string(),
            "foo".to_string(),
            "test.py".to_string(),
            "L1".to_string(),
        ));
        
        assert_eq!(result.nodes.len(), 1);
        assert_eq!(result.edges.len(), 0);
    }
}
