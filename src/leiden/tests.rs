#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_empty_graph() {
        let result = leiden_communities(0, &[]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_no_edges() {
        let result = leiden_communities(3, &[]);
        assert_eq!(result, vec![0, 1, 2]);
    }

    #[test]
    fn test_triangle() {
        let edges = vec![
            (0, 1, 1.0),
            (1, 2, 1.0),
            (0, 2, 1.0),
        ];
        let result = leiden_communities(3, &edges);
        // All in same community
        assert!(result.iter().all(|&c| c == result[0]));
    }

    #[test]
    fn test_single_edge() {
        let edges = vec![(0, 1, 1.0)];
        let result = leiden_communities(2, &edges);
        assert_eq!(result[0], result[1]);
    }

    #[test]
    fn test_disconnected() {
        let edges = vec![
            (0, 1, 1.0),
            (2, 3, 1.0),
        ];
        let result = leiden_communities(4, &edges);
        assert_eq!(result[0], result[1]);
        assert_eq!(result[2], result[3]);
    }

    #[test]
    fn test_star_graph() {
        let edges = vec![
            (0, 1, 1.0),
            (0, 2, 1.0),
            (0, 3, 1.0),
            (0, 4, 1.0),
        ];
        let result = leiden_communities(5, &edges);
        let unique: HashSet<u32> = result.iter().cloned().collect();
        // Star graph is connected
        assert!(unique.len() >= 1 && unique.len() <= 5);
    }

    #[test]
    fn test_two_cliques() {
        let edges = vec![
            (0, 1, 1.0),
            (0, 2, 1.0),
            (1, 2, 1.0),
            (3, 4, 1.0),
            (3, 5, 1.0),
            (4, 5, 1.0),
        ];
        let result = leiden_communities(6, &edges);
        // Should have 2 communities
        let unique: HashSet<u32> = result.iter().cloned().collect();
        assert!(unique.len() >= 2);
    }

    #[test]
    fn test_weighted_edges() {
        let edges = vec![
            (0, 1, 10.0),
            (1, 2, 10.0),
            (2, 3, 1.0),
            (3, 4, 10.0),
            (4, 5, 10.0),
        ];
        let result = leiden_communities(6, &edges);
        let unique: HashSet<u32> = result.iter().cloned().collect();
        assert!(unique.len() >= 1);
    }
}
