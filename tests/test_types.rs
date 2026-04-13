//! Comprehensive unit tests for types module

use garfield::types::{
    Node, Edge, Hyperedge, GraphData, GraphMetadata, ExtractionResult, Confidence,
    FileType, BuildSummary, CommunityResult,
};

mod node_tests {
    use super::*;

    #[test]
    fn test_node_creation() {
        let node = Node::new(
            "test:func".to_string(),
            "test_func".to_string(),
            "src/test.rs".to_string(),
            "src/test.rs @ L1".to_string(),
        );
        assert_eq!(node.id, "test:func");
        assert_eq!(node.label, "test_func");
        assert_eq!(node.source_file, "src/test.rs");
    }

    #[test]
    fn test_node_default_fields() {
        let node = Node::new(
            "id".to_string(),
            "label".to_string(),
            "file.rs".to_string(),
            "file.rs @ L1".to_string(),
        );
        assert!(node.community.is_none());
        assert!(node.node_type.is_none());
        assert!(node.file_type.is_none());
        assert!(node.file_stem.is_none());
    }
}

mod edge_tests {
    use super::*;

    #[test]
    fn test_edge_creation() {
        let edge = Edge::new(
            "a".to_string(),
            "b".to_string(),
            "calls".to_string(),
            Confidence::Extracted,
        );
        assert_eq!(edge.source, "a");
        assert_eq!(edge.target, "b");
        assert_eq!(edge.relation, "calls");
        assert_eq!(edge.confidence, Confidence::Extracted);
    }

    #[test]
    fn test_edge_with_confidence_score() {
        let edge = Edge::with_details(
            "a".to_string(),
            "b".to_string(),
            "defines".to_string(),
            Confidence::Inferred,
            0.8,
            "src/mod.rs".to_string(),
            Some("test note".to_string()),
        );
        assert_eq!(edge.confidence_score, 0.8);
        assert_eq!(edge.source_file, "src/mod.rs");
        assert_eq!(edge.note, Some("test note".to_string()));
    }

    #[test]
    fn test_edge_confidence_scores_auto() {
        let extracted = Edge::new(
            "a".to_string(),
            "b".to_string(),
            "calls".to_string(),
            Confidence::Extracted,
        );
        assert_eq!(extracted.confidence_score, 1.0);

        let inferred = Edge::new(
            "a".to_string(),
            "b".to_string(),
            "calls".to_string(),
            Confidence::Inferred,
        );
        assert_eq!(inferred.confidence_score, 0.75);

        let ambiguous = Edge::new(
            "a".to_string(),
            "b".to_string(),
            "calls".to_string(),
            Confidence::Ambiguous,
        );
        assert_eq!(ambiguous.confidence_score, 0.2);
    }
}

mod hyperedge_tests {
    use super::*;

    #[test]
    fn test_hyperedge_creation() {
        let hyperedge = Hyperedge {
            id: "module_a".to_string(),
            label: "Module A".to_string(),
            nodes: vec!["n1".to_string(), "n2".to_string(), "n3".to_string()],
            relation: "participates_in".to_string(),
            confidence: Confidence::Inferred,
            confidence_score: 0.85,
            source_file: "src/mod.rs".to_string(),
        };
        assert_eq!(hyperedge.id, "module_a");
        assert_eq!(hyperedge.nodes.len(), 3);
        assert_eq!(hyperedge.confidence_score, 0.85);
    }

    #[test]
    fn test_hyperedge_minimum_nodes() {
        let hyperedge = Hyperedge {
            id: "small".to_string(),
            label: "Small Module".to_string(),
            nodes: vec!["n1".to_string(), "n2".to_string()],
            relation: "module".to_string(),
            confidence: Confidence::Inferred,
            confidence_score: 0.5,
            source_file: "src/mod.rs".to_string(),
        };
        assert!(hyperedge.nodes.len() >= 2);
    }
}

mod graph_data_tests {
    use super::*;

    fn create_minimal_graph() -> GraphData {
        GraphData {
            nodes: vec![
                Node::new("n1".to_string(), "node1".to_string(), "src/mod.rs".to_string(), "src/mod.rs @ L1".to_string()),
                Node::new("n2".to_string(), "node2".to_string(), "src/mod.rs".to_string(), "src/mod.rs @ L10".to_string()),
            ],
            links: vec![
                Edge::new("n1".to_string(), "n2".to_string(), "calls".to_string(), Confidence::Extracted),
            ],
            hyperedges: vec![],
            metadata: GraphMetadata::new(2, 1, 1),
        }
    }

    #[test]
    fn test_graph_metadata_creation() {
        let meta = GraphMetadata::new(10, 15, 3);
        assert_eq!(meta.total_nodes, 10);
        assert_eq!(meta.total_edges, 15);
        assert_eq!(meta.communities, 3);
    }

    #[test]
    fn test_graph_with_nodes_and_edges() {
        let graph = create_minimal_graph();
        assert_eq!(graph.nodes.len(), 2);
        assert_eq!(graph.links.len(), 1);
    }

    #[test]
    fn test_graph_with_hyperedges() {
        let mut graph = create_minimal_graph();
        graph.hyperedges.push(Hyperedge {
            id: "test".to_string(),
            label: "Test".to_string(),
            nodes: vec!["n1".to_string(), "n2".to_string()],
            relation: "module".to_string(),
            confidence: Confidence::Inferred,
            confidence_score: 0.8,
            source_file: "src/mod.rs".to_string(),
        });
        assert_eq!(graph.hyperedges.len(), 1);
    }
}

mod extraction_result_tests {
    use super::*;

    #[test]
    fn test_empty_extraction() {
        let extraction = ExtractionResult::default();
        assert!(extraction.nodes.is_empty());
        assert!(extraction.links.is_empty());
        assert!(extraction.hyperedges.is_empty());
    }

    #[test]
    fn test_extraction_with_data() {
        let extraction = ExtractionResult {
            nodes: vec![
                Node::new("f1".to_string(), "func1".to_string(), "mod.rs".to_string(), "mod.rs @ L1".to_string()),
            ],
            links: vec![
                Edge::new("f1".to_string(), "f2".to_string(), "calls".to_string(), Confidence::Extracted),
            ],
            hyperedges: vec![],
        };
        assert_eq!(extraction.nodes.len(), 1);
        assert_eq!(extraction.links.len(), 1);
    }
}

mod confidence_tests {
    use super::*;

    #[test]
    fn test_confidence_variants() {
        assert!(matches!(Confidence::Extracted, Confidence::Extracted));
        assert!(matches!(Confidence::Inferred, Confidence::Inferred));
        assert!(matches!(Confidence::Ambiguous, Confidence::Ambiguous));
    }

    #[test]
    fn test_confidence_variants_work() {
        let e = Confidence::Extracted;
        let i = Confidence::Inferred;
        let a = Confidence::Ambiguous;
        assert!(matches!(e, Confidence::Extracted));
        assert!(matches!(i, Confidence::Inferred));
        assert!(matches!(a, Confidence::Ambiguous));
    }
}

mod file_type_tests {
    use super::*;

    #[test]
    fn test_file_type_variants() {
        assert!(matches!(FileType::Code, FileType::Code));
        assert!(matches!(FileType::Markdown, FileType::Markdown));
        assert!(matches!(FileType::Binary, FileType::Binary));
        assert!(matches!(FileType::Rationale, FileType::Rationale));
    }

    #[test]
    fn test_file_type_serialization() {
        // Should serialize to lowercase
        let json = serde_json::to_string(&FileType::Code).unwrap();
        assert_eq!(json, "\"code\"");
    }
}

mod build_summary_tests {
    use super::*;

    #[test]
    fn test_build_summary_creation() {
        let summary = BuildSummary {
            total_nodes: 100,
            total_edges: 150,
            communities: 10,
            hyperedges: 5,
            changed_files: 20,
            cached_files: 80,
        };
        assert_eq!(summary.total_nodes, 100);
        assert_eq!(summary.hyperedges, 5);
    }
}

mod community_result_tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_community_result_creation() {
        let mut cohesion = HashMap::new();
        cohesion.insert(0, 0.5);
        cohesion.insert(1, 0.7);

        let mut sizes = HashMap::new();
        sizes.insert(0, 10);
        sizes.insert(1, 5);

        let result = CommunityResult {
            assignments: vec![0, 0, 0, 1, 1],
            cohesion_scores: cohesion,
            community_sizes: sizes,
        };

        assert_eq!(result.assignments.len(), 5);
        assert_eq!(result.community_sizes.get(&0), Some(&10));
    }

    #[test]
    fn test_empty_community_result() {
        let result = CommunityResult {
            assignments: vec![],
            cohesion_scores: HashMap::new(),
            community_sizes: HashMap::new(),
        };
        assert!(result.assignments.is_empty());
        assert!(result.cohesion_scores.is_empty());
    }
}
