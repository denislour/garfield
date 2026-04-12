/**
 * Garfield PI Extension
 * 
 * Provides /gf command and tools for querying Garfield knowledge graphs.
 * Garfield is a Rust-based code extraction tool (simplified graphify).
 * 
 * Usage:
 *   /gf build <path>       - Build knowledge graph
 *   /gf query <question>   - Query the graph
 *   /gf path <A> <B>       - Find path between nodes
 *   /gf explain <name>     - Explain a node
 *   /gf report             - Show graph report
 */

import type { ExtensionAPI } from "@mariozechner/pi-coding-agent";
import { Type } from "@sinclair/typebox";
import { existsSync, readFileSync, writeFileSync, mkdirSync } from "node:fs";
import { join, dirname } from "node:path";

// Global context for PI agent
let piContext: { bash?: (cmd: string) => Promise<string>; ui?: unknown } | null = null;

interface GarfieldGraph {
    nodes: Array<{
        id: string;
        label: string;
        source_file: string;
        source_location?: string;
        community?: number;
    }>;
    edges: Array<{
        source: string;
        target: string;
        relation: string;
        confidence: string;
    }>;
    metadata?: {
        total_nodes: number;
        total_edges: number;
        communities: number;
    };
}

interface GarfieldReport {
    title: string;
    summary: string;
    god_nodes: Array<{ id: string; label: string; edges: number }>;
    communities: Array<{
        id: number;
        nodes: string[];
        cohesion: number;
    }>;
    surprising_connections: Array<{
        source: string;
        target: string;
        score: number;
    }>;
}

// Check if garfield binary is available
function isGarfieldAvailable(): boolean {
    try {
        const { execSync } = require("child_process");
        execSync("which garfield", { stdio: "ignore" });
        return true;
    } catch {
        return false;
    }
}

// Run garfield command
async function runGarfield(args: string[]): Promise<string> {
    // Try piContext.bash first if available (PI agent's built-in method)
    if (piContext?.bash) {
        try {
            return await piContext.bash(`garfield ${args.join(" ")}`);
        } catch (error: unknown) {
            // Fall through to try other methods
        }
    }
    
    // Fallback: try different binary paths with execSync
    const binaryPaths = [
        "garfield",
        "./target/release/garfield",
        "./garfield",
        "/home/jake/Compa/garfield/target/release/garfield"
    ];
    
    let lastError = "";
    
    for (const binary of binaryPaths) {
        try {
            // Use spawn instead of execSync for better compatibility
            const { spawn } = require("child_process");
            return new Promise((resolve, reject) => {
                const proc = spawn(binary, args, {
                    encoding: "utf-8",
                    timeout: 120000,
                    shell: true
                });
                
                let stdout = "";
                let stderr = "";
                
                proc.stdout?.on("data", (data: string) => { stdout += data; });
                proc.stderr?.on("data", (data: string) => { stderr += data; });
                
                proc.on("close", (code: number) => {
                    if (code === 0) {
                        resolve(stdout);
                    } else {
                        reject(new Error(stderr || `Exit code: ${code}`));
                    }
                });
                
                proc.on("error", (err: Error) => {
                    reject(err);
                });
            });
        } catch (error: unknown) {
            const err = error as { message?: string };
            lastError = err.message || "Unknown error";
            continue;
        }
    }
    
    throw new Error(`Garfield not found. Tried: ${binaryPaths.join(", ")}. Last error: ${lastError}`);
}

// Load graph from JSON file
function loadGraph(graphPath: string = "graphify-out/graph.json"): GarfieldGraph | null {
    try {
        if (!existsSync(graphPath)) {
            return null;
        }
        const content = readFileSync(graphPath, "utf-8");
        return JSON.parse(content);
    } catch {
        return null;
    }
}

// Load report
function loadReport(reportPath: string = "graphify-out/GRAPH_REPORT.md"): string | null {
    try {
        if (!existsSync(reportPath)) {
            return null;
        }
        return readFileSync(reportPath, "utf-8");
    } catch {
        return null;
    }
}

// Build graph using garfield binary
async function buildGraph(path: string, update: boolean = false): Promise<{
    nodes: number;
    edges: number;
    communities: number;
    changed: number;
    cached: number;
}> {
    const output = "graphify-out";
    const args = update ? ["build", path, "--update", "--output", output] : ["build", path, "--output", output];
    
    await runGarfield(args);
    
    const graph = loadGraph();
    if (!graph) {
        throw new Error("Failed to build graph");
    }
    
    return {
        nodes: graph.metadata?.total_nodes || graph.nodes.length,
        edges: graph.metadata?.total_edges || graph.edges.length,
        communities: graph.metadata?.communities || 0,
        changed: 0,
        cached: 0,
    };
}

// Query graph using BFS/DFS
async function queryGraph(question: string, dfs: boolean = false, depth: number = 3): Promise<string> {
    const flag = dfs ? "--dfs" : "";
    const result = await runGarfield(["query", `"${question}"`, "--depth", String(depth), flag]);
    return result;
}

// Find shortest path
async function findPath(source: string, target: string, maxHops: number = 8): Promise<string[] | null> {
    try {
        const result = await runGarfield(["path", `"${source}"`, `"${target}"`, "--max-hops", String(maxHops)]);
        // Parse result - extract nodes from output
        const lines = result.split("\n").filter(line => line.trim() && !line.includes("→") && !line.includes("Path found"));
        if (lines.length === 0) return null;
        return lines.map(line => line.trim().replace(/^  /, ""));
    } catch {
        return null;
    }
}

// Explain node
async function explainNode(name: string): Promise<string | null> {
    try {
        return await runGarfield(["explain", `"${name}"`]);
    } catch {
        return null;
    }
}

export default function garfieldExtension(pi: ExtensionAPI) {
    // Notify on load
    pi.on("session_start", async (_event, ctx) => {
        // Save ctx globally for runGarfield to use
        piContext = ctx as { bash?: (cmd: string) => Promise<string>; ui?: unknown };
        if (isGarfieldAvailable()) {
            ctx.ui.notify("Garfield extension loaded (gf available)", "success");
        } else {
            ctx.ui.notify("Garfield extension loaded (gf not found - install from garfield/)", "warning");
        }
    });

    // Register /gf command
    pi.registerCommand("gf", {
        description: "Garfield knowledge graph commands: build, query, path, explain, report",
        getArgumentCompletions: (prefix) => {
            const subcommands = ["build", "query", "path", "explain", "report", "help"];
            const filtered = subcommands.filter(s => s.startsWith(prefix));
            return filtered.length > 0 ? filtered.map(s => ({ value: s, label: s })) : null;
        },
        handler: async (args, ctx) => {
            const parts = args.trim().split(/\s+/);
            const subcommand = parts[0] || "help";

            switch (subcommand) {
                // Handle /gf . or /gf (no args) -> build current directory
                case ".":
                case "":
                case "build": {
                    const path = subcommand === "build" ? (parts[1] || ".") : ".";
                    const update = parts.includes("--update");
                    await ctx.ui.setStatus("gf", "Building graph...");
                    try {
                        const result = await buildGraph(path, update);
                        ctx.ui.notify(
                            `Built: ${result.nodes} nodes, ${result.edges} edges, ${result.communities} communities`,
                            "success"
                        );
                    } catch (error: unknown) {
                        const err = error as { message?: string };
                        ctx.ui.notify(`Build failed: ${err.message}`, "error");
                    }
                    await ctx.ui.setStatus("gf", "Ready");
                    break;
                }

                case "query": {
                    const question = parts.slice(1).join(" ") || "help";
                    const dfs = parts.includes("--dfs");
                    if (question === "help") {
                        ctx.ui.notify("Usage: /gf query <question> [--dfs]", "info");
                        break;
                    }
                    await ctx.ui.setStatus("gf", "Querying...");
                    try {
                        const result = await queryGraph(question, dfs);
                        ctx.ui.notify(`Query: "${question}"\n${result.substring(0, 500)}`, "info");
                    } catch (error: unknown) {
                        const err = error as { message?: string };
                        ctx.ui.notify(`Query failed: ${err.message}`, "error");
                    }
                    await ctx.ui.setStatus("gf", "Ready");
                    break;
                }

                case "path": {
                    if (parts.length < 3) {
                        ctx.ui.notify("Usage: /gf path <source> <target>", "info");
                        break;
                    }
                    const source = parts[1];
                    const target = parts[2];
                    await ctx.ui.setStatus("gf", "Finding path...");
                    try {
                        const path = await findPath(source, target);
                        if (path) {
                            ctx.ui.notify(`Path: ${path.join(" → ")}`, "success");
                        } else {
                            ctx.ui.notify("No path found", "warning");
                        }
                    } catch (error: unknown) {
                        const err = error as { message?: string };
                        ctx.ui.notify(`Path failed: ${err.message}`, "error");
                    }
                    await ctx.ui.setStatus("gf", "Ready");
                    break;
                }

                case "explain": {
                    const name = parts.slice(1).join(" ");
                    if (!name) {
                        ctx.ui.notify("Usage: /gf explain <node-name>", "info");
                        break;
                    }
                    await ctx.ui.setStatus("gf", "Explaining...");
                    try {
                        const result = await explainNode(name);
                        if (result) {
                            ctx.ui.notify(result.substring(0, 500), "info");
                        } else {
                            ctx.ui.notify("Node not found", "warning");
                        }
                    } catch (error: unknown) {
                        const err = error as { message?: string };
                        ctx.ui.notify(`Explain failed: ${err.message}`, "error");
                    }
                    await ctx.ui.setStatus("gf", "Ready");
                    break;
                }

                case "report": {
                    const report = loadReport();
                    if (report) {
                        ctx.ui.notify(report.substring(0, 500) + "...", "info");
                    } else {
                        ctx.ui.notify("No report found. Run /gf build first.", "warning");
                    }
                    break;
                }

                case "help":
                default: {
                    const helpText = `
Garfield Commands:
  /gf build <path>        - Build knowledge graph
  /gf build --update       - Incremental build
  /gf query <question>     - Query graph (BFS)
  /gf query <q> --dfs      - Query graph (DFS)
  /gf path <A> <B>         - Find path A → B
  /gf explain <name>        - Explain node
  /gf report               - Show graph report

Install: cargo install --path garfield/
                    `.trim();
                    ctx.ui.notify(helpText, "info");
                    break;
                }
            }
        },
    });

    // Register gf_graph_query tool for LLM
    pi.registerTool({
        name: "gf_graph_query",
        label: "Garfield Query",
        description: `Query the Garfield knowledge graph for code relationships.
        
Use this when:
- User asks about "architecture", "code structure", "how does X work"
- User asks about "what connects A to B"
- User asks about "god nodes", "key classes", "core abstractions"
- Before searching raw files for structural questions

Garfield extracts code structure using tree-sitter (248+ languages).
It identifies definitions, calls, and imports to build a knowledge graph.`,
        parameters: Type.Object({
            question: Type.String({
                description: "Natural language question about code relationships (e.g., 'what does CartItemService connect to?')"
            }),
            mode: Type.Optional(Type.Union([
                Type.Literal("bfs"),
                Type.Literal("dfs")
            ], {
                description: "Traversal mode: bfs (breadth-first, default) or dfs (depth-first)"
            })),
            depth: Type.Optional(Type.Number({
                description: "Maximum traversal depth (default: 3)",
                minimum: 1,
                maximum: 10
            })),
        }),
        async execute(toolCallId, params, signal, onUpdate, ctx) {
            try {
                const graph = loadGraph();
                if (!graph) {
                    return {
                        content: [{
                            type: "text",
                            text: "No graph found. Run 'gf build <path>' first to build the knowledge graph."
                        }],
                        details: { error: "no_graph" },
                    };
                }

                const dfs = params.mode === "dfs";
                const depth = params.depth || 3;
                const result = await queryGraph(params.question, dfs, depth);

                return {
                    content: [{
                        type: "text",
                        text: result || "No results found"
                    }],
                    details: {
                        question: params.question,
                        mode: dfs ? "dfs" : "bfs",
                        depth,
                        nodes: graph.nodes.length,
                        edges: graph.edges.length,
                    },
                };
            } catch (error: unknown) {
                const err = error as { message?: string };
                return {
                    content: [{
                        type: "text",
                        text: `Query failed: ${err.message || "Unknown error"}`
                    }],
                    details: { error: err.message },
                };
            }
        },
    });

    // Register gf_build tool
    pi.registerTool({
        name: "gf_build",
        label: "Garfield Build",
        description: "Build or update the Garfield knowledge graph from source code. Run this before querying.",
        parameters: Type.Object({
            path: Type.Optional(Type.String({
                description: "Path to analyze (default: current directory)"
            })),
            update: Type.Optional(Type.Boolean({
                description: "Incremental update using cache (faster for small changes)"
            })),
        }),
        async execute(toolCallId, params, signal, onUpdate, ctx) {
            try {
                const path = params.path || ".";
                const update = params.update || false;

                onUpdate?.({ content: [{ type: "text", text: `Building graph from ${path}...` }] });

                const result = await buildGraph(path, update);

                return {
                    content: [{
                        type: "text",
                        text: `Graph built successfully!\n\n` +
                              `Nodes: ${result.nodes}\n` +
                              `Edges: ${result.edges}\n` +
                              `Communities: ${result.communities}`
                    }],
                    details: {
                        nodes: result.nodes,
                        edges: result.edges,
                        communities: result.communities,
                        changed_files: result.changed,
                        cached_files: result.cached,
                    },
                };
            } catch (error: unknown) {
                const err = error as { message?: string };
                return {
                    content: [{
                        type: "text",
                        text: `Build failed: ${err.message || "Unknown error"}`
                    }],
                    details: { error: err.message },
                };
            }
        },
    });

    // Register gf_path tool
    pi.registerTool({
        name: "gf_path",
        label: "Garfield Path",
        description: "Find the shortest path between two nodes in the knowledge graph.",
        parameters: Type.Object({
            source: Type.String({
                description: "Source node name or pattern"
            }),
            target: Type.String({
                description: "Target node name or pattern"
            }),
            max_hops: Type.Optional(Type.Number({
                description: "Maximum number of hops (default: 8)",
                minimum: 1,
                maximum: 20
            })),
        }),
        async execute(toolCallId, params, signal, onUpdate, ctx) {
            try {
                const maxHops = params.max_hops || 8;
                const path = await findPath(params.source, params.target, maxHops);

                if (path) {
                    return {
                        content: [{
                            type: "text",
                            text: `Path found (${path.length - 1} hops):\n\n${path.join("\n→ ")}`
                        }],
                        details: {
                            path,
                            hops: path.length - 1,
                        },
                    };
                } else {
                    return {
                        content: [{
                            type: "text",
                            text: `No path found between "${params.source}" and "${params.target}" within ${maxHops} hops.`
                        }],
                        details: { path: null },
                    };
                }
            } catch (error: unknown) {
                const err = error as { message?: string };
                return {
                    content: [{
                        type: "text",
                        text: `Path finding failed: ${err.message || "Unknown error"}`
                    }],
                    details: { error: err.message },
                };
            }
        },
    });

    // Register gf_explain tool
    pi.registerTool({
        name: "gf_explain",
        label: "Garfield Explain",
        description: "Explain a specific node in the knowledge graph, showing its connections.",
        parameters: Type.Object({
            name: Type.String({
                description: "Node name or pattern to explain"
            }),
        }),
        async execute(toolCallId, params, signal, onUpdate, ctx) {
            try {
                const result = await explainNode(params.name);

                if (result) {
                    return {
                        content: [{
                            type: "text",
                            text: result
                        }],
                        details: { found: true },
                    };
                } else {
                    return {
                        content: [{
                            type: "text",
                            text: `Node not found: "${params.name}"`
                        }],
                        details: { found: false },
                    };
                }
            } catch (error: unknown) {
                const err = error as { message?: string };
                return {
                    content: [{
                        type: "text",
                        text: `Explain failed: ${err.message || "Unknown error"}`
                    }],
                    details: { error: err.message },
                };
            }
        },
    });
}
