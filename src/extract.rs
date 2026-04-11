//! AST extraction module using tree-sitter

use std::collections::HashMap;
use std::path::Path;
use tree_sitter::{Parser, Node as TsNode};
use tree_sitter_language_pack::get_language;
use crate::types::{Node, Edge, Confidence, ExtractionResult};

/// Context cho extraction
pub struct ExtractContext {
    pub file_path: String,
    pub file_stem: String,
    pub known_nodes: HashMap<String, bool>,
}

impl ExtractContext {
    /// Create new context
    pub fn new(file_path: &str) -> Self {
        let path = Path::new(file_path);
        let file_stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();
        
        Self {
            file_path: file_path.to_string(),
            file_stem,
            known_nodes: HashMap::new(),
        }
    }
    
    /// Add a known node
    pub fn add_known_node(&mut self, node_id: String) {
        self.known_nodes.insert(node_id, true);
    }
    
    /// Check if node is known
    pub fn is_known(&self, node_id: &str) -> bool {
        self.known_nodes.contains_key(node_id)
    }
}

/// Extract from a file
pub fn extract_file(path: &Path, source: &str) -> anyhow::Result<ExtractionResult> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    
    let lang = match ext.as_str() {
        "py" | "pyi" | "pyw" => "python",
        "js" | "mjs" | "cjs" | "jsx" => "javascript",
        "ts" | "tsx" => "typescript",
        "go" => "go",
        "rs" => "rust",
        "java" => "java",
        "c" => "c",
        "cpp" | "cc" | "cxx" | "hpp" | "hxx" => "cpp",
        "rb" => "ruby",
        "cs" => "csharp",
        "kt" | "kts" => "kotlin",
        "scala" => "scala",
        "php" => "php",
        "swift" => "swift",
        "lua" => "lua",
        "zig" => "zig",
        _ => {
            // Try to find language by extension
            if let Some(lang) = find_language_by_ext(&ext) {
                lang
            } else {
                return simple_extract(source, path.to_string_lossy().as_ref());
            }
        }
    };
    
    extract_with_language(source, path.to_string_lossy().as_ref(), lang)
}

/// Find language by extension
fn find_language_by_ext(ext: &str) -> Option<&'static str> {
    // Map common extensions to language names
    let mapping: HashMap<&str, &str> = [
        ("py", "python"),
        ("rs", "rust"),
        ("js", "javascript"),
        ("ts", "typescript"),
        ("go", "go"),
        ("java", "java"),
        ("c", "c"),
        ("cpp", "cpp"),
        ("rb", "ruby"),
        ("cs", "csharp"),
        ("swift", "swift"),
        ("kt", "kotlin"),
        ("scala", "scala"),
        ("php", "php"),
        ("lua", "lua"),
        ("zig", "zig"),
    ].into_iter().collect();
    
    mapping.get(ext).copied()
}

/// Extract using specific language
fn extract_with_language(source: &str, file_path: &str, language: &str) -> anyhow::Result<ExtractionResult> {
    let mut ctx = ExtractContext::new(file_path);
    let mut result = ExtractionResult::new();
    let source_bytes = source.as_bytes();
    
    // Get language parser
    let lang = match get_language(language) {
        Ok(l) => l,
        Err(e) => {
            // Language not supported
            eprintln!("Language '{}' not supported: {:?}", language, e);
            return simple_extract(source, file_path);
        }
    };
    
    // Parse
    let mut parser = Parser::new();
    
    if let Err(e) = parser.set_language(&lang.into()) {
        eprintln!("Failed to set language {}: {}", language, e);
        return simple_extract(source, file_path);
    }
    
    let tree = match parser.parse(source, None) {
        Some(t) => t,
        None => {
            return simple_extract(source, file_path);
        }
    };
    
    let root = tree.root_node();
    
    // Pass 1: Collect all nodes
    let node_count_before = result.nodes.len();
    walk_tree_pass1(&root, &mut ctx, &mut result, source_bytes);
    if result.nodes.len() > node_count_before {
        eprintln!("  Found {} new definitions in {}", result.nodes.len() - node_count_before, file_path);
    }
    
    // Pass 2: Build edges
    walk_tree_pass2(&root, &mut ctx, &mut result, source_bytes);
    
    Ok(result)
}

/// Simple extraction for unsupported languages
fn simple_extract(source: &str, file_path: &str) -> anyhow::Result<ExtractionResult> {
    let mut result = ExtractionResult::new();
    let path = Path::new(file_path);
    let file_stem = path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");
    
    // Simple heuristic: find function-like patterns
    let patterns = [
        ("fn ", "function"),
        ("func ", "function"),
        ("def ", "function"),
        ("class ", "class"),
        ("struct ", "struct"),
        ("interface ", "interface"),
        ("pub fn", "function"),
        ("pub func", "function"),
    ];
    
    for (i, line) in source.lines().enumerate() {
        for (pattern, _node_type) in &patterns {
            if line.contains(pattern) {
                // Try to extract name
                let name = extract_name_from_line(line, pattern);
                if !name.is_empty() {
                    let node_id = format!("{}:{}", file_stem, name);
                    result.add_node(Node::new(
                        node_id,
                        name,
                        file_stem.to_string(),
                        format!("L{}", i + 1),
                    ));
                }
            }
        }
    }
    
    Ok(result)
}

/// Extract name from line
fn extract_name_from_line(line: &str, keyword: &str) -> String {
    if let Some(pos) = line.find(keyword) {
        let after_keyword = &line[pos + keyword.len()..];
        // Find the name (usually until { or ( or ;)
        let name: String = after_keyword
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        name
    } else {
        String::new()
    }
}

/// Pass 1: Collect all nodes
fn walk_tree_pass1(node: &TsNode, ctx: &mut ExtractContext, result: &mut ExtractionResult, source: &[u8]) {
    let kind = node.kind();
    
    // Common definition kinds for different languages
    let definition_kinds = [
        "function_definition",
        "class_definition", 
        "method_definition",
        "module",
        "module_clause",
        "import_statement",
        "interface_declaration",
        "struct_declaration",
        "enum_declaration",
        "type_declaration",
        "function_declaration",
    ];
    
    if definition_kinds.contains(&kind) {
        let dbg_name = get_definition_name(node, source);
        if let Some(name) = dbg_name {
            // Skip if name is "identifier" (meaning we failed to extract real name)
            if name == "identifier" || name.len() < 2 {
                eprintln!("DEBUG: Skipping '{}' kind node (got name '{}')", kind, name);
                return;
            }
            
            let node_id = format!("{}:{}", ctx.file_stem, name);
            ctx.add_known_node(node_id.clone());
            
            result.add_node(Node::new(
                node_id,
                name,
                ctx.file_path.clone(),
                format!("L{}", node.start_position().row + 1),
            ));
        }
    }
    
    // Recurse using children
    let child_count = node.child_count();
    for i in 0..child_count {
        if let Ok(idx) = u32::try_from(i) {
            if let Some(child) = node.child(idx) {
                walk_tree_pass1(&child, ctx, result, source);
            }
        }
    }
}

/// Pass 2: Build edges
fn walk_tree_pass2(
    node: &TsNode,
    ctx: &mut ExtractContext,
    result: &mut ExtractionResult,
    source: &[u8],
) {
    let kind = node.kind();
    
    // Call expressions - look for function calls
    if kind == "call_expression" || kind == "call" || kind == "invocation" {
        if let Some((caller, callee)) = extract_call(node, ctx, source) {
            let confidence = if ctx.is_known(&callee) {
                Confidence::Extracted
            } else {
                Confidence::Inferred
            };
            
            result.add_edge(Edge::new(
                caller,
                callee,
                "calls".to_string(),
                confidence,
            ));
        }
    }
    
    // Recurse
    let child_count = node.child_count();
    for i in 0..child_count {
        if let Ok(idx) = u32::try_from(i) {
            if let Some(child) = node.child(idx) {
                walk_tree_pass2(&child, ctx, result, source);
            }
        }
    }
}

/// Get definition name - extract actual text from node
fn get_definition_name(node: &TsNode, source: &[u8]) -> Option<String> {
    // Try different field names for different languages
    let field_names = ["name", "identifier", "function", "declarator"];
    
    for field in &field_names {
        if let Some(name_node) = node.child_by_field_name(field) {
            let node_kind = name_node.kind();
            // Try to get text from the field node directly
            if let Ok(text) = name_node.utf8_text(source) {
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    return Some(trimmed.to_string());
                }
            }
            // If the field node itself is an identifier, get its text
            if node_kind == "identifier" || node_kind == "name" {
                // The identifier node should have its text
                let start = name_node.start_byte();
                let end = name_node.end_byte();
                if start < end && end <= source.len() {
                    if let Ok(text) = std::str::from_utf8(&source[start..end]) {
                        let trimmed = text.trim();
                        if !trimmed.is_empty() {
                            return Some(trimmed.to_string());
                        }
                    }
                }
            }
        }
    }
    
    // Fall back: get first identifier child
    let child_count = node.child_count();
    for i in 0..child_count {
        if let Ok(idx) = u32::try_from(i) {
            if let Some(child) = node.child(idx) {
                let child_kind = child.kind();
                // Look for identifier or name nodes
                if child_kind == "identifier" || child_kind == "name" || child_kind == "string" {
                    if let Ok(text) = child.utf8_text(source) {
                        let trimmed = text.trim_matches('"').trim();
                        if !trimmed.is_empty() {
                            return Some(trimmed.to_string());
                        }
                    }
                }
                // Also look for dotted names like "os.path.join"
                if child_kind == "dotted_name" || child_kind == "attribute" {
                    if let Ok(text) = child.utf8_text(source) {
                        let trimmed = text.trim();
                        if !trimmed.is_empty() {
                            return Some(trimmed.to_string());
                        }
                    }
                }
            }
        }
    }
    
    // Last resort: use node's own text
    if let Ok(text) = node.utf8_text(source) {
        let trimmed = text.trim();
        // Make sure it's not too long
        if !trimmed.is_empty() && trimmed.len() < 100 {
            // Try to extract just the first identifier-like part
            let first_word: String = trimmed.chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if !first_word.is_empty() && first_word.chars().next().map(|c| c.is_alphabetic()).unwrap_or(false) {
                return Some(first_word);
            }
        }
    }
    
    None
}

/// Extract call information - find function being called
fn extract_call(node: &TsNode, ctx: &ExtractContext, source: &[u8]) -> Option<(String, String)> {
    // Look for the function being called - usually first child that's an identifier
    let child_count = node.child_count();
    let mut func_name: Option<String> = None;
    
    for i in 0..child_count {
        if let Ok(idx) = u32::try_from(i) {
            if let Some(child) = node.child(idx) {
                let kind = child.kind();
                
                // Primary: identifier directly
                if kind == "identifier" || kind == "name" {
                    if let Ok(text) = child.utf8_text(source) {
                        let trimmed = text.trim();
                        if !trimmed.is_empty() {
                            func_name = Some(trimmed.to_string());
                            break;
                        }
                    }
                }
                
                // Secondary: dotted_name like "os.path.join"
                if kind == "dotted_name" || kind == "attribute" {
                    if let Ok(text) = child.utf8_text(source) {
                        let trimmed = text.trim();
                        if !trimmed.is_empty() {
                            func_name = Some(trimmed.to_string());
                            break;
                        }
                    }
                }
            }
        }
    }
    
    // If we found a function name, get the caller
    if let Some(name) = func_name {
        if name.is_empty() || name == "_" {
            return None;
        }
        
        // Get enclosing function/class
        let caller = get_enclosing_name_with_source(node, source)
            .unwrap_or_else(|| "global".to_string());
        
        let callee = format!("{}:{}", ctx.file_stem, name);
        let caller_full = format!("{}:{}", ctx.file_stem, caller);
        
        return Some((caller_full, callee));
    }
    
    None
}

/// Get enclosing definition name (with source for text extraction)
fn get_enclosing_name_with_source(node: &TsNode, source: &[u8]) -> Option<String> {
    let mut current = node.clone();
    
    while let Some(parent) = current.parent() {
        let kind = parent.kind();
        if kind == "function_definition" || kind == "method_definition" 
            || kind == "class_definition" || kind == "function_declaration"
            || kind == "procedure_definition" {
            return get_definition_name(&parent, source);
        }
        current = parent;
    }
    
    None
}

/// Extract multiple files in parallel
pub fn extract_files(paths: &[std::path::PathBuf]) -> Vec<ExtractionResult> {
    use rayon::prelude::*;
    
    paths
        .par_iter()
        .filter_map(|path| {
            let source = std::fs::read_to_string(path).ok()?;
            extract_file(path, &source).ok()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_python() {
        let source = r#"
def hello():
    pass

class World:
    def greet(self):
        pass
"#;
        
        let result = simple_extract(source, "test.py").unwrap();
        
        // Should find hello and World
        assert!(result.nodes.len() >= 2);
    }

    #[test]
    fn test_extract_name() {
        assert_eq!(extract_name_from_line("def hello():", "def "), "hello");
        assert_eq!(extract_name_from_line("fn world()", "fn "), "world");
        assert_eq!(extract_name_from_line("class Foo:", "class "), "Foo");
    }
}
