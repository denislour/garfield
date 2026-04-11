//! Garfield - Build knowledge graph from source code
//! 
//! A Rust port of graphify for code extraction only (no LLM required).

pub mod types;
pub mod detect;
pub mod extract;
pub mod build;
pub mod cluster;
pub mod analyze;
pub mod report;
pub mod cache;
pub mod validate;
pub mod export;
pub mod serve;

// Re-export commonly used types
pub use types::{
    Node, Edge, Confidence, GraphData, GraphMetadata,
    ExtractionResult, CommunityResult, BuildSummary,
    DetectedFile, DetectStats, FileType,
};
pub use detect::{detect, filter_code_files, get_stats, print_summary};
pub use extract::{extract_file, extract_files};
pub use build::{build_graph, merge_extractions};
pub use cluster::{cluster, add_communities, split_oversized};
pub use analyze::{analyze, find_god_nodes, find_surprising_connections, Analysis};
pub use report::{generate_report, print_report};
pub use cache::{FileCache, check_cache, update_cache, compute_hash};
pub use validate::{validate_graph, validate_extraction};
pub use export::{to_json, from_json, export_stats};
pub use serve::{query, find_shortest_path, score_nodes};

use std::path::Path;

/// Build graph from source path
pub fn run_build(root: &str, output: &str, update: bool) -> anyhow::Result<BuildSummary> {
    use std::path::Path;
    
    let root_path = Path::new(root);
    let output_path = Path::new(output);
    
    // 1. Detect files
    println!("Detecting files...");
    let files = detect(root_path)?;
    let code_files = filter_code_files(&files);
    
    print_summary(&files);
    
    if code_files.is_empty() {
        return Ok(BuildSummary {
            total_nodes: 0,
            total_edges: 0,
            communities: 0,
            changed_files: 0,
            cached_files: 0,
        });
    }
    
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
    
    println!("Cache: {} changed, {} unchanged", changed.len(), cached.len());
    
    // 4. Extract changed files
    println!("Extracting from {} files...", changed.len());
    let extractions = extract_files(&changed);
    
    // 5. Build graph
    println!("Building graph...");
    let graph = build_graph(extractions);
    
    // 6. Validate
    if let Err(e) = validate_graph(&graph) {
        eprintln!("Warning: Validation error: {:?}", e);
    }
    
    // 7. Export JSON
    let graph_path = output_path.join("graph.json");
    to_json(&graph, &graph_path)?;
    
    // 8. Generate report
    let report_path = output_path.join("GRAPH_REPORT.md");
    generate_report(&graph, &report_path)?;
    
    // 9. Update cache
    if update {
        cache::update_cache(&mut cache, &changed)?;
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
pub fn run_query(graph_path: &str, question: &str, use_dfs: bool, depth: usize, budget: usize) -> anyhow::Result<String> {
    let graph = from_json(Path::new(graph_path))?;
    Ok(serve::query(&graph, question, use_dfs, depth, budget))
}

/// Find path between nodes
pub fn run_path(graph_path: &str, source: &str, target: &str, max_hops: usize) -> anyhow::Result<Option<Vec<String>>> {
    let graph = from_json(Path::new(graph_path))?;
    Ok(serve::find_shortest_path(&graph, source, target, max_hops))
}
