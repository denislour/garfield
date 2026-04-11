//! Garfield CLI - Build knowledge graph from source code

use clap::Parser;

#[derive(Parser)]
#[command(name = "garfield")]
#[command(about = "Build knowledge graph from source code")]
#[command(version = "0.1.0")]
enum Cli {
    /// Build graph từ source code
    Build {
        /// Path to analyze
        #[arg(default_value = ".")]
        path: String,
        
        /// Incremental update
        #[arg(long, short)]
        update: bool,
        
        /// Output directory
        #[arg(long, default_value = "graphify-out")]
        output: String,
    },
    
    /// Query the graph
    Query {
        /// Question or terms to search
        question: String,
        
        /// Use DFS traversal instead of BFS
        #[arg(long)]
        dfs: bool,
        
        /// Traversal depth
        #[arg(long, default_value = "3")]
        depth: usize,
        
        /// Token budget
        #[arg(long, default_value = "2000")]
        budget: usize,
        
        /// Graph file path
        #[arg(long, default_value = "graphify-out/graph.json")]
        graph: String,
    },
    
    /// Find shortest path between two nodes
    Path {
        /// Source node (or label pattern)
        source: String,
        
        /// Target node (or label pattern)
        target: String,
        
        /// Max hops
        #[arg(long, default_value = "8")]
        max_hops: usize,
        
        /// Graph file path
        #[arg(long, default_value = "graphify-out/graph.json")]
        graph: String,
    },
    
    /// Explain a specific node
    Explain {
        /// Node name or pattern
        name: String,
        
        /// Graph file path
        #[arg(long, default_value = "graphify-out/graph.json")]
        graph: String,
    },
}

fn main() {
    let cli = Cli::parse();
    
    match cli {
        Cli::Build { path, update, output } => {
            println!("Garfield v{} - Building graph", env!("CARGO_PKG_VERSION"));
            println!("Path: {}", path);
            println!("Output: {}\n", output);
            
            match garfield::run_build(&path, &output, update) {
                Ok(summary) => {
                    println!("\n✅ Build complete!");
                    println!("  Nodes: {}", summary.total_nodes);
                    println!("  Edges: {}", summary.total_edges);
                    println!("  Communities: {}", summary.communities);
                    if update {
                        println!("  Changed files: {}", summary.changed_files);
                        println!("  Cached files: {}", summary.cached_files);
                    }
                }
                Err(e) => {
                    eprintln!("\n❌ Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        
        Cli::Query { question, dfs, depth, budget, graph } => {
            let mode = if dfs { "DFS" } else { "BFS" };
            println!("Query: {}", question);
            println!("Mode: {} (depth={}, budget={})\n", mode, depth, budget);
            
            match garfield::run_query(&graph, &question, dfs, depth, budget) {
                Ok(result) => {
                    println!("{}", result);
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        
        Cli::Path { source, target, max_hops, graph } => {
            println!("Finding path: {} → {} (max {} hops)\n", source, target, max_hops);
            
            match garfield::run_path(&graph, &source, &target, max_hops) {
                Ok(Some(path)) => {
                    println!("Path found ({} hops):", path.len() - 1);
                    for (i, node) in path.iter().enumerate() {
                        if i < path.len() - 1 {
                            println!("  {} →", node);
                        } else {
                            println!("  {}", node);
                        }
                    }
                }
                Ok(None) => {
                    println!("No path found between {} and {}", source, target);
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        
        Cli::Explain { name, graph } => {
            println!("Explaining: {}\n", name);
            
            match garfield::from_json(std::path::Path::new(&graph)) {
                Ok(g) => {
                    // Find node
                    let node = g.nodes.iter().find(|n| 
                        n.label.to_lowercase().contains(&name.to_lowercase()) ||
                        n.id.to_lowercase().contains(&name.to_lowercase())
                    );
                    
                    if let Some(node) = node {
                        println!("NODE: {}", node.label);
                        println!("  ID: {}", node.id);
                        println!("  File: {}", node.source_file);
                        println!("  Location: {}", node.source_location);
                        if let Some(c) = node.community {
                            println!("  Community: {}", c);
                        }
                        
                        // Find connections
                        let connections: Vec<_> = g.edges.iter()
                            .filter(|e| e.source == node.id || e.target == node.id)
                            .collect();
                        
                        if !connections.is_empty() {
                            println!("\n  Connections ({}):", connections.len());
                            for conn in connections.iter().take(10) {
                                let other = if conn.source == node.id { &conn.target } else { &conn.source };
                                let other_label = g.nodes.iter()
                                    .find(|n| &n.id == other)
                                    .map(|n| n.label.as_str())
                                    .unwrap_or(other);
                                
                                println!("    - {} ({})", other_label, conn.relation);
                            }
                        }
                    } else {
                        println!("Node not found: {}", name);
                    }
                }
                Err(e) => {
                    eprintln!("Error loading graph: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }
}
