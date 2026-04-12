//! Graph analysis module
//!
//! ## Analyze Flow
//!
//! 1. find_god_nodes() - Most connected entities
//! 2. find_surprising_connections() - Cross-community edges
//! 3. count_confidence() - Statistics on edge confidence levels
//! 4. calculate_cohesion_scores() - Community density ratios
//! 5. generate_community_labels() - Auto-generate human-readable names
//! 6. suggest_questions() - Generate questions the graph can answer
//! 7. graph_diff() - Compare two graph snapshots

use crate::types::{Confidence, GraphData};
use std::collections::{HashMap, HashSet};

/// Analysis result
#[derive(Debug)]
pub struct Analysis {
    pub god_nodes: Vec<GodNode>,
    pub surprising_connections: Vec<SurprisingConnection>,
    pub community_sizes: HashMap<u32, usize>,
    pub cohesion_scores: HashMap<u32, f64>,
    pub community_labels: HashMap<u32, String>,
    pub confidence_stats: ConfidenceStats,
}

/// God node - most connected node
#[derive(Debug, Clone)]
pub struct GodNode {
    pub node: crate::types::Node,
    pub degree: usize,
    pub edges: Vec<String>,
    pub source_file: String,
}

/// Surprising connection - cross-community edge with explanation
#[derive(Debug, Clone)]
pub struct SurprisingConnection {
    pub source: String,
    pub target: String,
    pub source_label: String,
    pub target_label: String,
    pub relation: String,
    pub confidence: Confidence,
    pub source_community: u32,
    pub target_community: u32,
    pub source_file: String,
    pub target_file: String,
    /// Why this connection is surprising
    pub why: String,
    /// Surprise score (higher = more surprising)
    pub score: i32,
}

/// Confidence statistics
#[derive(Debug, Clone)]
pub struct ConfidenceStats {
    pub extracted: usize,
    pub inferred: usize,
    pub ambiguous: usize,
}

/// Suggested question
#[derive(Debug, Clone)]
pub struct SuggestedQuestion {
    pub question_type: String,
    pub question: String,
    pub why: String,
}

/// Graph diff result
#[derive(Debug, Clone)]
pub struct GraphDiff {
    pub new_nodes: Vec<NodeChange>,
    pub removed_nodes: Vec<NodeChange>,
    pub new_edges: Vec<EdgeChange>,
    pub removed_edges: Vec<EdgeChange>,
    pub summary: String,
}

#[derive(Debug, Clone)]
pub struct NodeChange {
    pub id: String,
    pub label: String,
}

#[derive(Debug, Clone)]
pub struct EdgeChange {
    pub source: String,
    pub target: String,
    pub relation: String,
    pub confidence: String,
}

/// Main analysis function
pub fn analyze(graph: &GraphData) -> Analysis {
    let god_nodes = find_god_nodes(graph, 10);
    let surprising = find_surprising_connections(graph);
    let community_sizes = count_community_sizes(graph);
    let cohesion_scores = calculate_cohesion_scores(graph);
    let community_labels = generate_community_labels(graph, &community_sizes);
    let confidence_stats = count_confidence(graph);

    Analysis {
        god_nodes,
        surprising_connections: surprising,
        community_sizes,
        cohesion_scores,
        community_labels,
        confidence_stats,
    }
}

/// Find top N most connected nodes (god nodes)
/// Filters out file-level hub nodes and method stubs
pub fn find_god_nodes(graph: &GraphData, top_n: usize) -> Vec<GodNode> {
    // Build adjacency for degree calculation
    let mut degree: HashMap<&str, usize> = HashMap::new();
    let mut neighbors: HashMap<&str, Vec<&str>> = HashMap::new();

    for edge in &graph.links {
        *degree.entry(&edge.source).or_insert(0) += 1;
        *degree.entry(&edge.target).or_insert(0) += 1;
        neighbors.entry(&edge.source).or_default().push(&edge.target);
        neighbors.entry(&edge.target).or_default().push(&edge.source);
    }

    // Sort by degree
    let mut sorted: Vec<_> = degree.into_iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1));

    // Take top N and build GodNode structs
    sorted
        .into_iter()
        .filter(|(node_id, _)| !is_file_node(graph, node_id))
        .filter(|(node_id, _)| !is_method_stub(graph, node_id))
        .take(top_n)
        .filter_map(|(node_id, deg)| {
            graph.nodes.iter().find(|n| n.id == node_id).map(|node| {
                let neighbor_labels = neighbors
                    .get(node_id)
                    .map(|ns| {
                        ns.iter()
                            .filter_map(|nid| {
                                graph.nodes.iter().find(|n| &n.id == *nid).map(|n| n.label.clone())
                            })
                            .take(5)
                            .collect()
                    })
                    .unwrap_or_default();

                GodNode {
                    node: node.clone(),
                    degree: deg,
                    edges: neighbor_labels,
                    source_file: node.source_file.clone(),
                }
            })
        })
        .collect()
}

/// Check if node is a file-level hub
fn is_file_node(graph: &GraphData, node_id: &str) -> bool {
    let node = match graph.nodes.iter().find(|n| n.id == node_id) {
        Some(n) => n,
        None => return false,
    };
    let label = &node.label;
    
    // Check if label matches source filename
    let source_file_name = std::path::Path::new(&node.source_file)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");
    
    label == source_file_name
}

/// Check if node is a method stub (starts with . and ends with ())
fn is_method_stub(graph: &GraphData, node_id: &str) -> bool {
    let node = match graph.nodes.iter().find(|n| n.id == node_id) {
        Some(n) => n,
        None => return false,
    };
    let label = &node.label;
    
    // Method stubs: ".method_name()"
    if label.starts_with('.') && label.ends_with("()") {
        return true;
    }
    
    // Check if degree is <= 1 and ends with "()"
    let degree = graph.links.iter()
        .filter(|e| e.source == node_id || e.target == node_id)
        .count();
    
    degree <= 1 && label.ends_with("()")
}

/// Get file category from path
fn get_file_category(path: &str) -> &'static str {
    let ext = std::path::Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    
    match ext.as_str() {
        "py" | "js" | "ts" | "go" | "rs" | "java" | "rb" | "cs" | "kt" | "scala" | "php" | "swift" | "lua" | "zig" | "c" | "cpp" | "h" | "hpp" => "code",
        "md" | "markdown" | "txt" | "rst" => "doc",
        "pdf" => "paper",
        "png" | "jpg" | "jpeg" | "gif" | "svg" | "webp" => "image",
        _ => "other",
    }
}

/// Get top-level directory from path
fn get_top_level_dir(path: &str) -> String {
    std::path::Path::new(path)
        .components()
        .next()
        .map(|c| c.as_os_str().to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string())
}

/// Calculate surprise score for a cross-community edge
fn calculate_surprise_score(
    graph: &GraphData,
    src: &str,
    tgt: &str,
    edge: &crate::types::Edge,
    src_comm: u32,
    tgt_comm: u32,
) -> (i32, String) {
    let mut score = 0;
    let mut reasons: Vec<String> = Vec::new();

    // 1. Confidence weight
    match edge.confidence {
        Confidence::Ambiguous => {
            score += 3;
            reasons.push("AMBIGUOUS connection - not explicitly stated in source".to_string());
        }
        Confidence::Inferred => {
            score += 2;
            reasons.push("INFERRED connection - model-reasoned relationship".to_string());
        }
        Confidence::Extracted => {
            score += 1;
        }
    }

    // 2. Cross file-type bonus
    let src_file = graph.nodes.iter()
        .find(|n| n.id == src)
        .map(|n| n.source_file.as_str())
        .unwrap_or("");
    let tgt_file = graph.nodes.iter()
        .find(|n| n.id == tgt)
        .map(|n| n.source_file.as_str())
        .unwrap_or("");
    
    let cat_src = get_file_category(src_file);
    let cat_tgt = get_file_category(tgt_file);
    
    if cat_src != cat_tgt && cat_src != "other" && cat_tgt != "other" {
        score += 2;
        reasons.push(format!("crosses file types ({} → {})", cat_src, cat_tgt));
    }

    // 3. Cross-repo bonus
    let dir_src = get_top_level_dir(src_file);
    let dir_tgt = get_top_level_dir(tgt_file);
    
    if !dir_src.is_empty() && !dir_tgt.is_empty() && dir_src != dir_tgt {
        score += 2;
        reasons.push("connects across different directories/repos".to_string());
    }

    // 4. Cross-community bonus
    if src_comm != tgt_comm {
        score += 1;
        reasons.push("bridges separate communities".to_string());
    }

    // 5. Peripheral→hub bonus
    let deg_src = graph.links.iter().filter(|e| e.source == src || e.target == src).count();
    let deg_tgt = graph.links.iter().filter(|e| e.source == tgt || e.target == tgt).count();
    
    if (deg_src <= 2 && deg_tgt >= 5) || (deg_tgt <= 2 && deg_src >= 5) {
        score += 1;
        reasons.push("peripheral node unexpectedly reaches hub".to_string());
    }

    let why = if reasons.is_empty() {
        "cross-file semantic connection".to_string()
    } else {
        reasons.join("; ")
    };

    (score, why)
}

/// Find cross-community edges with detailed explanations
pub fn find_surprising_connections(graph: &GraphData) -> Vec<SurprisingConnection> {
    // Identify unique source files
    let source_files: HashSet<&str> = graph
        .nodes
        .iter()
        .filter(|n| !n.source_file.is_empty())
        .map(|n| n.source_file.as_str())
        .collect();
    
    let is_multi_source = source_files.len() > 1;
    
    if is_multi_source {
        find_cross_file_surprises(graph)
    } else {
        find_cross_community_surprises(graph)
    }
}

/// Find surprising cross-file connections
fn find_cross_file_surprises(graph: &GraphData) -> Vec<SurprisingConnection> {
    let mut candidates = Vec::new();
    
    for edge in &graph.links {
        let src = edge.source.clone();
        let tgt = edge.target.clone();
        
        // Skip structural relationships
        if ["imports", "imports_from", "contains", "method"].contains(&edge.relation.as_str()) {
            continue;
        }
        
        // Get node info
        let src_node = match graph.nodes.iter().find(|n| n.id == src) {
            Some(n) => n,
            None => continue,
        };
        let tgt_node = match graph.nodes.iter().find(|n| n.id == tgt) {
            Some(n) => n,
            None => continue,
        };
        
        // Skip concept nodes and file nodes
        if is_file_node(graph, &src) || is_file_node(graph, &tgt) {
            continue;
        }
        
        let src_file = &src_node.source_file;
        let tgt_file = &tgt_node.source_file;
        
        // Skip same-file edges
        if src_file.is_empty() || tgt_file.is_empty() || src_file == tgt_file {
            continue;
        }
        
        let src_comm = src_node.community.unwrap_or(0);
        let tgt_comm = tgt_node.community.unwrap_or(0);
        
        let (score, why) = calculate_surprise_score(
            graph, &src, &tgt, edge, src_comm, tgt_comm
        );
        
        candidates.push(SurprisingConnection {
            source: src.clone(),
            target: tgt.clone(),
            source_label: src_node.label.clone(),
            target_label: tgt_node.label.clone(),
            relation: edge.relation.clone(),
            confidence: edge.confidence,
            source_community: src_comm,
            target_community: tgt_comm,
            source_file: src_file.clone(),
            target_file: tgt_file.clone(),
            why,
            score,
        });
    }
    
    // Sort by score descending
    candidates.sort_by(|a, b| {
        b.score.cmp(&a.score)
            .then_with(|| {
                let order = |c: &Confidence| match c {
                    Confidence::Ambiguous => 0,
                    Confidence::Inferred => 1,
                    Confidence::Extracted => 2,
                };
                order(&a.confidence).cmp(&order(&b.confidence))
            })
    });
    
    candidates.into_iter().take(10).collect()
}

/// Find surprising cross-community connections
fn find_cross_community_surprises(graph: &GraphData) -> Vec<SurprisingConnection> {
    let mut surprises = Vec::new();
    let mut seen_pairs: HashSet<(u32, u32)> = HashSet::new();
    
    for edge in &graph.links {
        let src = edge.source.clone();
        let tgt = edge.target.clone();
        
        // Skip structural relationships
        if ["imports", "imports_from", "contains", "method"].contains(&edge.relation.as_str()) {
            continue;
        }
        
        let src_node = match graph.nodes.iter().find(|n| n.id == src) {
            Some(n) => n,
            None => continue,
        };
        let tgt_node = match graph.nodes.iter().find(|n| n.id == tgt) {
            Some(n) => n,
            None => continue,
        };
        
        // Skip file nodes
        if is_file_node(graph, &src) || is_file_node(graph, &tgt) {
            continue;
        }
        
        let src_comm = src_node.community.unwrap_or(0);
        let tgt_comm = tgt_node.community.unwrap_or(0);
        
        // Skip same-community edges
        if src_comm == tgt_comm {
            continue;
        }
        
        // Deduplicate by community pair
        let pair = (src_comm.min(tgt_comm), src_comm.max(tgt_comm));
        if seen_pairs.contains(&pair) {
            continue;
        }
        seen_pairs.insert(pair);
        
        let (score, why) = calculate_surprise_score(
            graph, &src, &tgt, edge, src_comm, tgt_comm
        );
        
        surprises.push(SurprisingConnection {
            source: src.clone(),
            target: tgt.clone(),
            source_label: src_node.label.clone(),
            target_label: tgt_node.label.clone(),
            relation: edge.relation.clone(),
            confidence: edge.confidence,
            source_community: src_comm,
            target_community: tgt_comm,
            source_file: src_node.source_file.clone(),
            target_file: tgt_node.source_file.clone(),
            why,
            score,
        });
    }
    
    // Sort: AMBIGUOUS first, then INFERRED, then EXTRACTED
    surprises.sort_by(|a, b| {
        let order = |c: &Confidence| match c {
            Confidence::Ambiguous => 0,
            Confidence::Inferred => 1,
            Confidence::Extracted => 2,
        };
        order(&a.confidence).cmp(&order(&b.confidence))
    });
    
    surprises.into_iter().take(10).collect()
}

/// Count community sizes
fn count_community_sizes(graph: &GraphData) -> HashMap<u32, usize> {
    let mut sizes: HashMap<u32, usize> = HashMap::new();
    for node in &graph.nodes {
        if let Some(c) = node.community {
            *sizes.entry(c).or_insert(0) += 1;
        }
    }
    sizes
}

/// Count confidence statistics
fn count_confidence(graph: &GraphData) -> ConfidenceStats {
    let mut stats = ConfidenceStats {
        extracted: 0,
        inferred: 0,
        ambiguous: 0,
    };

    for edge in &graph.links {
        match edge.confidence {
            Confidence::Extracted => stats.extracted += 1,
            Confidence::Inferred => stats.inferred += 1,
            Confidence::Ambiguous => stats.ambiguous += 1,
        }
    }

    stats
}

/// Calculate cohesion scores for each community
fn calculate_cohesion_scores(graph: &GraphData) -> HashMap<u32, f64> {
    let n = graph.nodes.len();
    if n == 0 {
        return HashMap::new();
    }

    // Build adjacency list
    let mut adj: HashMap<&str, HashSet<&str>> = HashMap::new();
    for node in &graph.nodes {
        adj.insert(node.id.as_str(), HashSet::new());
    }
    for edge in &graph.links {
        adj.entry(&edge.source).or_default().insert(&edge.target);
        adj.entry(&edge.target).or_default().insert(&edge.source);
    }

    // Group nodes by community
    let mut communities: HashMap<u32, Vec<&str>> = HashMap::new();
    for node in &graph.nodes {
        if let Some(c) = node.community {
            communities.entry(c).or_default().push(&node.id);
        }
    }

    let mut cohesion: HashMap<u32, f64> = HashMap::new();

    for (comm, nodes) in &communities {
        let comm_size = nodes.len();
        if comm_size <= 1 {
            cohesion.insert(*comm, 1.0);
            continue;
        }

        // Count actual intra-community edges
        let mut actual_edges = 0usize;
        for &node_id in nodes {
            if let Some(neighbors) = adj.get(node_id) {
                for &neighbor in neighbors {
                    // Count each edge once (only when neighbor > node)
                    let node_idx = nodes.iter().position(|n| *n == node_id).unwrap();
                    let neighbor_idx = nodes.iter().position(|n| *n == neighbor);
                    if let Some(ni) = neighbor_idx {
                        if ni > node_idx
                            && graph.nodes.iter().any(|n| n.id == neighbor && n.community == Some(*comm))
                        {
                            actual_edges += 1;
                        }
                    }
                }
            }
        }

        // Calculate possible edges: n * (n-1) / 2
        let possible_edges = (comm_size * (comm_size - 1)) as f64 / 2.0;

        let score = if possible_edges > 0.0 {
            actual_edges as f64 / possible_edges
        } else {
            0.0
        };

        cohesion.insert(*comm, (score * 100.0).round() / 100.0);
    }

    cohesion
}

/// Generate automatic community labels
pub fn generate_community_labels(
    graph: &GraphData,
    community_sizes: &HashMap<u32, usize>,
) -> HashMap<u32, String> {
    let mut labels: HashMap<u32, String> = HashMap::new();

    for (&comm, &size) in community_sizes {
        // Get nodes in this community
        let comm_nodes: Vec<_> = graph
            .nodes
            .iter()
            .filter(|n| n.community == Some(comm))
            .collect();

        // Strategy 1: Most common file prefix
        let mut file_prefixes: HashMap<String, usize> = HashMap::new();
        for node in &comm_nodes {
            if let Some(parent) = std::path::Path::new(&node.source_file).parent() {
                let prefix = parent.to_string_lossy().to_string();
                if !prefix.is_empty() && prefix != "." {
                    *file_prefixes.entry(prefix).or_insert(0) += 1;
                }
            }
        }

        // Strategy 2: Most frequent term in labels
        let mut term_counts: HashMap<String, usize> = HashMap::new();
        let noise_terms = [
            "util", "helper", "manager", "handler", "service", "controller",
            "model", "view", "index", "main", "init", "config", "setup",
            "base", "common", "core", "data", "info", "params", "args",
        ];
        
        for node in &comm_nodes {
            let label_lower = node.label.to_lowercase();
            for word in label_lower.split('_') {
                if word.len() > 3 && !noise_terms.contains(&word) {
                    *term_counts.entry(word.to_string()).or_insert(0) += 1;
                }
            }
            // Also check camelCase split
            let mut current_word = String::new();
            for c in node.label.chars() {
                if c.is_uppercase() && !current_word.is_empty() {
                    let w = current_word.to_lowercase();
                    if w.len() > 3 && !noise_terms.contains(&w.as_str()) {
                        *term_counts.entry(w).or_insert(0) += 1;
                    }
                    current_word.clear();
                }
                current_word.push(c);
            }
        }

        // Choose best label
        let label = if let Some((prefix, count)) = file_prefixes.into_iter().max_by_key(|(_, c)| *c) {
            if count * 2 >= size {
                prefix.split('/').last().map(|s| {
                    s.split('_')
                        .map(|part| part.chars().next().unwrap().to_uppercase().to_string() + &part[1..])
                        .collect::<Vec<_>>()
                        .join("")
                }).unwrap_or_else(|| format!("Community {}", comm))
            } else {
                term_counts.into_iter()
                    .max_by_key(|(_, c)| *c)
                    .map(|(term, _)| term.chars().next().unwrap().to_uppercase().to_string() + &term[1..])
                    .unwrap_or_else(|| format!("Community {}", comm))
            }
        } else {
            term_counts.into_iter()
                .max_by_key(|(_, c)| *c)
                .map(|(term, _)| term.chars().next().unwrap().to_uppercase().to_string() + &term[1..])
                .unwrap_or_else(|| format!("Community {}", comm))
        };

        labels.insert(comm, label);
    }

    labels
}

/// Generate suggested questions based on graph analysis
pub fn suggest_questions(graph: &GraphData, top_n: usize) -> Vec<SuggestedQuestion> {
    let mut questions = Vec::new();
    
    // 1. AMBIGUOUS edges
    for edge in &graph.links {
        if edge.confidence == Confidence::Ambiguous {
            let src_label = graph.nodes.iter()
                .find(|n| n.id == edge.source)
                .map(|n| n.label.as_str())
                .unwrap_or(&edge.source);
            let tgt_label = graph.nodes.iter()
                .find(|n| n.id == edge.target)
                .map(|n| n.label.as_str())
                .unwrap_or(&edge.target);
            
            questions.push(SuggestedQuestion {
                question_type: "ambiguous_edge".to_string(),
                question: format!("What is the exact relationship between `{}` and `{}`?", src_label, tgt_label),
                why: format!("Edge tagged AMBIGUOUS (relation: {}) - confidence is low.", edge.relation),
            });
        }
    }
    
    // 2. Bridge nodes
    let betweenness = calculate_betweenness(graph);
    let community_sizes = count_community_sizes(graph);
    
    for (node_id, score) in betweenness.iter().take(10) {
        let score_val = *score;
        if score_val == 0.0 { continue; }
        if is_file_node(graph, node_id) { continue; }
        
        let node = match graph.nodes.iter().find(|n| n.id == *node_id) {
            Some(n) => n,
            None => continue,
        };
        let label = &node.label;
        let cid = node.community;
        
        // Count neighbors in different communities
        let mut neighbor_comms = HashSet::new();
        for edge in &graph.links {
            if edge.source == *node_id {
                if let Some(n) = graph.nodes.iter().find(|n| n.id == edge.target) {
                    if let Some(c) = n.community {
                        neighbor_comms.insert(c);
                    }
                }
            } else if edge.target == *node_id {
                if let Some(n) = graph.nodes.iter().find(|n| n.id == edge.source) {
                    if let Some(c) = n.community {
                        neighbor_comms.insert(c);
                    }
                }
            }
        }
        
        if let Some(main_comm) = cid {
            neighbor_comms.remove(&main_comm);
        }
        
        if !neighbor_comms.is_empty() && score_val > 0.0 {
            let comm_labels = generate_community_labels(graph, &community_sizes);
            let main_label = cid.map(|c| comm_labels.get(&c).cloned().unwrap_or_else(|| format!("Community {}", c)))
                .unwrap_or_else(|| "unknown".to_string());
            let other_labels: Vec<_> = neighbor_comms.iter()
                .map(|c| comm_labels.get(c).cloned().unwrap_or_else(|| format!("Community {}", c)))
                .collect();
            
            questions.push(SuggestedQuestion {
                question_type: "bridge_node".to_string(),
                question: format!("Why does `{}` connect `{}` to {}?", label, main_label,
                    other_labels.iter().map(|l| format!("`{}`", l)).collect::<Vec<_>>().join(", ")),
                why: format!("High betweenness centrality ({}) - this node is a cross-community bridge.", score_val),
            });
        }
    }
    
    // 3. God nodes with many INFERRED edges
    let god_nodes = find_god_nodes(graph, 5);
    for god in god_nodes {
        let inferred_count = graph.links.iter()
            .filter(|e| {
                (e.source == god.node.id || e.target == god.node.id) 
                && e.confidence == Confidence::Inferred
            })
            .count();
        
        if inferred_count >= 2 {
            let inferred_edges: Vec<_> = graph.links.iter()
                .filter(|e| {
                    (e.source == god.node.id || e.target == god.node.id) 
                    && e.confidence == Confidence::Inferred
                })
                .take(2)
                .collect();
            
            let others: Vec<_> = inferred_edges.iter()
                .filter_map(|e| {
                    let other_id = if e.source == god.node.id { &e.target } else { &e.source };
                    graph.nodes.iter().find(|n| n.id == *other_id).map(|n| n.label.as_str())
                })
                .collect();
            
            if !others.is_empty() {
                questions.push(SuggestedQuestion {
                    question_type: "verify_inferred".to_string(),
                    question: format!("Are the {} inferred relationships involving `{}` (e.g. with `{}`) actually correct?", 
                        inferred_count, god.node.label, others.join("` and `")),
                    why: format!("`{}` has {} INFERRED edges - model-reasoned connections that need verification.", god.node.label, inferred_count),
                });
            }
        }
    }
    
    // 4. Isolated nodes
    let isolated: Vec<_> = graph.nodes.iter()
        .filter(|n| {
            let degree = graph.links.iter()
                .filter(|e| e.source == n.id || e.target == n.id)
                .count();
            !is_file_node(graph, &n.id) && degree <= 1
        })
        .take(3)
        .collect();
    
    if !isolated.is_empty() {
        let labels: Vec<_> = isolated.iter().map(|n| n.label.as_str()).collect();
        questions.push(SuggestedQuestion {
            question_type: "isolated_nodes".to_string(),
            question: format!("What connects {} to the rest of the system?",
                labels.iter().map(|l| format!("`{}`", l)).collect::<Vec<_>>().join(", ")),
            why: format!("{} weakly-connected nodes found - possible documentation gaps or missing edges.", isolated.len()),
        });
    }
    
    // 5. Low-cohesion communities
    let cohesion = calculate_cohesion_scores(graph);
    for (cid, &score) in &cohesion {
        let comm_size = *community_sizes.get(cid).unwrap_or(&0);
        if score < 0.15 && comm_size >= 5 {
            let labels = generate_community_labels(graph, &community_sizes);
            let label = labels.get(cid).cloned().unwrap_or_else(|| format!("Community {}", cid));
            
            questions.push(SuggestedQuestion {
                question_type: "low_cohesion".to_string(),
                question: format!("Should `{}` be split into smaller, more focused modules?", label),
                why: format!("Cohesion score {:.2} - nodes in this community are weakly interconnected.", score),
            });
        }
    }
    
    questions.truncate(top_n);
    
    if questions.is_empty() {
        questions.push(SuggestedQuestion {
            question_type: "no_signal".to_string(),
            question: String::new(),
            why: "Not enough signal to generate questions. This usually means the corpus has no AMBIGUOUS edges, no bridge nodes, no INFERRED relationships, and all communities are tightly cohesive.".to_string(),
        });
    }
    
    questions
}

/// Calculate betweenness centrality (simplified)
fn calculate_betweenness(graph: &GraphData) -> HashMap<String, f64> {
    let mut betweenness: HashMap<String, f64> = HashMap::new();
    
    for node in &graph.nodes {
        betweenness.insert(node.id.clone(), 0.0);
    }
    
    for source in &graph.nodes {
        let mut queue = vec![source.id.as_str()];
        let mut visited = HashSet::new();
        let mut paths: HashMap<&str, Vec<Vec<&str>>> = HashMap::new();
        let mut distance: HashMap<&str, usize> = HashMap::new();
        
        paths.insert(source.id.as_str(), vec![vec![source.id.as_str()]]);
        distance.insert(source.id.as_str(), 0);
        
        while let Some(current) = queue.pop() {
            if visited.contains(current) {
                continue;
            }
            visited.insert(current);
            
            for edge in &graph.links {
                let neighbor = if edge.source == current {
                    edge.target.as_str()
                } else if edge.target == current {
                    edge.source.as_str()
                } else {
                    continue;
                };
                
                if !visited.contains(neighbor) {
                    let current_dist = *distance.get(current).unwrap_or(&usize::MAX);
                    let new_dist = current_dist + 1;
                    
                    if !distance.contains_key(neighbor) || new_dist < *distance.get(neighbor).unwrap() {
                        distance.insert(neighbor, new_dist);
                    }
                    
                    if new_dist == current_dist + 1 {
                        // Collect new paths first to avoid borrow conflict
                        let mut new_paths_for_neighbor: Vec<Vec<&str>> = Vec::new();
                        if let Some(current_paths) = paths.get(current) {
                            for path in current_paths {
                                let mut new_path = path.clone();
                                new_path.push(neighbor);
                                new_paths_for_neighbor.push(new_path);
                            }
                        }
                        // Then insert them
                        for new_path in new_paths_for_neighbor {
                            paths.entry(neighbor).or_default().push(new_path);
                        }
                        queue.push(neighbor);
                    }
                }
            }
        }
        
        for (node, node_paths) in &paths {
            if **node == *source.id.as_str() {
                continue;
            }
            let count = node_paths.len() as f64;
            *betweenness.entry((*node).to_string()).or_insert(0.0) += count;
        }
    }
    
    let n = graph.nodes.len();
    if n > 2 {
        let factor = 2.0 / ((n - 1) * (n - 2)) as f64;
        for (_, score) in betweenness.iter_mut() {
            *score *= factor;
        }
    }
    
    betweenness
}

/// Compare two graph snapshots
pub fn graph_diff(old_graph: &GraphData, new_graph: &GraphData) -> GraphDiff {
    let old_node_ids: HashSet<_> = old_graph.nodes.iter().map(|n| n.id.clone()).collect();
    let new_node_ids: HashSet<_> = new_graph.nodes.iter().map(|n| n.id.clone()).collect();
    
    let added_ids: Vec<_> = new_node_ids.difference(&old_node_ids).cloned().collect();
    let removed_ids: Vec<_> = old_node_ids.difference(&new_node_ids).cloned().collect();
    
    let new_nodes_list: Vec<NodeChange> = added_ids.iter()
        .filter_map(|id| {
            new_graph.nodes.iter()
                .find(|n| n.id == *id)
                .map(|n| NodeChange { id: n.id.clone(), label: n.label.clone() })
        })
        .collect();
    
    let removed_nodes_list: Vec<NodeChange> = removed_ids.iter()
        .filter_map(|id| {
            old_graph.nodes.iter()
                .find(|n| n.id == *id)
                .map(|n| NodeChange { id: n.id.clone(), label: n.label.clone() })
        })
        .collect();
    
    // Edge diff
    let old_edge_keys: HashSet<_> = old_graph.links.iter()
        .map(|e| (e.source.as_str(), e.target.as_str(), e.relation.as_str()))
        .collect();
    let new_edge_keys: HashSet<_> = new_graph.links.iter()
        .map(|e| (e.source.as_str(), e.target.as_str(), e.relation.as_str()))
        .collect();
    
    let added_edge_keys: Vec<_> = new_edge_keys.difference(&old_edge_keys).collect();
    let removed_edge_keys: Vec<_> = old_edge_keys.difference(&new_edge_keys).collect();
    
    let new_edges_list: Vec<EdgeChange> = added_edge_keys.iter()
        .filter_map(|(src, tgt, rel)| {
            new_graph.links.iter()
                .find(|e| e.source.as_str() == *src && e.target.as_str() == *tgt && e.relation.as_str() == *rel)
                .map(|e| EdgeChange {
                    source: e.source.clone(),
                    target: e.target.clone(),
                    relation: e.relation.clone(),
                    confidence: format!("{:?}", e.confidence),
                })
        })
        .collect();
    
    let removed_edges_list: Vec<EdgeChange> = removed_edge_keys.iter()
        .filter_map(|(src, tgt, rel)| {
            old_graph.links.iter()
                .find(|e| e.source.as_str() == *src && e.target.as_str() == *tgt && e.relation.as_str() == *rel)
                .map(|e| EdgeChange {
                    source: e.source.clone(),
                    target: e.target.clone(),
                    relation: e.relation.clone(),
                    confidence: format!("{:?}", e.confidence),
                })
        })
        .collect();
    
    let mut parts = Vec::new();
    if !new_nodes_list.is_empty() {
        parts.push(format!("{} new node{}", new_nodes_list.len(), if new_nodes_list.len() != 1 { "s" } else { "" }));
    }
    if !new_edges_list.is_empty() {
        parts.push(format!("{} new edge{}", new_edges_list.len(), if new_edges_list.len() != 1 { "s" } else { "" }));
    }
    if !removed_nodes_list.is_empty() {
        parts.push(format!("{} node{} removed", removed_nodes_list.len(), if removed_nodes_list.len() != 1 { "s" } else { "" }));
    }
    if !removed_edges_list.is_empty() {
        parts.push(format!("{} edge{} removed", removed_edges_list.len(), if removed_edges_list.len() != 1 { "s" } else { "" }));
    }
    
    let summary = if parts.is_empty() {
        "no changes".to_string()
    } else {
        parts.join(", ")
    };
    
    GraphDiff {
        new_nodes: new_nodes_list,
        removed_nodes: removed_nodes_list,
        new_edges: new_edges_list,
        removed_edges: removed_edges_list,
        summary,
    }
}
