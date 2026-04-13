//! Garfield - Build knowledge graph from source code
//!
//! A Rust port of graphify for code extraction only (no LLM required).

pub mod analyze;
pub mod build;
pub mod cache;
pub mod community;
pub mod detect;
pub mod export;
pub mod extract;
pub mod hyperedge;
pub mod lang;
pub mod leiden;
pub mod report;
pub mod serve;
pub mod types;
pub mod validate;

// Re-export commonly used types
pub use analyze::{
    analyze, find_god_nodes, find_surprising_connections, graph_diff, suggest_questions, Analysis,
    EdgeChange, GraphDiff, NodeChange, SuggestedQuestion,
};
pub use build::{build_graph, merge_extractions, merge_into_graph};
pub use cache::{
    check_cache, clear_all_cache, compute_hash, load_cached, save_cached, update_cache, FileCache,
};
pub use community::{add_communities, cluster, split_oversized};

pub use detect::{
    detect, estimate_word_count, filter_code_files, get_stats, print_summary, DetectResult,
};
pub use export::{export_stats, from_json, to_json};
pub use extract::{extract_file, extract_files};
pub use hyperedge::detect_hyperedges;
pub use lang::{get_extension_lang, get_ts_language, LangConfig, LANG_CONFIGS};
pub use report::{generate_report, print_report, DetectInfo, DiffInfo};
pub use serve::{
    find_shortest_path, format_graph_stats, get_community, get_hyperedge, get_neighbors, get_node,
    get_node_body, graph_stats, query, query_with_filters, score_nodes, CommunityInfo, GraphStats,
    HyperedgeInfo, NodeDetails,
};
pub use types::{
    BuildSummary, CommunityResult, Confidence, DetectStats, DetectedFile, Edge, ExtractionResult,
    FileType, GraphData, GraphMetadata, Hyperedge, Node,
};
pub use validate::{validate_extraction, validate_graph};

use std::path::Path;

/// Build graph from source path
pub fn run_build(root: &str, output: &str, update: bool) -> anyhow::Result<BuildSummary> {
    let root_path = Path::new(root);
    let output_path = Path::new(output);

    // 1. Detect files
    println!("Detecting files...");
    let detect_result = detect(root_path)?;
    let code_files = filter_code_files(&detect_result.files);

    print_summary(&detect_result.files);

    if code_files.is_empty() {
        return Ok(BuildSummary {
            total_nodes: 0,
            total_edges: 0,
            communities: 0,
            hyperedges: 0,
            changed_files: 0,
            cached_files: 0,
        });
    }

    // Calculate word count for report
    let total_words = estimate_word_count(&detect_result.files);
    let detect_info = DetectInfo {
        total_files: detect_result.files.len(),
        total_words,
        warning: None,
    };

    // 2. Load cache
    let cache_path = output_path.join("cache.json");
    let mut cache = cache::FileCache::load(&cache_path).unwrap_or_default();

    // 3. Check cache
    let paths: Vec<_> = code_files.iter().map(|f| f.path.clone()).collect();
    let (changed, cached) = if update {
        cache::check_cache(&paths, &cache)
    } else {
        (paths.clone(), vec![])
    };

    println!(
        "Cache: {} changed, {} unchanged",
        changed.len(),
        cached.len()
    );

    // 4. Build or update graph
    let graph_path = output_path.join("graph.json");

    let graph = if update && graph_path.exists() {
        // UPDATE MODE: Load existing graph and merge
        println!("Loading existing graph...");
        let mut existing = from_json(&graph_path)?;

        // Extract only changed files
        println!("Extracting from {} changed files...", changed.len());
        let ast_extractions = extract_files(&changed);

        // Merge AST extraction into existing graph
        for extraction in ast_extractions {
            merge_into_graph(&mut existing, extraction);
        }

        // Re-cluster to account for new nodes
        println!("Re-clustering...");
        let community_result = cluster(&existing);
        add_communities(&mut existing, &community_result.assignments);
        split_oversized(&mut existing, 25);

        existing
    } else {
        // FULL REBUILD: Extract all files and build fresh
        println!("Extracting from {} files...", changed.len());
        let ast_extractions = extract_files(&changed);

        let all_extractions = ast_extractions;

        println!("Building graph...");
        build_graph(all_extractions)
    };

    // 5. Detect hyperedges
    println!("Detecting hyperedges...");
    let hyperedges = detect_hyperedges(&graph);
    println!("Found {} hyperedges", hyperedges.len());

    // Add hyperedges to graph
    let mut graph_with_hyperedges = graph;
    for he in hyperedges {
        graph_with_hyperedges.hyperedges.push(he);
    }

    // 6. Validate
    if let Err(e) = validate_graph(&graph_with_hyperedges) {
        eprintln!("Warning: Validation error: {:?}", e);
    }

    // 7. Export JSON
    to_json(&graph_with_hyperedges, &graph_path)?;
    let report_path = output_path.join("GRAPH_REPORT.md");
    generate_report(
        &graph_with_hyperedges,
        &report_path,
        Some(detect_info),
        None,
    )?;

    // 8. Update cache
    if update {
        cache::update_cache(&mut cache, &changed, None)?;
        cache.save(&cache_path)?;
    }

    Ok(BuildSummary {
        total_nodes: graph_with_hyperedges.metadata.total_nodes,
        total_edges: graph_with_hyperedges.metadata.total_edges,
        communities: graph_with_hyperedges.metadata.communities,
        hyperedges: graph_with_hyperedges.hyperedges.len(),
        changed_files: changed.len(),
        cached_files: cached.len(),
    })
}

/// Query the graph
pub fn run_query(
    graph_path: &str,
    question: &str,
    use_dfs: bool,
    depth: usize,
    budget: usize,
) -> anyhow::Result<String> {
    let graph = from_json(Path::new(graph_path))?;
    Ok(serve::query(&graph, question, use_dfs, depth, budget))
}

/// Query with filters
pub fn run_query_with_filters(
    graph_path: &str,
    question: &str,
    use_dfs: bool,
    depth: usize,
    budget: usize,
    node_type: Option<&str>,
    community: Option<u32>,
    source: Option<&str>,
    hyperedge: Option<&str>,
) -> anyhow::Result<String> {
    let graph = from_json(Path::new(graph_path))?;
    Ok(serve::query_with_filters(
        &graph, question, use_dfs, depth, budget, node_type, community, source, hyperedge,
    ))
}

/// Find path between nodes
pub fn run_path(
    graph_path: &str,
    source: &str,
    target: &str,
    max_hops: usize,
) -> anyhow::Result<Option<Vec<String>>> {
    let graph = from_json(Path::new(graph_path))?;
    Ok(serve::find_shortest_path(&graph, source, target, max_hops))
}

/// Explain a node - show node details + hyperedge (module) info
pub fn run_explain(graph_path: &str, identifier: &str) -> anyhow::Result<String> {
    let graph = from_json(Path::new(graph_path))?;

    // Get node details (includes hyperedge)
    let details = serve::get_node(&graph, identifier);

    if let Some(details) = details {
        let mut output = String::new();

        output.push_str("═══ NODE ═══\n");
        output.push_str(&format!("ID: {}\n", details.id));
        output.push_str(&format!("Label: {}\n", details.label));
        output.push_str(&format!("File: {}\n", details.source_file));
        output.push_str(&format!("Location: {}\n", details.source_location));

        if let Some(he) = details.hyperedge {
            output.push_str("\n═══ MODULE (Hyperedge) ═══\n");
            output.push_str(&format!("Module: {}\n", he.label));
            output.push_str(&format!("Members: {} functions\n", he.member_count));
            output.push_str(&format!("Confidence: {:.2}\n", he.confidence_score));
        }

        if !details.outgoing_edges.is_empty() {
            output.push_str("\n═══ CALLS ═══\n");
            for edge in details.outgoing_edges.iter().take(5) {
                output.push_str(&format!("  → {} ({})\n", edge.target_label, edge.relation));
            }
        }

        if !details.incoming_edges.is_empty() {
            output.push_str("\n═══ CALLED BY ═══\n");
            for edge in details.incoming_edges.iter().take(5) {
                output.push_str(&format!("  ← {} ({})\n", edge.source_label, edge.relation));
            }
        }

        Ok(output)
    } else {
        Ok(format!("Node '{}' not found", identifier))
    }
}
