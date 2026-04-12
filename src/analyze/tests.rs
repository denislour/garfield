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
            Edge::new("a.py:A".into(), "a.py:B".into(), "calls".into(), Confidence::Extracted),
            Edge::new("a.py:A".into(), "a.py:C".into(), "calls".into(), Confidence::Inferred),
            Edge::new("a.py:B".into(), "b.py:D".into(), "imports".into(), Confidence::Extracted),
        ];

        GraphData {
            nodes,
            links: edges,
            metadata: GraphMetadata::new(4, 3, 2),
            hyperedges: Vec::new(),
        }
    }

    #[test]
    fn test_find_god_nodes() {
        let graph = create_test_graph();
        let gods = find_god_nodes(&graph, 2);
        assert!(!gods.is_empty());
        assert!(gods[0].degree >= 2);
    }

    #[test]
    fn test_confidence_stats() {
        let graph = create_test_graph();
        let stats = count_confidence(&graph);
        assert_eq!(stats.extracted, 2);
        assert_eq!(stats.inferred, 1);
        assert_eq!(stats.ambiguous, 0);
    }
    
    #[test]
    fn test_suggest_questions() {
        let graph = create_test_graph();
        let questions = suggest_questions(&graph, 5);
        assert!(!questions.is_empty());
    }
    
    #[test]
    fn test_graph_diff() {
        let old = create_test_graph();
        
        let mut new_nodes = old.nodes.clone();
        new_nodes.push(Node::new("c.py:E".into(), "E".into(), "c.py".into(), "L1".into()));
        
        let mut new_edges = old.links.clone();
        new_edges.push(Edge::new("a.py:A".into(), "c.py:E".into(), "calls".into(), Confidence::Inferred));
        
        let new_graph = GraphData {
            nodes: new_nodes,
            links: new_edges,
            metadata: GraphMetadata::new(5, 4, 2),
            hyperedges: Vec::new(),
        };
        
        let diff = graph_diff(&old, &new_graph);
        assert_eq!(diff.new_nodes.len(), 1);
        assert_eq!(diff.new_edges.len(), 1);
        assert!(diff.summary.contains("new"));
    }
}
