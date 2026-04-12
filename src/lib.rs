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
pub mod lang;
pub mod leiden;
pub mod report;
pub mod serve;
pub mod types;
pub mod validate;

// Re-export commonly used types
pub use analyze::{
    analyze, find_god_nodes, find_surprising_connections, suggest_questions, graph_diff,
    Analysis, GraphDiff, NodeChange, EdgeChange, SuggestedQuestion,
};
pub use build::{build_graph, merge_extractions, merge_into_graph};
pub use cache::{
    check_cache, compute_hash, update_cache, FileCache, 
    load_cached, save_cached, clear_all_cache,
};
pub use community::{add_communities, cluster, split_oversized};

pub use detect::{detect, estimate_word_count, filter_code_files, get_stats, print_summary, DetectResult};
pub use export::{export_stats, from_json, to_json};
pub use extract::{extract_file, extract_files};
pub use report::{generate_report, print_report, DetectInfo, DiffInfo};
pub use serve::{
    find_shortest_path, query, score_nodes, get_node, get_neighbors, get_community, 
    graph_stats, format_graph_stats, get_node_body, NodeDetails, CommunityInfo, GraphStats,
};
pub use lang::{LangConfig, LANG_CONFIGS, get_ts_language, get_extension_lang};
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

    // 5. Validate
    if let Err(e) = validate_graph(&graph) {
        eprintln!("Warning: Validation error: {:?}", e);
    }

    // 6. Export JSON
    to_json(&graph, &graph_path)?;
    let report_path = output_path.join("GRAPH_REPORT.md");
    generate_report(&graph, &report_path, Some(detect_info), None)?;

    // 8. Update cache
    if update {
        cache::update_cache(&mut cache, &changed, None)?;
        cache.save(&cache_path)?;
    }

    Ok(BuildSummary {
        total_nodes: graph.metadata.total_nodes,
        total_edges: graph.metadata.total_edges,
        communities: graph.metadata.communities,
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
