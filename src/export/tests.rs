#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Confidence, Edge, GraphMetadata, Node};

    #[test]
    fn test_roundtrip() {
        let graph = GraphData {
            nodes: vec![
                Node::new("a.py:foo".into(), "foo".into(), "a.py".into(), "L1".into()),
                Node::new("b.py:bar".into(), "bar".into(), "b.py".into(), "L1".into()),
            ],
            links: vec![Edge::new(
                "a.py:foo".into(),
                "b.py:bar".into(),
                "calls".into(),
                Confidence::Extracted,
            )],
            metadata: GraphMetadata::new(2, 1, 1),
            hyperedges: Vec::new(),
        };

        // Serialize
        let json = serde_json::to_string_pretty(&graph).unwrap();
        assert!(json.contains("foo"));
        assert!(json.contains("bar"));

        // Deserialize
        let loaded: GraphData = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.nodes.len(), 2);
        assert_eq!(loaded.links.len(), 1);
    }
}
