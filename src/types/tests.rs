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
