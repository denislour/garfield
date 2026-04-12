#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Confidence, Edge, ExtractionResult, Node};

    #[test]
    fn test_validate_valid_extraction() {
        let mut extraction = ExtractionResult::new();
        extraction.add_node(Node::new(
            "a.py:foo".into(),
            "foo".into(),
            "a.py".into(),
            "L1".into(),
        ));
        extraction.add_node(Node::new(
            "a.py:bar".into(),
            "bar".into(),
            "a.py".into(),
            "L2".into(),
        ));
        extraction.add_edge(Edge::new(
            "a.py:foo".into(),
            "a.py:bar".into(),
            "calls".into(),
            Confidence::Extracted,
        ));

        assert!(validate_extraction(&extraction).is_ok());
    }

    #[test]
    fn test_validate_duplicate_node() {
        let mut extraction = ExtractionResult::new();
        extraction.add_node(Node::new(
            "a.py:foo".into(),
            "foo".into(),
            "a.py".into(),
            "L1".into(),
        ));
        extraction.add_node(Node::new(
            "a.py:foo".into(),
            "foo".into(),
            "a.py".into(),
            "L2".into(),
        ));

        assert!(matches!(
            validate_extraction(&extraction),
            Err(ValidationError::DuplicateNodeId(_))
        ));
    }

    #[test]
    fn test_validate_unknown_edge_reference() {
        let mut extraction = ExtractionResult::new();
        extraction.add_node(Node::new(
            "a.py:foo".into(),
            "foo".into(),
            "a.py".into(),
            "L1".into(),
        ));
        extraction.add_edge(Edge::new(
            "a.py:foo".into(),
            "b.py:bar".into(),
            "calls".into(),
            Confidence::Extracted,
        ));

        assert!(matches!(
            validate_extraction(&extraction),
            Err(ValidationError::UnknownNodeReference { .. })
        ));
    }
}
