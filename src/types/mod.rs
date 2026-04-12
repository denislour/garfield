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
