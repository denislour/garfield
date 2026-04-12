//! Garfield CLI - Build knowledge graph from source code

use clap::Parser;

#[derive(Parser)]
#[command(name = "garfield")]
#[command(
    long_about = "Build knowledge graph from source code\n\nGarfield is a simplified Rust version of graphify, focusing on code extraction only."
)]
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
        #[arg(long, default_value = "garfield-out")]
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
        #[arg(long, default_value = "garfield-out/graph.json")]
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
        #[arg(long, default_value = "garfield-out/graph.json")]
        graph: String,
    },

    /// Explain a specific node
    Explain {
        /// Node name or pattern
        name: String,

        /// Graph file path
        #[arg(long, default_value = "garfield-out/graph.json")]
        graph: String,
    },

    /// Install agent integration (pi, claude, cursor)
    Agent {
        /// Install agent: pi, claude, cursor
        #[arg(value_enum)]
        name: AgentName,

        /// Force overwrite existing files
        #[arg(long, short = 'f')]
        force: bool,
    },

    /// Uninstall agent integration (pi, claude, cursor)
    Uninstall {
        /// Uninstall agent: pi, claude, cursor
        #[arg(value_enum)]
        name: AgentName,
    },
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum AgentName {
    Pi,
    Claude,
    Cursor,
}

fn main() {
    let cli = Cli::parse();

    match cli {
        Cli::Build {
            path,
            update,
            output,
        } => {
            println!("garfield v{} - Building graph", env!("CARGO_PKG_VERSION"));
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

        Cli::Query {
            question,
            dfs,
            depth,
            budget,
            graph,
        } => {
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

        Cli::Path {
            source,
            target,
            max_hops,
            graph,
        } => {
            println!(
                "Finding path: {} → {} (max {} hops)\n",
                source, target, max_hops
            );

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
                    let node = g.nodes.iter().find(|n| {
                        n.label.to_lowercase().contains(&name.to_lowercase())
                            || n.id.to_lowercase().contains(&name.to_lowercase())
                    });

                    if let Some(node) = node {
                        println!("NODE: {}", node.label);
                        println!("  ID: {}", node.id);
                        println!("  File: {}", node.source_file);
                        println!("  Location: {}", node.source_location);
                        if let Some(c) = node.community {
                            println!("  Community: {}", c);
                        }

                        // Find connections
                        let connections: Vec<_> = g
                            .links
                            .iter()
                            .filter(|e| e.source == node.id || e.target == node.id)
                            .collect();

                        if !connections.is_empty() {
                            println!("\n  Connections ({}):", connections.len());
                            for conn in connections.iter().take(10) {
                                let other = if conn.source == node.id {
                                    &conn.target
                                } else {
                                    &conn.source
                                };
                                let other_label = g
                                    .nodes
                                    .iter()
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

        Cli::Agent { name, force } => {
            let name_str = match name {
                AgentName::Pi => "pi",
                AgentName::Claude => "claude",
                AgentName::Cursor => "cursor",
            };
            install_agent(name_str, force);
        }

        Cli::Uninstall { name } => {
            let name_str = match name {
                AgentName::Pi => "pi",
                AgentName::Claude => "claude",
                AgentName::Cursor => "cursor",
            };
            uninstall_agent(name_str);
        }
    }
}

/// Install agent integration
fn install_agent(name: &str, force: bool) {
    println!("garfield agent {} - Installing agent integration\n", name);

    // Get home directory and exe path
    let home = dirs::home_dir().expect("Could not find home directory");
    let current_exe = std::env::current_exe().expect("Could not get current executable path");
    let exe_path = current_exe.to_string_lossy().to_string();
    let garfield_binary = "garfield";

    // Get current directory for graph path
    let cwd = std::env::current_dir().unwrap_or_default();
    let graph_json_path = cwd.join("garfield-out/graph.json");
    let graph_json_absolute = graph_json_path.to_string_lossy().to_string();

    // Get agent directory
    let agent_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("agents")
        .join(name);

    if !agent_dir.exists() {
        eprintln!("❌ Unknown agent: {}", name);
        eprintln!("Available agents:");
        eprintln!("  pi      - PI agent");
        eprintln!("  claude  - Claude Code");
        eprintln!("  cursor  - Cursor IDE");
        std::process::exit(1);
    }

    match name {
        "pi" => install_pi_agent(&home, &exe_path, force),
        "claude" => install_claude_agent(&cwd, garfield_binary, &graph_json_absolute, force),
        "cursor" => install_cursor_agent(&cwd, garfield_binary, force),
        _ => {
            eprintln!("❌ Unknown agent: {}", name);
            std::process::exit(1);
        }
    }
}

/// Uninstall agent integration
fn uninstall_agent(name: &str) {
    println!("garfield uninstall {} - Removing agent integration\n", name);

    let home = dirs::home_dir().expect("Could not find home directory");

    match name {
        "pi" => {
            let ext_dir = home.join(".pi/agent/extensions/garfield");
            let skill_dir = home.join(".pi/agent/skills/garfield");

            // Remove extension
            if ext_dir.exists() {
                std::fs::remove_dir_all(&ext_dir).ok();
                println!("✅ PI Extension removed: {}", ext_dir.display());
            } else {
                println!("  PI Extension not found");
            }

            // Remove skill
            if skill_dir.exists() {
                std::fs::remove_dir_all(&skill_dir).ok();
                println!("✅ PI Skill removed: {}", skill_dir.display());
            } else {
                println!("  PI Skill not found");
            }

            println!("\n✨ PI agent uninstallation complete!");
        }
        "claude" => {
            let cwd = std::env::current_dir().unwrap_or_default();
            let agents_md = cwd.join("AGENTS.md");
            let mcp_config = cwd.join(".claude_desktop_config.json");

            // Remove AGENTS.md section
            if agents_md.exists() {
                let content = std::fs::read_to_string(&agents_md).unwrap_or_default();
                if content.contains("## garfield") || content.contains("## Garfield") {
                    let cleaned = remove_garfield_section(&content);
                    std::fs::write(&agents_md, cleaned).ok();
                    println!("✅ AGENTS.md section removed");
                } else {
                    println!("  AGENTS.md section not found");
                }
            }

            // Remove MCP config
            if mcp_config.exists() {
                std::fs::remove_file(&mcp_config).ok();
                println!("✅ Claude Desktop config removed");
            }

            println!("\n✨ Claude agent uninstallation complete!");
        }
        "cursor" => {
            let cwd = std::env::current_dir().unwrap_or_default();
            let agents_md = cwd.join("AGENTS.md");

            // Remove AGENTS.md section
            if agents_md.exists() {
                let content = std::fs::read_to_string(&agents_md).unwrap_or_default();
                if content.contains("## garfield") || content.contains("## Garfield") {
                    let cleaned = remove_garfield_section(&content);
                    std::fs::write(&agents_md, cleaned).ok();
                    println!("✅ AGENTS.md section removed");
                } else {
                    println!("  AGENTS.md section not found");
                }
            }

            println!("\n✨ Cursor agent uninstallation complete!");
        }
        _ => {
            eprintln!("❌ Unknown agent: {}", name);
            std::process::exit(1);
        }
    }
}

/// Remove ## garfield section from markdown
#[allow(unused_variables)]
fn remove_garfield_section(content: &str) -> String {
    // Find ## garfield section manually without lookahead
    let lines: Vec<&str> = content.lines().collect();
    let mut result_lines: Vec<&str> = Vec::new();
    let mut in_garfield_section = false;

    for line in &lines {
        let lower = line.to_lowercase();
        if lower.starts_with("## garfield") {
            in_garfield_section = true;
            continue;
        }

        if in_garfield_section && lower.starts_with("## ") {
            // Next section started
            in_garfield_section = false;
            result_lines.push(line);
        } else if !in_garfield_section {
            result_lines.push(line);
        }
    }

    result_lines.join("\n")
}

/// Install PI agent
fn install_pi_agent(home: &std::path::Path, exe_path: &str, force: bool) {
    use std::fs;

    let ext_dir = home.join(".pi/agent/extensions/garfield");
    let skill_dir = home.join(".pi/agent/skills/garfield");

    // Get agent source directory
    let src_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("agents/pi");

    // Install extension
    let ext_src = src_dir.join("index.ts");
    if ext_src.exists() {
        println!("📦 Installing PI Extension...");
        println!("  Source: {}", ext_src.display());
        println!("  Dest:   {}", ext_dir.display());

        fs::create_dir_all(&ext_dir).expect("Failed to create extension directory");

        let ext_dst = ext_dir.join("index.ts");
        if ext_dst.exists() && !force {
            println!("  ⚠️  Extension already exists (use -f to overwrite)");
        } else {
            // Generate extension with correct exe path
            let ext_content = generate_extension_ts(exe_path);
            fs::write(&ext_dst, ext_content).expect("Failed to write extension file");
            println!("  ✅ Extension installed!");
        }
    }

    // Install skill
    let skill_src = src_dir.join("SKILL.md");
    if skill_src.exists() {
        println!("\n📚 Installing PI Skill...");
        println!("  Source: {}", skill_src.display());
        println!("  Dest:   {}", skill_dir.display());

        fs::create_dir_all(&skill_dir).expect("Failed to create skill directory");

        let skill_dst = skill_dir.join("SKILL.md");
        if skill_dst.exists() && !force {
            println!("  ⚠️  Skill already exists (use -f to overwrite)");
        } else {
            let skill_content = fs::read_to_string(&skill_src).expect("Failed to read skill file");
            // Update skill content with correct output path
            let updated_content = skill_content.replace("graphify-out", "garfield-out");
            fs::write(&skill_dst, updated_content).expect("Failed to write skill file");
            println!("  ✅ Skill installed!");
        }
    }

    println!("\n✨ PI agent installation complete!");
    println!("\nNext steps:");
    println!("  1. Start PI: pi");
    println!("  2. Type /reload to load the extension");
    println!("  3. Try: /garfield help");
}

/// Install Claude agent (AGENTS.md + MCP config)
fn install_claude_agent(
    cwd: &std::path::Path,
    garfield_binary: &str,
    graph_json_path: &str,
    force: bool,
) {
    let agents_md = cwd.join("AGENTS.md");
    let mcp_config = cwd.join(".claude_desktop_config.json");

    println!("📝 Installing Claude Code integration...");
    println!("  AGENTS.md: {}", agents_md.display());

    if agents_md.exists() && !force {
        let content = std::fs::read_to_string(&agents_md).unwrap_or_default();
        if content.contains("## garfield") || content.contains("## Garfield") {
            println!("  ⚠️  AGENTS.md already has garfield section (use -f to overwrite)");
        } else {
            let new_content = content.trim_end().to_string()
                + "\n\n"
                + &generate_garfield_section(garfield_binary);
            std::fs::write(&agents_md, new_content).expect("Failed to write AGENTS.md");
            println!("  ✅ AGENTS.md section added!");
        }
    } else {
        let content = generate_garfield_section(garfield_binary);
        std::fs::write(&agents_md, content).expect("Failed to write AGENTS.md");
        println!("  ✅ AGENTS.md created!");
    }

    println!("\n🔌 Installing Claude Desktop MCP...");
    println!("  Config: {}", mcp_config.display());

    if mcp_config.exists() && !force {
        println!("  ⚠️  .claude_desktop_config.json already exists (use -f to overwrite)");
    } else {
        let content = generate_mcp_config(graph_json_path, garfield_binary);
        std::fs::write(&mcp_config, content).expect("Failed to write MCP config");
        println!("  ✅ MCP config installed!");
    }

    println!("\n✨ Claude agent installation complete!");
}

/// Install Cursor agent (AGENTS.md)
fn install_cursor_agent(cwd: &std::path::Path, garfield_binary: &str, force: bool) {
    let agents_md = cwd.join("AGENTS.md");

    println!("📝 Installing Cursor integration...");
    println!("  AGENTS.md: {}", agents_md.display());

    if agents_md.exists() && !force {
        let content = std::fs::read_to_string(&agents_md).unwrap_or_default();
        if content.contains("## garfield") || content.contains("## Garfield") {
            println!("  ⚠️  AGENTS.md already has garfield section (use -f to overwrite)");
        } else {
            let new_content = content.trim_end().to_string()
                + "\n\n"
                + &generate_garfield_section(garfield_binary);
            std::fs::write(&agents_md, new_content).expect("Failed to write AGENTS.md");
            println!("  ✅ AGENTS.md section added!");
        }
    } else {
        let content = generate_garfield_section(garfield_binary);
        std::fs::write(&agents_md, content).expect("Failed to write AGENTS.md");
        println!("  ✅ AGENTS.md created!");
    }

    println!("\n✨ Cursor agent installation complete!");
}

/// Generate ## garfield section for AGENTS.md
fn generate_garfield_section(_gf_binary: &str) -> String {
    r#"## garfield

This project has a garfield knowledge graph at garfield-out/.

Rules:
- Before answering architecture or codebase questions, read garfield-out/GRAPH_REPORT.md for god nodes and community structure
- After modifying code files in this session, run `garfield build . --update` to keep the graph current

Commands:
- `garfield build <path>` - Build knowledge graph
- `garfield query "X"` - Query the graph
- `garfield path "A" "B"` - Find path between nodes
- `garfield explain "Node"` - Explain a node
"#.to_string()
}

/// Generate TypeScript extension content
fn generate_extension_ts(exe_path: &str) -> String {
    format!(
        r#"/**
 * Garfield PI Extension
 * 
 * Auto-generated by: garfield agent pi
 * Garfield is a Rust-based code extraction tool.
 * Binary: {exe_path}
 */

import type {{ ExtensionAPI }} from "@mariozechner/pi-coding-agent";
import {{ Type }} from "@sinclair/typebox";
import {{ existsSync, readFileSync, execSync }} from "node:fs";
import {{ join, resolve }} from "node:path";

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

// Garfield binary path
const GF_BINARY = "{exe_path}";
const GRAPH_DIR = "garfield-out";
const GRAPH_JSON = join(GRAPH_DIR, "graph.json");
const GRAPH_REPORT = join(GRAPH_DIR, "GRAPH_REPORT.md");

function loadGraph(): GarfieldGraph | null {{
    try {{
        if (!existsSync(GRAPH_JSON)) return null;
        return JSON.parse(readFileSync(GRAPH_JSON, "utf-8"));
    }} catch {{
        return null;
    }}
}}

function runGarfield(args: string[]): {{ stdout: string; stderr: string; code: number }} {{
    try {{
        const stdout = execSync(`${{GF_BINARY}} ${{args.join(' ')}}`, {{ encoding: "utf-8" }});
        return {{ stdout, stderr: "", code: 0 }};
    }} catch (e: any) {{
        return {{ stdout: e.stdout || "", stderr: e.stderr || e.message, code: e.status || 1 }};
    }}
}}

export default function garfieldExtension(pi: ExtensionAPI) {{
    pi.on("session_start", async (_event, ctx) => {{
        // Check if graph exists and notify
        const graph = loadGraph();
        if (graph) {{
            ctx.ui.notify(`Garfield: ${{graph.metadata?.total_nodes || 0}} nodes, ${{graph.metadata?.total_edges || 0}} edges`, "success");
        }}
    }});

    // /garfield command
    pi.registerCommand("garfield", {{
        description: "Garfield: build, query, path, explain, report",
        handler: async (args, ctx) => {{
            const parts = args.trim().split(/\s+/).filter(Boolean);
            const cmd = parts[0] || "help";
            
            switch (cmd) {{
                case "help":
                    ctx.ui.notify("garfield commands:\n/build, /query, /path, /explain, /report", "info");
                    break;
                case "build": {{
                    const path = parts[1] || ".";
                    const update = parts.includes("--update");
                    ctx.ui.notify(`Building graph from ${{path}}...`, "info");
                    const result = runGarfield(["build", path, ...(update ? ["--update"] : [])]);
                    if (result.code === 0) {{
                        ctx.ui.notify("Build complete!", "success");
                    }} else {{
                        ctx.ui.notify("Build failed: " + result.stderr, "error");
                    }}
                    break;
                }}
                case "query": {{
                    const question = parts.slice(1).join(" ");
                    if (!question) {{
                        ctx.ui.notify("Usage: /garfield query \"question\"", "error");
                        break;
                    }}
                    const result = runGarfield(["query", question]);
                    ctx.ui.notify(result.stdout || result.stderr || "No results", result.code === 0 ? "info" : "error");
                    break;
                }}
                case "path": {{
                    const source = parts[1];
                    const target = parts[2];
                    if (!source || !target) {{
                        ctx.ui.notify("Usage: /garfield path \"A\" \"B\"", "error");
                        break;
                    }}
                    const result = runGarfield(["path", source, target]);
                    ctx.ui.notify(result.stdout || result.stderr || "No path found", result.code === 0 ? "info" : "error");
                    break;
                }}
                case "explain": {{
                    const name = parts.slice(1).join(" ");
                    if (!name) {{
                        ctx.ui.notify("Usage: /garfield explain \"NodeName\"", "error");
                        break;
                    }}
                    const result = runGarfield(["explain", name]);
                    ctx.ui.notify(result.stdout || result.stderr || "Node not found", result.code === 0 ? "info" : "error");
                    break;
                }}
                case "report": {{
                    if (existsSync(GRAPH_REPORT)) {{
                        const report = readFileSync(GRAPH_REPORT, "utf-8");
                        ctx.ui.notify(report.substring(0, 500) + "...", "info");
                    }} else {{
                        ctx.ui.notify("No graph found. Run /garfield build first.", "warning");
                    }}
                    break;
                }}
                case ".": {{
                    // Shortcut for /garfield . (build current directory)
                    ctx.ui.notify("Building graph from current directory...", "info");
                    const result = runGarfield(["build", "."]);
                    if (result.code === 0) {{
                        ctx.ui.notify("Build complete!", "success");
                    }} else {{
                        ctx.ui.notify("Build failed: " + result.stderr, "error");
                    }}
                    break;
                }}
                default:
                    ctx.ui.notify("Run /garfield help for available commands", "info");
            }}
        }},
    }});

    // Alias /gf to /garfield
    pi.registerCommand("gf", {{
        description: "Alias for /garfield",
        handler: async (args, ctx) => {{
            // Delegate to garfield command
            const parts = args.trim().split(/\s+/).filter(Boolean);
            if (parts[0] === "." && parts.length === 1) {{
                ctx.ui.notify("Building graph from current directory...", "info");
                const result = runGarfield(["build", "."]);
                if (result.code === 0) {{
                    ctx.ui.notify("Build complete!", "success");
                }} else {{
                    ctx.ui.notify("Build failed: " + result.stderr, "error");
                }}
            }} else {{
                ctx.ui.notify("Use /garfield instead", "info");
            }}
        }},
    }});

    // garfield_build tool
    pi.registerTool({{
        name: "garfield_build",
        label: "Garfield Build",
        description: "Build Garfield knowledge graph from source code",
        parameters: Type.Object({{
            path: Type.Optional(Type.String()),
            update: Type.Optional(Type.Boolean()),
        }}),
        async execute(toolCallId, params) {{
            const path = params.path || ".";
            const update = params.update || false;
            
            const result = runGarfield(["build", path, ...(update ? ["--update"] : [])]);
            
            if (result.code === 0) {{
                const graph = loadGraph();
                return {{
                    content: [{{ type: "text", text: `Graph built: ${{graph?.metadata?.total_nodes || 0}} nodes, ${{graph?.metadata?.total_edges || 0}} edges` }}],
                    details: graph?.metadata,
                }};
            }}
            return {{
                content: [{{ type: "text", text: "Build failed: " + result.stderr }}],
                details: {{ error: result.stderr }},
            }};
        }},
    }});

    // garfield_graph_query tool
    pi.registerTool({{
        name: "garfield_graph_query",
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
                    content: [{{ type: "text", text: "No graph found. Run 'garfield build' first." }}],
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

    // garfield_path tool
    pi.registerTool({{
        name: "garfield_path",
        label: "Garfield Path",
        description: "Find shortest path between nodes",
        parameters: Type.Object({{
            source: Type.String(),
            target: Type.String(),
        }}),
        async execute(toolCallId, params) {{
            const result = runGarfield(["path", params.source, params.target]);
            return {{
                content: [{{ 
                    type: "text", 
                    text: result.stdout || result.stderr || "No path found"
                }}],
                details: {{ code: result.code }},
            }};
        }},
    }});
}}
"#,
        exe_path = exe_path
    )
}

/// Generate Claude Desktop MCP config
fn generate_mcp_config(graph_json_path: &str, garfield_binary: &str) -> String {
    let config = serde_json::json!({
        "mcpServers": {
            "garfield": {
                "command": garfield_binary,
                "args": ["serve", graph_json_path]
            }
        }
    });

    serde_json::to_string_pretty(&config).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_extension() {
        let ext = generate_extension_ts("/usr/local/bin/garfield");
        assert!(ext.contains("garfieldExtension"));
        assert!(ext.contains("garfield_build"));
        assert!(ext.contains("garfield_graph_query"));
    }

    #[test]
    fn test_generate_mcp_config() {
        let config = generate_mcp_config("/path/to/graph.json", "garfield");
        assert!(config.contains("garfield"));
        assert!(config.contains("serve"));
    }

    #[test]
    fn test_remove_garfield_section() {
        let content = "# Header\n\n## garfield\n\ncontent here\n\n## Other\n\nmore content";
        let result = remove_garfield_section(content);
        assert!(!result.contains("garfield"));
        assert!(result.contains("Other"));
    }
}
