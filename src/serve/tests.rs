#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Confidence, Edge, GraphMetadata, Node};

    fn create_test_graph() -> GraphData {
        let nodes = vec![
            Node::new("a.py:A".into(), "A".into(), "a.py".into(), "L1".into()),
            Node::new("a.py:B".into(), "B".into(), "a.py".into(), "L1".into()),
            Node::new("a.py:C".into(), "C".into(), "a.py".into(), "L1".into()),
            Node::new("b.py:D".into(), "D".into(), "b.py".into(), "L1".into()),
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
                "a.py:C".into(),
                "calls".into(),
                Confidence::Extracted,
            ),
            Edge::new(
                "a.py:C".into(),
                "b.py:D".into(),
                "imports".into(),
                Confidence::Inferred,
            ),
        ];

        GraphData {
            nodes,
            links: edges,
            metadata: GraphMetadata::new(4, 3, 2),
            hyperedges: Vec::new(),
        }
    }

    #[test]
    fn test_score_nodes() {
        let graph = create_test_graph();
        let terms = vec!["a".to_string()];
        let scores = score_nodes(&graph, &terms);

        assert!(!scores.is_empty());
        // a.py:A should score higher (label match)
        assert!(scores[0].1.contains("a.py"));
    }

    #[test]
    fn test_bfs() {
        let graph = create_test_graph();
        let start = vec!["a.py:A"];
        let (nodes, _edges) = bfs(&graph, &start, 2);

        assert!(!nodes.is_empty());
        assert!(nodes.contains(&"a.py:A".to_string()));
    }

    #[test]
    fn test_shortest_path() {
        let graph = create_test_graph();
        let path = find_shortest_path(&graph, "a.py:A", "b.py:D", 5);

        assert!(path.is_some());
        let path = path.unwrap();
        assert!(path.len() <= 5);
    }
    
    #[test]
    fn test_get_node() {
        let graph = create_test_graph();
        let details = get_node(&graph, "a.py:A");
        
        assert!(details.is_some());
        let details = details.unwrap();
        assert_eq!(details.label, "A");
        assert_eq!(details.outgoing_edges.len(), 1);
    }
    
    #[test]
    fn test_graph_stats() {
        let graph = create_test_graph();
        let stats = graph_stats(&graph);
        
        assert_eq!(stats.total_nodes, 4);
        assert_eq!(stats.total_edges, 3);
        assert_eq!(stats.confidence_breakdown.extracted, 2);
        assert_eq!(stats.confidence_breakdown.inferred, 1);
    }
}
