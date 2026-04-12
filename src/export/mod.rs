//! Export module - JSON serialization

use crate::types::GraphData;
use std::fs;
use std::path::Path;

/// Export graph to JSON file
pub fn to_json(graph: &GraphData, output_path: &Path) -> anyhow::Result<()> {
    // Ensure directory exists
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Serialize to JSON with pretty formatting
    let json = serde_json::to_string_pretty(graph)?;
    fs::write(output_path, json)?;

    println!("Exported graph to {}", output_path.display());
    Ok(())
}

/// Load graph from JSON file
pub fn from_json(path: &Path) -> anyhow::Result<GraphData> {
    let content = fs::read_to_string(path)?;
    let graph: GraphData = serde_json::from_str(&content)?;
    Ok(graph)
}

/// Export stats to JSON
pub fn export_stats(graph: &GraphData, output_path: &Path) -> anyhow::Result<()> {
    #[derive(serde::Serialize)]
    struct Stats {
        total_nodes: usize,
        total_edges: usize,
        communities: usize,
        extracted_edges: usize,
        inferred_edges: usize,
        ambiguous_edges: usize,
        created: String,
    }

    use crate::types::Confidence;

    let extracted = graph
        .links
        .iter()
        .filter(|e| matches!(e.confidence, Confidence::Extracted))
        .count();
    let inferred = graph
        .links
        .iter()
        .filter(|e| matches!(e.confidence, Confidence::Inferred))
        .count();
    let ambiguous = graph
        .links
        .iter()
        .filter(|e| matches!(e.confidence, Confidence::Ambiguous))
        .count();

    let stats = Stats {
        total_nodes: graph.nodes.len(),
        total_edges: graph.links.len(),
        communities: graph.metadata.communities,
        extracted_edges: extracted,
        inferred_edges: inferred,
        ambiguous_edges: ambiguous,
        created: graph.metadata.created.clone(),
    };

    let json = serde_json::to_string_pretty(&stats)?;
    fs::write(output_path, json)?;

    Ok(())
}
