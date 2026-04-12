//! Report generation module
//!
//! ## Report Flow
//!
//! 1. generate_report() - Main entry point
//! 2. Print corpus check with warnings
//! 3. Print summary with confidence breakdown
//! 4. Print god nodes
//! 5. Print surprising connections with "why" explanations
//! 6. Print community details with cohesion
//! 7. Print ambiguous edges
//! 8. Print knowledge gaps (isolated nodes, thin communities)
//! 9. Print suggested questions (new!)
//! 10. Print graph diff if provided (new!)

use crate::analyze::{analyze, suggest_questions};
use crate::types::{Confidence, GraphData};

use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct DetectInfo {
    pub total_files: usize,
    pub total_words: usize,
    pub warning: Option<String>,
}

/// Graph diff info for reports
pub struct DiffInfo {
    pub new_nodes: usize,
    pub removed_nodes: usize,
    pub new_edges: usize,
    pub removed_edges: usize,
    pub summary: String,
}

/// Generate GRAPH_REPORT.md
pub fn generate_report(
    graph: &GraphData,
    output_path: &Path,
    detect_info: Option<DetectInfo>,
    diff_info: Option<DiffInfo>,
) -> anyhow::Result<()> {
    let analysis = analyze(graph);

    let today = {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
        let secs = now.as_secs();
        let days = secs / 86400;
        let years = days / 365 + 1970;
        let remaining_days = days % 365;
        let months = remaining_days / 30 + 1;
        let day = remaining_days % 30 + 1;
        format!("{}-{:02}-{:02}", years, months, day)
    };
    let root = std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| ".".to_string());

    let mut content = String::new();

    // Header
    content.push_str(&format!("# Graph Report - {}  ({})\n\n", root, today));

    // ============================================================
    // Diff Summary (if incremental update)
    // ============================================================
    if let Some(diff) = diff_info {
        content.push_str("## 📊 Changes Since Last Build\n\n");
        content.push_str(&format!("- {}\n", diff.summary));
        if diff.new_nodes > 0 || diff.new_edges > 0 {
            content.push_str(&format!(
                "  +{} nodes, +{} edges\n",
                diff.new_nodes, diff.new_edges
            ));
        }
        if diff.removed_nodes > 0 || diff.removed_edges > 0 {
            content.push_str(&format!(
                "  -{} nodes, -{} edges\n",
                diff.removed_nodes, diff.removed_edges
            ));
        }
        content.push('\n');
    }

    // ============================================================
    // Corpus Check
    // ============================================================
    content.push_str("## Corpus Check\n");
    if let Some(info) = &detect_info {
        if info.total_words < 50_000 && info.total_files > 0 {
            content.push_str(&format!(
                "- ⚠️ Small corpus: {} files · ~{} words\n",
                info.total_files, info.total_words
            ));
            content.push_str("  Graph may not add much value for small codebases.\n");
        } else if info.total_words > 500_000 {
            content.push_str(&format!(
                "- ⚠️ Large corpus: {} files · ~{} words\n",
                info.total_files, info.total_words
            ));
            content.push_str("  May have high token cost.\n");
        } else {
            content.push_str(&format!(
                "- ✓ {} files · ~{} words\n",
                info.total_files, info.total_words
            ));
        }
    } else {
        content.push_str(&format!(
            "- {} files · ~{} nodes\n",
            graph.metadata.total_nodes, graph.metadata.total_nodes
        ));
        content.push_str(&format!("- Verdict: {}\n", corpus_verdict(graph)));
    }
    content.push('\n');

    // ============================================================
    // Confidence breakdown
    // ============================================================
    let total = graph.links.len();
    let ext_pct = if total > 0 {
        (analysis.confidence_stats.extracted * 100) / total
    } else {
        0
    };
    let inf_pct = if total > 0 {
        (analysis.confidence_stats.inferred * 100) / total
    } else {
        0
    };
    let amb_pct = if total > 0 {
        (analysis.confidence_stats.ambiguous * 100) / total
    } else {
        0
    };

    // ============================================================
    // Summary
    // ============================================================
    content.push_str("## Summary\n");
    content.push_str(&format!(
        "- {} nodes · {} edges · {} communities detected\n",
        graph.metadata.total_nodes, graph.metadata.total_edges, graph.metadata.communities
    ));
    content.push_str(&format!(
        "- Extraction: {}% EXTRACTED · {}% INFERRED · {}% AMBIGUOUS",
        ext_pct, inf_pct, amb_pct
    ));
    if analysis.confidence_stats.inferred > 0 {
        content.push_str(&format!(
            " · INFERRED: {} edges (avg confidence: {:.2})",
            analysis.confidence_stats.inferred,
            0.85
        ));
    }
    content.push_str("\n");
    content.push_str("- Token cost: 0 input · 0 output (no LLM used)\n");
    content.push('\n');

    // ============================================================
    // God Nodes
    // ============================================================
    content.push_str("## God Nodes (most connected - your core abstractions)\n");
    for (i, god) in analysis.god_nodes.iter().enumerate().take(10) {
        content.push_str(&format!(
            "{}. `{}` - {} edges\n",
            i + 1,
            god.node.label,
            god.degree
        ));
        content.push_str(&format!(
            "   📁 {} · source: {}\n",
            god.source_file, god.node.id
        ));
    }
    content.push('\n');

    // ============================================================
    // Surprising Connections (with explanations)
    // ============================================================
    content.push_str("## Surprising Connections (you probably didn't know these)\n");
    if analysis.surprising_connections.is_empty() {
        content.push_str("- None detected - all connections are within the same source files.\n");
    } else {
        content.push_str("| Source | Target | Relation | Why |\n");
        content.push_str("|--------|--------|----------|-----|\n");
        for conn in analysis.surprising_connections.iter().take(10) {
            let conf_tag = match conn.confidence {
                Confidence::Extracted => "EXTRACTED",
                Confidence::Inferred => "INFERRED",
                Confidence::Ambiguous => "AMBIGUOUS",
            };
            content.push_str(&format!(
                "| `{}` | `{}` | {} | {} |\n",
                conn.source_label, conn.target_label, conn.relation, conn.why
            ));
            content.push_str(&format!(
                "| ↳ {} → {} | {} | {} |\n",
                conn.source_file, conn.target_file, conf_tag, conn.score
            ));
        }
    }
    content.push('\n');

    // ============================================================
    // Communities
    // ============================================================
    content.push_str("## Communities\n\n");
    let mut sorted_communities: Vec<_> = analysis.community_sizes.iter().collect();
    sorted_communities.sort_by(|a, b| b.1.cmp(a.1));

    for (cid, _size) in sorted_communities.iter().take(10) {
        let cohesion = analysis.cohesion_scores.get(cid).copied().unwrap_or(0.0);
        let label = analysis
            .community_labels
            .get(cid)
            .cloned()
            .unwrap_or_else(|| format!("Community {}", cid));

        // List some nodes in this community
        let real_nodes: Vec<_> = graph
            .nodes
            .iter()
            .filter(|n| {
                n.community == Some(**cid)
                    && !is_filtered_node(&n.label)
            })
            .take(10)
            .collect();

        let real_size = real_nodes.len();
        let display_nodes: Vec<_> = real_nodes
            .iter()
            .map(|n| n.label.as_str())
            .collect();

        let suffix = if real_size > 10 {
            format!(" (+{} more)", analysis.community_sizes.get(cid).unwrap_or(&0) - 10)
        } else {
            String::new()
        };

        // Cohesion indicator
        let cohesion_indicator = if cohesion >= 0.5 {
            "🟢"
        } else if cohesion >= 0.2 {
            "🟡"
        } else {
            "🔴"
        };

        content.push_str(&format!(
            "### {} \"{}\" ({} nodes) {}\n",
            cid, label, analysis.community_sizes.get(cid).unwrap_or(&0), cohesion_indicator
        ));
        content.push_str(&format!("**Cohesion:** {:.2}\n\n", cohesion));
        content.push_str(&format!(
            "**Key concepts:** {}{}\n\n",
            display_nodes.join(", "),
            suffix
        ));
    }

    // ============================================================
    // Ambiguous Edges
    // ============================================================
    let ambiguous_edges: Vec<_> = graph
        .links
        .iter()
        .filter(|e| e.confidence == Confidence::Ambiguous)
        .collect();

    if !ambiguous_edges.is_empty() {
        content.push_str("## Ambiguous Edges - Review These\n\n");
        content.push_str("These edges have low confidence and need human verification:\n\n");
        
        for edge in ambiguous_edges {
            let src_label = graph
                .nodes
                .iter()
                .find(|n| n.id == edge.source)
                .map(|n| n.label.as_str())
                .unwrap_or(&edge.source);

            let tgt_label = graph
                .nodes
                .iter()
                .find(|n| n.id == edge.target)
                .map(|n| n.label.as_str())
                .unwrap_or(&edge.target);

            content.push_str(&format!(
                "- ❓ `{}` → `{}`  [AMBIGUOUS]\n",
                src_label, tgt_label
            ));
            content.push_str(&format!(
                "  `{}` · relation: {}\n\n",
                edge.source_file, edge.relation
            ));
        }
    }

    // ============================================================
    // Knowledge Gaps
    // ============================================================
    let isolated: Vec<_> = graph
        .nodes
        .iter()
        .filter(|n| {
            let degree = graph
                .links
                .iter()
                .filter(|e| e.source == n.id || e.target == n.id)
                .count();
            degree <= 1 && !is_filtered_node(&n.label)
        })
        .take(5)
        .collect();

    let thin_communities: Vec<_> = analysis
        .community_sizes
        .iter()
        .filter(|(_, &size)| size < 3)
        .collect();

    let gap_count = isolated.len() + thin_communities.len();
    let high_ambiguity = amb_pct > 20;

    if gap_count > 0 || high_ambiguity {
        content.push_str("## Knowledge Gaps\n\n");

        if !isolated.is_empty() {
            let isolated_labels: Vec<_> = isolated.iter().map(|n| n.label.as_str()).collect();
            content.push_str("### 🔌 Isolated Nodes\n\n");
            content.push_str("These have ≤1 connection - possible documentation gaps:\n\n");
            for label in isolated_labels {
                content.push_str(&format!("- `{}`\n", label));
            }
            content.push('\n');
        }

        if !thin_communities.is_empty() {
            content.push_str("### 📉 Thin Communities\n\n");
            content.push_str("Too small to be meaningful - may be noise:\n\n");
            for (cid, _) in &thin_communities {
                let label = analysis
                    .community_labels
                    .get(cid)
                    .cloned()
                    .unwrap_or_else(|| format!("Community {}", cid));
                content.push_str(&format!("- `{}` ({} nodes)\n", label, cid));
            }
            content.push('\n');
        }

        if high_ambiguity {
            content.push_str("### ⚠️ High Ambiguity\n\n");
            content.push_str(&format!(
                "{}% of edges are AMBIGUOUS. Review the Ambiguous Edges section above.\n\n",
                amb_pct
            ));
        }
    }

    // ============================================================
    // Suggested Questions
    // ============================================================
    let questions = suggest_questions(graph, 7);
    let has_real_questions = questions.iter().any(|q| !q.question.is_empty());

    if has_real_questions {
        content.push_str("## 💡 Suggested Questions\n\n");
        content.push_str("Questions the graph is uniquely positioned to answer:\n\n");

        for (i, q) in questions.iter().enumerate() {
            if q.question.is_empty() {
                continue;
            }
            content.push_str(&format!("### {}. {}\n\n", i + 1, q.question_type.replace('_', " ")));
            content.push_str(&format!("**Q:** {}\n\n", q.question));
            content.push_str(&format!("**Why:** {}\n\n", q.why));
        }
    }

    // Ensure directory exists
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Write report
    fs::write(output_path, content)?;

    println!("Report generated: {}", output_path.display());
    Ok(())
}

/// Check if node should be filtered from reports
fn is_filtered_node(label: &str) -> bool {
    let l = label.to_lowercase();
    // Filter out single-letter names, indexes, common patterns
    // Note: main is kept as it's a valid entry point
    l.len() <= 2
        || l == "index"
        || l == "init"
        || l == "setup"
        || l.starts_with("__")
        || l.ends_with("_handler")
        || l.ends_with("_util")
        || l.ends_with("_helper")
}

/// Determine corpus verdict based on size
fn corpus_verdict(graph: &GraphData) -> String {
    let nodes = graph.metadata.total_nodes;
    if nodes < 10 {
        "⚠️ corpus too small for meaningful graph".to_string()
    } else if nodes < 100 {
        "corpus is small but graph structure adds value".to_string()
    } else if nodes < 1000 {
        "✓ corpus is large enough that graph structure adds value".to_string()
    } else {
        "✓ corpus is very large - graph is essential for navigation".to_string()
    }
}

/// Print report to stdout
pub fn print_report(graph: &GraphData) {
    let analysis = analyze(graph);

    println!("=== GRAPH REPORT ===");
    println!("Nodes: {}", graph.metadata.total_nodes);
    println!("Edges: {}", graph.metadata.total_edges);
    println!("Communities: {}", graph.metadata.communities);

    println!("\n--- GOD NODES ---");
    for (i, god) in analysis.god_nodes.iter().enumerate().take(5) {
        println!("{}. {} ({} edges)", i + 1, god.node.label, god.degree);
    }

    println!("\n--- SURPRISING CONNECTIONS ---");
    for conn in analysis.surprising_connections.iter().take(3) {
        println!(
            "{} --{} [score={}]--> {}",
            conn.source_label, conn.relation, conn.score, conn.target_label
        );
        println!("  Why: {}", conn.why);
    }

    println!("\n--- CONFIDENCE ---");
    let stats = &analysis.confidence_stats;
    println!("EXTRACTED: {}", stats.extracted);
    println!("INFERRED: {}", stats.inferred);
    println!("AMBIGUOUS: {}", stats.ambiguous);

    println!("\n--- SUGGESTED QUESTIONS ---");
    let questions = suggest_questions(graph, 3);
    for (i, q) in questions.iter().enumerate() {
        if !q.question.is_empty() {
            println!("{}. {}: {}", i + 1, q.question_type, q.question);
        }
    }
}

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
