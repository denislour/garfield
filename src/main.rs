//! Garfield CLI - Build knowledge graph from source code

use clap::Parser;

#[derive(Parser)]
#[command(name = "gf")]
#[command(aliases = ["garfield"])]
#[command(long_about = "Build knowledge graph from source code\n\nGarfield is a simplified Rust version of graphify, focusing on code extraction only.")]
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
    
    /// Install PI extension and skill
    Install {
        /// Target: pi, all (default: all)
        #[arg(default_value = "all")]
        target: String,
        
        /// Use symlink instead of copy
        #[arg(long, short = 's')]
        symlink: bool,
        
        /// Force overwrite existing files
        #[arg(long, short = 'f')]
        force: bool,
    },
    
    /// Uninstall PI integration
    Uninstall {
        /// Target: pi, all (default: all)
        #[arg(default_value = "all")]
        target: String,
    },
    
    /// Install configuration files for various agents (Claude, Cursor, etc.)
    Agents {
        /// Agent type: claude, cursor, all (default: all)
        #[arg(default_value = "all")]
        agent: String,
        
        /// Force overwrite existing files
        #[arg(long, short = 'f')]
        force: bool,
    },
    
    /// Start MCP stdio server for agent integration
    Serve {
        /// Graph file path (default: graphify-out/graph.json)
        #[arg(default_value = "graphify-out/graph.json")]
        graph: String,
    },
}

fn main() {
    let cli = Cli::parse();
    
    match cli {
        Cli::Build { path, update, output } => {
            println!("gf v{} - Building graph", env!("CARGO_PKG_VERSION"));
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
        
        Cli::Install { target, symlink, force } => {
            install_pi(&target, symlink, force);
        }
        
        Cli::Uninstall { target } => {
            uninstall_pi(&target);
        }
        
        Cli::Agents { agent, force } => {
            install_agents(&agent, force);
        }
        
        Cli::Serve { graph } => {
            println!("gf serve - Starting MCP stdio server...");
            println!("Graph: {}", graph);
            println!("\nMCP server running. Press Ctrl+C to stop.");
            
            // TODO: Implement actual MCP server
            // For now, just show info
            println!("\n⚠️  MCP server not yet implemented.");
            println!("Use 'gf build' to build the graph, then query with 'gf query'.");
        }
    }
}

/// Install PI extension and skill
fn install_pi(target: &str, _use_symlink: bool, force: bool) {
    use std::fs;
    
    println!("gf install - Setting up PI integration\n");
    
    // Get home directory
    let home = dirs::home_dir().expect("Could not find home directory");
    
    // Get current executable path
    let current_exe = std::env::current_exe().expect("Could not get current executable path");
    let exe_path = current_exe.to_string_lossy().to_string();
    
    // Check if we should install extension
    let install_ext = target == "all" || target == "pi" || target == "extension";
    let install_skill = target == "all" || target == "pi" || target == "skill";
    
    if install_ext {
        let ext_dir = home.join(".pi/agent/extensions/garfield");
        let ext_file = ext_dir.join("index.ts");
        
        println!("📦 Installing PI Extension...");
        println!("  Path: {}", ext_dir.display());
        
        // Create directory
        if !ext_dir.exists() {
            fs::create_dir_all(&ext_dir).expect("Failed to create extension directory");
        }
        
        // Generate extension content
        let ext_content = generate_extension_ts(&exe_path);
        
        // Check if file exists
        if ext_file.exists() && !force {
            println!("  ⚠️  Extension already exists (use -f to overwrite)");
        } else {
            fs::write(&ext_file, ext_content).expect("Failed to write extension file");
            println!("  ✅ Extension installed!");
        }
    }
    
    if install_skill {
        let skill_dir = home.join(".pi/agent/skills/garfield");
        let skill_file = skill_dir.join("SKILL.md");
        
        println!("\n📚 Installing PI Skill...");
        println!("  Path: {}", skill_dir.display());
        
        // Create directory
        if !skill_dir.exists() {
            fs::create_dir_all(&skill_dir).expect("Failed to create skill directory");
        }
        
        // Generate skill content
        let skill_content = generate_skill_md();
        
        if skill_file.exists() && !force {
            println!("  ⚠️  Skill already exists (use -f to overwrite)");
        } else {
            fs::write(&skill_file, skill_content).expect("Failed to write skill file");
            println!("  ✅ Skill installed!");
        }
    }
    
    println!("\n✨ Installation complete!");
    println!("\nNext steps:");
    println!("  1. Start PI: pi");
    println!("  2. Type /reload to load the extension");
    println!("  3. Try: /gf help");
}

/// Uninstall PI integration
fn uninstall_pi(target: &str) {
    use std::fs;
    
    println!("gf uninstall - Removing PI integration\n");
    
    let home = dirs::home_dir().expect("Could not find home directory");
    
    let uninstall_ext = target == "all" || target == "pi" || target == "extension";
    let uninstall_skill = target == "all" || target == "pi" || target == "skill";
    
    if uninstall_ext {
        let ext_dir = home.join(".pi/agent/extensions/garfield");
        if ext_dir.exists() {
            fs::remove_dir_all(&ext_dir).expect("Failed to remove extension directory");
            println!("🗑️  Removed extension: {}", ext_dir.display());
        } else {
            println!("⚠️  Extension not found: {}", ext_dir.display());
        }
    }
    
    if uninstall_skill {
        let skill_dir = home.join(".pi/agent/skills/garfield");
        if skill_dir.exists() {
            fs::remove_dir_all(&skill_dir).expect("Failed to remove skill directory");
            println!("🗑️  Removed skill: {}", skill_dir.display());
        } else {
            println!("⚠️  Skill not found: {}", skill_dir.display());
        }
    }
    
    println!("\n✨ Uninstallation complete!");
}

/// Generate TypeScript extension content
fn generate_extension_ts(_exe_path: &str) -> String {
    format!(r#"""
/**
 * Garfield PI Extension
 * 
 * Auto-generated by gf install
 * Garfield is a Rust-based code extraction tool.
 * 
 * Usage:
 *   /gf build <path>       - Build knowledge graph
 *   /gf query <question>   - Query the graph
 *   /gf path <A> <B>       - Find path between nodes
 *   /gf explain <name>     - Explain a node
 */

import type {{ ExtensionAPI }} from "@mariozechner/pi-coding-agent";
import {{ Type }} from "@sinclair/typebox";
import {{ existsSync, readFileSync }} from "node:fs";

interface GarfieldGraph {{
    nodes: Array<{{
        id: string;
        label: string;
        source_file: string;
        source_location?: string;
        community?: number;
    }}>;
    edges: Array<{{
        source: string;
        target: string;
        relation: string;
        confidence: string;
    }}>;
    metadata?: {{
        total_nodes: number;
        total_edges: number;
        communities: number;
    }};
}}

// Garfield binary path (auto-detected)
const GF_BINARY = "gf";

function loadGraph(graphPath: string = "graphify-out/graph.json"): GarfieldGraph | null {{
    try {{
        if (!existsSync(graphPath)) return null;
        return JSON.parse(readFileSync(graphPath, "utf-8"));
    }} catch {{
        return null;
    }}
}}

export default function garfieldExtension(pi: ExtensionAPI) {{
    pi.on("session_start", async (_event, ctx) => {{
        ctx.ui.notify("Garfield extension loaded", "success");
    }});

    pi.registerCommand("gf", {{
        description: "Garfield: build, query, path, explain, report",
        handler: async (args, ctx) => {{
            const parts = args.trim().split(/\s+/);
            const cmd = parts[0] || "help";
            
            switch (cmd) {{
                case "help":
                    ctx.ui.notify("gf commands:\n/build, /query, /path, /explain, /report", "info");
                    break;
                case "report": {{
                    const report = loadGraph();
                    if (report) {{
                        ctx.ui.notify(`Graph: ${{report.metadata?.total_nodes || 0}} nodes, ${{
eport.metadata?.total_edges || 0}} edges`, "info");
                    }} else {{
                        ctx.ui.notify("No graph found. Run /gf build first.", "warning");
                    }}
                    break;
                }}
                default:
                    ctx.ui.notify("Run /gf help for available commands", "info");
            }}
        }},
    }});

    // gf_build tool
    pi.registerTool({{
        name: "gf_build",
        label: "Garfield Build",
        description: "Build Garfield knowledge graph from source code",
        parameters: Type.Object({{
            path: Type.Optional(Type.String()),
            update: Type.Optional(Type.Boolean()),
        }}),
        async execute(toolCallId, params) {{
            const graph = loadGraph();
            if (graph) {{
                return {{
                    content: [{{ type: "text", text: `Graph already exists: ${{graph.metadata?.total_nodes}} nodes` }}],
                    details: graph.metadata,
                }};
            }}
            return {{
                content: [{{ type: "text", text: "Run 'gf build <path>' in terminal first" }}],
                details: {{ error: "no_graph" }},
            }};
        }},
    }});

    // gf_graph_query tool
    pi.registerTool({{
        name: "gf_graph_query",
        label: "Garfield Query",
        description: "Query Garfield knowledge graph for code relationships",
        parameters: Type.Object({{
            question: Type.String(),
            mode: Type.Optional(Type.Union([Type.Literal("bfs"), Type.Literal("dfs")])),
            depth: Type.Optional(Type.Number()),
        }}),
        async execute(toolCallId, params) {{
            const graph = loadGraph();
            if (!graph) {{
                return {{
                    content: [{{ type: "text", text: "No graph found. Run 'gf build' first." }}],
                    details: {{ error: "no_graph" }},
                }};
            }}
            
            const query = params.question.toLowerCase();
            const matchingNodes = graph.nodes.filter(n => 
                n.label.toLowerCase().includes(query) ||
                n.id.toLowerCase().includes(query)
            );
            
            return {{
                content: [{{ 
                    type: "text", 
                    text: matchingNodes.length > 0 
                        ? `Found ${{matchingNodes.length}} matching nodes:\n${{matchingNodes.slice(0, 5).map(n => `- ${{n.label}} (${{n.source_file}})`).join('\n')}}`
                        : "No matching nodes found"
                }}],
                details: {{ matches: matchingNodes.length }},
            }};
        }},
    }});

    // gf_path tool
    pi.registerTool({{
        name: "gf_path",
        label: "Garfield Path",
        description: "Find shortest path between nodes",
        parameters: Type.Object({{
            source: Type.String(),
            target: Type.String(),
        }}),
        async execute(toolCallId, params) {{
            const graph = loadGraph();
            if (!graph) {{
                return {{
                    content: [{{ type: "text", text: "No graph found" }}],
                    details: {{ error: "no_graph" }},
                }};
            }}
            
            const source = graph.nodes.find(n => n.label.toLowerCase().includes(params.source.toLowerCase()));
            const target = graph.nodes.find(n => n.label.toLowerCase().includes(params.target.toLowerCase()));
            
            if (!source || !target) {{
                return {{
                    content: [{{ type: "text", text: "Source or target not found" }}],
                    details: {{ error: "not_found" }},
                }};
            }}
            
            return {{
                content: [{{ 
                    type: "text", 
                    text: `Run 'gf path "${{source.label}}" "${{target.label}}"' in terminal` 
                }}],
                details: {{ source: source.label, target: target.label }},
            }};
        }},
    }});
}}
"#)
}

/// Generate SKILL.md content
fn generate_skill_md() -> String {
    r#"---
name: garfield
description: Query Garfield knowledge graph for code architecture. Use when asked about architecture, code relationships, or god nodes.
---

# Garfield Knowledge Graph

Garfield is a Rust-based knowledge graph builder for source code.

## Rules

Before searching files, check if graph exists at `graphify-out/graph.json`.

Use Garfield when:
- User asks about "architecture", "code structure"
- User asks about "what connects A to B"
- User asks about "god nodes", "key classes"

## Tools

```
gf_build      - Build graph if not exists
gf_graph_query - Query relationships
gf_path       - Find path between nodes
gf_explain    - Explain specific node
```

## Workflow

```
1. gf_build (if no graph)
2. gf_graph_query (ask questions)
3. gf_path (find connections)
```
"#.to_string()
}

/// Install agent configuration files (AGENTS.md, MCP configs, etc.)
fn install_agents(agent: &str, force: bool) {
    use std::fs;
    
    println!("gf install agents - Setting up agent configurations\n");
    
    let install_claude = agent == "all" || agent == "claude" || agent == "cursor";
    let install_mcp = agent == "all" || agent == "mcp" || agent == "claude-desktop";
    
    // Get current directory
    let cwd = std::env::current_dir().unwrap_or_default();
    let graph_json_path = cwd.join("graphify-out/graph.json");
    let graph_json_absolute = graph_json_path.to_string_lossy().to_string();
    
    // Get gf binary path
    let gf_binary = if cfg!(windows) { "gf.exe" } else { "gf" };
    
    if install_claude {
        let agents_md = cwd.join("AGENTS.md");
        println!("📝 Installing AGENTS.md...");
        println!("  Path: {}", agents_md.display());
        
        let content = generate_agents_md(gf_binary);
        
        if agents_md.exists() && !force {
            println!("  ⚠️  AGENTS.md already exists (use -f to overwrite)");
        } else {
            fs::write(&agents_md, content).expect("Failed to write AGENTS.md");
            println!("  ✅ AGENTS.md installed!");
        }
    }
    
    if install_mcp {
        let mcp_config = cwd.join(".claude_desktop_config.json");
        println!("\n🔌 Installing MCP config for Claude Desktop...");
        println!("  Path: {}", mcp_config.display());
        
        let content = generate_mcp_config(&graph_json_absolute, gf_binary);
        
        if mcp_config.exists() && !force {
            println!("  ⚠️  .claude_desktop_config.json already exists (use -f to overwrite)");
        } else {
            fs::write(&mcp_config, content).expect("Failed to write MCP config");
            println!("  ✅ MCP config installed!");
        }
    }
    
    println!("\n✨ Agent configuration complete!");
    println!("\nSupported agents:");
    println!("  Claude Code    - Reads AGENTS.md automatically");
    println!("  Cursor        - Reads AGENTS.md automatically");
    println!("  Claude Desktop - Uses .claude_desktop_config.json");
}

/// Generate AGENTS.md content
fn generate_agents_md(gf_binary: &str) -> String {
    format!(r#"# Garfield Knowledge Graph

Garfield is a fast Rust-based code knowledge graph builder.

## Commands

```bash
# Build knowledge graph
{gf_binary} build <path>          # Full build
{gf_binary} build <path> --update # Incremental build

# Query
{gf_binary} query "function_name" # BFS traversal
{gf_binary} query "X" --dfs       # DFS traversal

# Find paths
{gf_binary} path "A" "B"         # Shortest path A → B

# Explain
{gf_binary} explain "NodeName"    # Node details
```

## When to Use

- Understanding codebase architecture
- Finding what connects A to B
- Identifying god nodes and key abstractions
- Before modifying unfamiliar code

## Workflow

```
1. {gf_binary} build . (if no graph exists)
2. {gf_binary} query "what connects X to Y?"
3. {gf_binary} path "X" "Y" (find direct path)
4. {gf_binary} explain "NodeName" (node details)
```

## Output

- `graphify-out/graph.json` - Knowledge graph
- `graphify-out/GRAPH_REPORT.md` - Human-readable report

## Integration

Garfield tools are available as:
- CLI commands (shown above)
- PI extension (`{gf_binary} install pi`)
- MCP server (`{gf_binary} serve`)
"#, gf_binary = gf_binary)
}

/// Generate Claude Desktop MCP config
fn generate_mcp_config(graph_json_path: &str, gf_binary: &str) -> String {
    let config = format!(r#"{{
  "mcpServers": {{
    "garfield": {{
      "command": "{gf_binary}",
      "args": ["serve", "{graph_json_path}"]
    }}
  }}
}}"#, gf_binary = gf_binary, graph_json_path = graph_json_path);
    
    // Format JSON nicely
    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&config) {
        serde_json::to_string_pretty(&parsed).unwrap_or(config)
    } else {
        config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_generate_extension() {
        let ext = generate_extension_ts("/usr/local/bin/gf");
        assert!(ext.contains("garfieldExtension"));
        assert!(ext.contains("gf_build"));
        assert!(ext.contains("gf_graph_query"));
    }
    
    #[test]
    fn test_generate_skill() {
        let skill = generate_skill_md();
        assert!(skill.contains("Garfield"));
        assert!(skill.contains("gf_build"));
    }
}

