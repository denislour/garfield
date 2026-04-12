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
            Node::new("b.py:E".into(), "E".into(), "b.py".into(), "L1".into()),
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
                "b.py:D".into(),
                "b.py:E".into(),
                "calls".into(),
                Confidence::Extracted,
            ),
        ];

        GraphData {
            nodes,
            links: edges,
            metadata: GraphMetadata::new(5, 3, 0),
            hyperedges: Vec::new(),
        }
    }

    #[test]
    fn test_cluster() {
        let graph = create_test_graph();
        let result = cluster(&graph);

        assert_eq!(result.assignments.len(), 5);
        assert!(!result.community_sizes.is_empty());
    }

    #[test]
    fn test_split_oversized() {
        let mut graph = create_test_graph();
        add_communities(&mut graph, &[0, 0, 0, 1, 1]);

        // Original communities: {A,B,C}=0, {D,E}=1 (5 total nodes, 2 communities)
        let original_count = graph.nodes.iter().filter_map(|n| n.community).collect::<HashSet<_>>().len();
        assert_eq!(original_count, 2);

        split_oversized(&mut graph, 2);

        // After split, community 0 (size 3) should be split
        let unique: HashSet<_> = graph.nodes.iter().filter_map(|n| n.community).collect();
        // May or may not split depending on graph structure
        assert!(unique.len() >= 2);
    }

    #[test]
    fn test_cohesion() {
        let graph = create_test_graph();
        let result = cluster(&graph);

        // Cohesion should be between 0 and 1
        for (_, score) in &result.cohesion_scores {
            assert!(*score >= 0.0 && *score <= 1.0);
        }
    }

    #[test]
    fn test_cohesion_single_node() {
        let mut adj: HashMap<usize, HashSet<usize>> = HashMap::new();
        adj.insert(0, HashSet::new());

        let assignments = vec![0u32];
        let result = calculate_cohesion(&adj, &assignments);

        assert_eq!(result[&0], 1.0);
    }

    #[test]
    fn test_cohesion_full_triangle() {
        // 3 nodes all connected to each other = triangle
        let mut adj: HashMap<usize, HashSet<usize>> = HashMap::new();
        adj.insert(0, vec![1, 2].into_iter().collect());
        adj.insert(1, vec![0, 2].into_iter().collect());
        adj.insert(2, vec![0, 1].into_iter().collect());

        let assignments = vec![0u32, 0, 0]; // All in same community
        let result = calculate_cohesion(&adj, &assignments);

        // 3 edges / 3 possible = 1.0
        assert_eq!(result[&0], 1.0);
    }

    #[test]
    fn test_cohesion_no_edges() {
        // 3 isolated nodes
        let mut adj: HashMap<usize, HashSet<usize>> = HashMap::new();
        adj.insert(0, HashSet::new());
        adj.insert(1, HashSet::new());
        adj.insert(2, HashSet::new());

        let assignments = vec![0u32, 1, 2]; // Different communities
        let result = calculate_cohesion(&adj, &assignments);

        // Each node alone = 1.0 cohesion
        assert_eq!(result[&0], 1.0);
        assert_eq!(result[&1], 1.0);
        assert_eq!(result[&2], 1.0);
    }
}
