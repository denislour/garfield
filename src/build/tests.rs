#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Confidence, Edge, Node};

    #[test]
    fn test_build_graph() {
        let mut extraction1 = ExtractionResult::new();
        extraction1.add_node(Node::new(
            "a.py:foo".into(),
            "foo".into(),
            "a.py".into(),
            "L1".into(),
        ));
        extraction1.add_node(Node::new(
            "a.py:bar".into(),
            "bar".into(),
            "a.py".into(),
            "L1".into(),
        ));
        extraction1.add_edge(Edge::new(
            "a.py:foo".into(),
            "a.py:bar".into(),
            "calls".into(),
            Confidence::Extracted,
        ));

        let mut extraction2 = ExtractionResult::new();
        extraction2.add_node(Node::new(
            "b.py:baz".into(),
            "baz".into(),
            "b.py".into(),
            "L1".into(),
        ));
        extraction2.add_edge(Edge::new(
            "a.py:foo".into(),
            "b.py:baz".into(),
            "imports".into(),
            Confidence::Inferred,
        ));

        let graph = build_graph(vec![extraction1, extraction2]);

        assert_eq!(graph.nodes.len(), 3);
        assert_eq!(graph.links.len(), 2);
        assert!(graph.metadata.communities > 0);
    }

    #[test]
    fn test_merge_extractions() {
        let mut extraction1 = ExtractionResult::new();
        extraction1.add_node(Node::new(
            "a.py:foo".into(),
            "foo".into(),
            "a.py".into(),
            "L1".into(),
        ));

        let mut extraction2 = ExtractionResult::new();
        extraction2.add_node(Node::new(
            "a.py:foo".into(),
            "foo".into(),
            "a.py".into(),
            "L2".into(),
        )); // Duplicate
        extraction2.add_node(Node::new(
            "b.py:bar".into(),
            "bar".into(),
            "b.py".into(),
            "L1".into(),
        ));

        let merged = merge_extractions(vec![extraction1, extraction2]);

        // Should have only 2 nodes (foo deduplicated)
        assert_eq!(merged.nodes.len(), 2);
    }
}
