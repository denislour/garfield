//! Validation module

use crate::types::{GraphData, ExtractionResult};

/// Validation error
#[derive(Debug)]
pub enum ValidationError {
    DuplicateNodeId(String),
    UnknownNodeReference { edge: String, node: String },
    EmptyNodeId,
    EmptyLabel,
    InvalidConfidence(String),
}

/// Validate extraction result
pub fn validate_extraction(extraction: &ExtractionResult) -> Result<(), ValidationError> {
    // Check for empty IDs
    for node in &extraction.nodes {
        if node.id.is_empty() {
            return Err(ValidationError::EmptyNodeId);
        }
        if node.label.is_empty() {
            return Err(ValidationError::EmptyLabel);
        }
    }
    
    // Check for duplicate node IDs
    let mut seen_ids = std::collections::HashSet::new();
    for node in &extraction.nodes {
        if !seen_ids.insert(node.id.clone()) {
            return Err(ValidationError::DuplicateNodeId(node.id.clone()));
        }
    }
    
    // Collect known node IDs
    let known_ids: std::collections::HashSet<_> = extraction
        .nodes
        .iter()
        .map(|n| n.id.clone())
        .collect();
    
    // Check edge references
    for edge in &extraction.edges {
        if !known_ids.contains(&edge.source) {
            return Err(ValidationError::UnknownNodeReference {
                edge: format!("{} -> {}", edge.source, edge.target),
                node: edge.source.clone(),
            });
        }
        if !known_ids.contains(&edge.target) {
            return Err(ValidationError::UnknownNodeReference {
                edge: format!("{} -> {}", edge.source, edge.target),
                node: edge.target.clone(),
            });
        }
    }
    
    Ok(())
}

/// Validate full graph
pub fn validate_graph(graph: &GraphData) -> Result<(), ValidationError> {
    // Check for empty fields
    for node in &graph.nodes {
        if node.id.is_empty() {
            return Err(ValidationError::EmptyNodeId);
        }
        if node.label.is_empty() {
            return Err(ValidationError::EmptyLabel);
        }
    }
    
    // Check for duplicate node IDs
    let mut seen_ids = std::collections::HashSet::new();
    for node in &graph.nodes {
        if !seen_ids.insert(node.id.clone()) {
            return Err(ValidationError::DuplicateNodeId(node.id.clone()));
        }
    }
    
    // Collect known node IDs
    let known_ids: std::collections::HashSet<_> = graph
        .nodes
        .iter()
        .map(|n| n.id.clone())
        .collect();
    
    // Check all edges reference valid nodes
    for edge in &graph.edges {
        if !known_ids.contains(&edge.source) {
            return Err(ValidationError::UnknownNodeReference {
                edge: format!("{} -> {}", edge.source, edge.target),
                node: edge.source.clone(),
            });
        }
        if !known_ids.contains(&edge.target) {
            return Err(ValidationError::UnknownNodeReference {
                edge: format!("{} -> {}", edge.source, edge.target),
                node: edge.target.clone(),
            });
        }
    }
    
    // Check metadata consistency
    if graph.metadata.total_nodes != graph.nodes.len() {
        eprintln!(
            "Warning: metadata.total_nodes ({}) != nodes.len() ({})",
            graph.metadata.total_nodes,
            graph.nodes.len()
        );
    }
    
    if graph.metadata.total_edges != graph.edges.len() {
        eprintln!(
            "Warning: metadata.total_edges ({}) != edges.len() ({})",
            graph.metadata.total_edges,
            graph.edges.len()
        );
    }
    
    Ok(())
}

/// Pretty print validation errors
pub fn format_error(err: &ValidationError) -> String {
    match err {
        ValidationError::DuplicateNodeId(id) => {
            format!("Duplicate node ID: {}", id)
        }
        ValidationError::UnknownNodeReference { edge, node } => {
            format!("Edge '{}' references unknown node: {}", edge, node)
        }
        ValidationError::EmptyNodeId => {
            "Node has empty ID".to_string()
        }
        ValidationError::EmptyLabel => {
            "Node has empty label".to_string()
        }
        ValidationError::InvalidConfidence(value) => {
            format!("Invalid confidence value: {}. Expected EXTRACTED, INFERRED, or AMBIGUOUS", value)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Node, Edge, Confidence, ExtractionResult};

    #[test]
    fn test_validate_valid_extraction() {
        let mut extraction = ExtractionResult::new();
        extraction.add_node(Node::new("a.py:foo".into(), "foo".into(), "a.py".into(), "L1".into()));
        extraction.add_node(Node::new("a.py:bar".into(), "bar".into(), "a.py".into(), "L2".into()));
        extraction.add_edge(Edge::new("a.py:foo".into(), "a.py:bar".into(), "calls".into(), Confidence::Extracted));
        
        assert!(validate_extraction(&extraction).is_ok());
    }

    #[test]
    fn test_validate_duplicate_node() {
        let mut extraction = ExtractionResult::new();
        extraction.add_node(Node::new("a.py:foo".into(), "foo".into(), "a.py".into(), "L1".into()));
        extraction.add_node(Node::new("a.py:foo".into(), "foo".into(), "a.py".into(), "L2".into()));
        
        assert!(matches!(
            validate_extraction(&extraction),
            Err(ValidationError::DuplicateNodeId(_))
        ));
    }

    #[test]
    fn test_validate_unknown_edge_reference() {
        let mut extraction = ExtractionResult::new();
        extraction.add_node(Node::new("a.py:foo".into(), "foo".into(), "a.py".into(), "L1".into()));
        extraction.add_edge(Edge::new("a.py:foo".into(), "b.py:bar".into(), "calls".into(), Confidence::Extracted));
        
        assert!(matches!(
            validate_extraction(&extraction),
            Err(ValidationError::UnknownNodeReference { .. })
        ));
    }
}
