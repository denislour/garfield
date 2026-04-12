//! 3-Tier Lazy Loading for Garfield
//!
//! TIER 1: Metadata (nodes, edges, communities) - ALWAYS in memory
//! TIER 2: File Summaries - Cached in file_summaries.json
//! TIER 3: Full Body - On-demand from source files

use crate::types::{FileSummary, GraphData, Node};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Generate file summaries from graph nodes
pub fn generate_file_summaries(graph: &GraphData) -> HashMap<String, FileSummary> {
    let mut summaries: HashMap<String, FileSummary> = HashMap::new();

    // Group nodes by source file
    let mut nodes_by_file: HashMap<String, Vec<&Node>> = HashMap::new();
    for node in &graph.nodes {
        nodes_by_file
            .entry(node.source_file.clone())
            .or_default()
            .push(node);
    }

    // Generate summary for each file
    for (file, nodes) in nodes_by_file {
        if nodes.is_empty() {
            continue;
        }

        // Detect functions by naming convention (snake_case or PascalCase)
        let functions: Vec<String> = nodes
            .iter()
            .filter(|n| is_function_like(&n.label))
            .map(|n| {
                if let Some(summary) = &n.summary {
                    format!("{} - {}", n.label, summary)
                } else {
                    n.label.clone()
                }
            })
            .collect();

        let public_apis: Vec<String> = nodes
            .iter()
            .filter(|n| n.label.chars().next().map(|c| c.is_uppercase()).unwrap_or(false))
            .map(|n| n.label.clone())
            .collect();

        let internal_functions: Vec<String> = nodes
            .iter()
            .filter(|n| is_function_like(&n.label) && n.label.chars().next().map(|c| c.is_lowercase()).unwrap_or(false))
            .map(|n| n.label.clone())
            .collect();

        // Generate file-level summary
        let summary = generate_file_level_summary(&nodes);

        let file_summary = FileSummary {
            filename: file.clone(),
            summary,
            function_count: functions.len(),
            functions,
            public_apis,
            dependencies: Vec::new(), // Will be populated from edges
            internal_functions,
            doc_comment: None,
        };

        summaries.insert(file, file_summary);
    }

    summaries
}

/// Check if a label looks like a function
fn is_function_like(label: &str) -> bool {
    // Skip short labels that might be variables
    if label.len() < 3 {
        return false;
    }
    
    // Skip labels with colons (likely qualified names like "Some", "Ok")
    if label.contains(':') {
        return false;
    }
    
    // Skip common type names
    let skip = ["String", "Vec", "HashMap", "Option", "Result", "bool", "i32", "i64", "f64", "usize"];
    if skip.contains(&label) {
        return false;
    }
    
    // Functions are typically snake_case or camelCase with multiple parts
    label.chars().any(|c| c == '_') || 
    (label.len() > 6 && label.chars().any(|c| c.is_uppercase()))
}

/// Generate a file-level summary based on nodes
fn generate_file_level_summary(nodes: &[&Node]) -> String {
    let function_count = nodes
        .iter()
        .filter(|n| n.node_type.as_deref() == Some("function") || n.node_type.as_deref() == Some("method"))
        .count();

    let mut topics: Vec<String> = Vec::new();

    for node in nodes {
        // Extract keywords from labels
        let label_lower = node.label.to_lowercase();
        if label_lower.contains("test") {
            topics.push("testing".to_string());
        }
        if label_lower.contains("config") || label_lower.contains("settings") {
            topics.push("configuration".to_string());
        }
        if label_lower.contains("handler") || label_lower.contains("controller") {
            topics.push("request handling".to_string());
        }
        if label_lower.contains("model") || label_lower.contains("entity") {
            topics.push("data modeling".to_string());
        }
        if label_lower.contains("service") {
            topics.push("business logic".to_string());
        }
        if label_lower.contains("db") || label_lower.contains("storage") || label_lower.contains("cache") {
            topics.push("data storage".to_string());
        }
        if label_lower.contains("auth") || label_lower.contains("login") || label_lower.contains("session") {
            topics.push("authentication".to_string());
        }
        if label_lower.contains("api") || label_lower.contains("endpoint") {
            topics.push("API".to_string());
        }
    }

    // Deduplicate topics
    topics.sort();
    topics.dedup();

    let topic_str = if topics.is_empty() {
        "general".to_string()
    } else {
        topics.join(", ")
    };

    format!("{} with {} functions - {}", topic_str, function_count, "code")
}

/// Save file summaries to JSON
pub fn save_file_summaries(
    summaries: &HashMap<String, FileSummary>,
    path: &Path,
) -> std::io::Result<()> {
    let json = serde_json::to_string_pretty(summaries)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    fs::write(path, json)
}

/// Load file summaries from JSON
pub fn load_file_summaries(path: &Path) -> std::io::Result<HashMap<String, FileSummary>> {
    let content = fs::read_to_string(path)?;
    serde_json::from_str(&content)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

/// Get summary for a specific file
pub fn get_file_summary<'a>(
    filename: &'a str,
    summaries: &'a HashMap<String, FileSummary>,
) -> Option<&'a FileSummary> {
    summaries.get(filename)
}

/// Get node body (TIER 3) - on-demand from source files
pub fn get_node_body(node: &Node) -> Option<String> {
    let source_path = Path::new(&node.source_file);
    if !source_path.exists() {
        return None;
    }

    // Read the source file
    let content = match fs::read_to_string(source_path) {
        Ok(c) => c,
        Err(_) => return None,
    };

    // For now, return the entire file if location is not specific
    // TODO: Implement precise function extraction using tree-sitter
    if node.source_location.is_empty() || node.source_location == "L1" {
        return Some(content);
    }

    // Try to extract function from source
    extract_function_body(&content, &node.label, &node.source_location)
}

/// Extract function body from source code
fn extract_function_body(content: &str, fn_name: &str, location: &str) -> Option<String> {
    // Simple heuristic: find the function declaration and extract until next function or end
    let lines: Vec<&str> = content.lines().collect();

    // Parse location (e.g., "L42" -> line 42)
    let start_line = location
        .trim_start_matches('L')
        .parse::<usize>()
        .ok()?
        .saturating_sub(1);

    if start_line >= lines.len() {
        return None;
    }

    // Find function start
    let mut fn_start = start_line;
    while fn_start > 0 && !lines[fn_start].contains(&format!("fn {}", fn_name))
        && !lines[fn_start].contains(&format!("def {}", fn_name))
    {
        fn_start -= 1;
    }

    // Find function end (matching brace)
    let mut brace_count = 0;
    let mut in_function = false;
    let mut fn_end = fn_start;

    for i in fn_start..lines.len() {
        let line = lines[i];
        for c in line.chars() {
            if c == '{' || c == '(' {
                brace_count += 1;
                in_function = true;
            } else if c == '}' || c == ')' {
                brace_count -= 1;
            }
        }
        if in_function && brace_count <= 0 {
            fn_end = i;
            break;
        }
        fn_end = i;
    }

    // Extract function body
    let body: String = lines[fn_start..=fn_end.min(lines.len() - 1)]
        .join("\n");

    Some(body)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_file_summaries() {
        let nodes = vec![
            Node {
                id: "test.rs:calculate_price".into(),
                label: "calculate_price".into(),
                source_file: "test.rs".into(),
                source_location: "L1".into(),
                community: Some(0),
                node_type: Some("function".into()),
                file_type: None,
                file_stem: None,
                summary: Some("First function".into()),
            },
            Node {
                id: "test.rs:process_order".into(),
                label: "process_order".into(),
                source_file: "test.rs".into(),
                source_location: "L10".into(),
                community: Some(0),
                node_type: Some("function".into()),
                file_type: None,
                file_stem: None,
                summary: Some("Second function".into()),
            },
        ];

        let graph = GraphData::new(nodes, vec![], 1);
        let summaries = generate_file_summaries(&graph);

        assert!(summaries.contains_key("test.rs"));
        let summary = summaries.get("test.rs").unwrap();
        assert_eq!(summary.function_count, 2);
        assert_eq!(summary.functions.len(), 2);
    }

    #[test]
    fn test_get_file_summary() {
        let mut summaries = HashMap::new();
        summaries.insert(
            "test.rs".into(),
            FileSummary::new("test.rs".into(), "Test file".into()),
        );

        let summary = get_file_summary("test.rs", &summaries);
        assert!(summary.is_some());
    }
}
