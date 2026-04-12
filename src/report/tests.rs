#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Confidence, Edge, GraphMetadata, Node};

    fn create_test_graph() -> GraphData {
        let nodes = vec![
            Node::new("a.py:A".into(), "A".into(), "a.py".into(), "L1".into()),
            Node::new("a.py:B".into(), "B".into(), "a.py".into(), "L1".into()),
            Node::new("b.py:C".into(), "C".into(), "b.py".into(), "L1".into()),
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
                "b.py:C".into(),
                "imports".into(),
                Confidence::Inferred,
            ),
        ];

        GraphData {
            nodes,
            links: edges,
            metadata: GraphMetadata::new(3, 2, 2),
            hyperedges: Vec::new(),
        }
    }

    #[test]
    fn test_corpus_verdict() {
        let small = GraphData::new(vec![], vec![], 0);
        assert!(corpus_verdict(&small).contains("small"));

        let large = GraphData::new(
            (0..100)
                .map(|i| {
                    Node::new(
                        format!("n{}", i),
                        format!("n{}", i),
                        "test.py".into(),
                        "L1".into(),
                    )
                })
                .collect(),
            vec![],
            10,
        );
        assert!(corpus_verdict(&large).contains("large"));
    }

    #[test]
    fn test_is_filtered_node() {
        assert!(is_filtered_node("__init__"));
        assert!(is_filtered_node("_helper"));
        assert!(!is_filtered_node("AuthService"));
        assert!(!is_filtered_node("main"));
    }
}
