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
    /// File type (for rationale nodes)
    #[serde(default)]
    pub file_type: Option<FileType>,
    /// File stem (module name)
    #[serde(default)]
    pub file_stem: Option<String>,
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
            file_type: None,
            file_stem: None,
        }
    }
}

/// Một edge trong graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub source: String,
    pub target: String,
    pub relation: String,
    pub confidence: Confidence,
    #[serde(default)]
    pub confidence_score: f64,
    #[serde(default)]
    pub source_file: String,
    #[serde(default)]
    pub note: Option<String>,
}

impl Edge {
    /// Create a new edge
    pub fn new(source: String, target: String, relation: String, confidence: Confidence) -> Self {
        Self {
            source,
            target,
            relation,
            confidence,
            confidence_score: match confidence {
                Confidence::Extracted => 1.0,
                Confidence::Inferred => 0.75,
                Confidence::Ambiguous => 0.2,
            },
            source_file: String::new(),
            note: None,
        }
    }

    /// Create with all fields
    pub fn with_details(
        source: String,
        target: String,
        relation: String,
        confidence: Confidence,
        confidence_score: f64,
        source_file: String,
        note: Option<String>,
    ) -> Self {
        Self {
            source,
            target,
            relation,
            confidence,
            confidence_score,
            source_file,
            note,
        }
    }
}

/// Hyperedge - a relationship among 3+ nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hyperedge {
    pub id: String,
    pub label: String,
    pub nodes: Vec<String>,
    pub relation: String,
    pub confidence: Confidence,
    #[serde(default)]
    pub confidence_score: f64,
    pub source_file: String,
}

impl Hyperedge {
    pub fn new(
        id: String,
        label: String,
        nodes: Vec<String>,
        relation: String,
        confidence: Confidence,
        source_file: String,
    ) -> Self {
        Self {
            id,
            label,
            nodes,
            relation,
            confidence,
            confidence_score: match confidence {
                Confidence::Extracted => 1.0,
                Confidence::Inferred => 0.75,
                Confidence::Ambiguous => 0.2,
            },
            source_file,
        }
    }
}

/// Kết quả extract từ một file
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExtractionResult {
    pub nodes: Vec<Node>,
    #[serde(alias = "edges", rename = "links")]
    pub links: Vec<Edge>,
    #[serde(default)]
    pub hyperedges: Vec<Hyperedge>,
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
        self.links.push(edge);
    }

    /// Add a hyperedge
    pub fn add_hyperedge(&mut self, hyperedge: Hyperedge) {
        self.hyperedges.push(hyperedge);
    }

    /// Merge another extraction result into this one
    pub fn merge(&mut self, other: ExtractionResult) {
        self.nodes.extend(other.nodes);
        self.links.extend(other.links);
        self.hyperedges.extend(other.hyperedges);
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
    #[serde(alias = "edges", rename = "links")]
    pub links: Vec<Edge>,
    pub metadata: GraphMetadata,
    #[serde(default)]
    pub hyperedges: Vec<Hyperedge>,
}

impl GraphData {
    /// Create new graph data
    pub fn new(nodes: Vec<Node>, links: Vec<Edge>, communities: usize) -> Self {
        let total_nodes = nodes.len();
        let total_edges = links.len();

        Self {
            metadata: GraphMetadata::new(total_nodes, total_edges, communities),
            nodes,
            links,
            hyperedges: Vec::new(),
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileType {
    Code,
    Markdown,
    Binary,
    /// Rationale/docstring nodes extracted from source code
    Rationale,
}

/// Detected file
#[derive(Debug, Clone)]
pub struct DetectedFile {
    pub path: std::path::PathBuf,
    pub file_type: FileType,
    pub extension: String,
    pub size_bytes: u64,
}

/// File summary (TIER 2) - stored in file_summaries.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSummary {
    pub filename: String,
    pub summary: String,
    pub function_count: usize,
    pub functions: Vec<String>,
    #[serde(default)]
    pub public_apis: Vec<String>,
    #[serde(default)]
    pub dependencies: Vec<String>,
    #[serde(default)]
    pub internal_functions: Vec<String>,
    #[serde(default)]
    pub doc_comment: Option<String>,
}

impl FileSummary {
    /// Create a new file summary
    pub fn new(filename: String, summary: String) -> Self {
        Self {
            filename,
            summary,
            function_count: 0,
            functions: Vec::new(),
            public_apis: Vec::new(),
            dependencies: Vec::new(),
            internal_functions: Vec::new(),
            doc_comment: None,
        }
    }
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
    let duration = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
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
        assert_eq!(edge.confidence_score, 1.0);
    }

    #[test]
    fn test_edge_with_details() {
        let edge = Edge::with_details(
            "a.py:foo".to_string(),
            "b.py:bar".to_string(),
            "semantically_similar_to".to_string(),
            Confidence::Inferred,
            0.85,
            "a.py".to_string(),
            Some("Both handle user validation".to_string()),
        );

        assert_eq!(edge.confidence_score, 0.85);
        assert!(edge.note.is_some());
    }

    #[test]
    fn test_hyperedge_creation() {
        let hyperedge = Hyperedge::new(
            "auth_protocol".to_string(),
            "Authentication Protocol".to_string(),
            vec![
                "auth.py:login".to_string(),
                "auth.py:verify".to_string(),
                "auth.py:logout".to_string(),
            ],
            "participate_in".to_string(),
            Confidence::Extracted,
            "auth.py".to_string(),
        );

        assert_eq!(hyperedge.nodes.len(), 3);
        assert_eq!(hyperedge.confidence_score, 1.0);
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
        assert_eq!(result.links.len(), 0);
        assert_eq!(result.hyperedges.len(), 0);
    }
}
