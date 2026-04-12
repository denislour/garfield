//! AST extraction module using tree-sitter
//!
//! ## Extract Flow (3-pass approach matching Graphify)
//!
//! ### Pass 1: Node Collection
//! - Collects all definitions (functions, classes, modules, structs, enums)
//! - Creates node entries with ID format: `{file_stem}:{name}`
//!
//! ### Pass 2: Edge Building  
//! - Builds edges from call expressions (calls relationship)
//! - Extracts import statements (imports/imports_from relationships)
//!
//! ### Pass 3: Rationale Detection (Python only)
//! - Extracts docstrings and rationale comments (# NOTE:, # WHY:, etc.)
//! - Creates "rationale_for" edges connecting rationale to code entities
//! - This is Python-specific and requires docstring parsing

use crate::lang::{get_ts_language, all_definition_kinds};
use crate::types::{Confidence, Edge, ExtractionResult, Node};
use std::collections::HashMap;
use std::path::Path;
use tree_sitter::{Node as TsNode, Parser};
use std::sync::LazyLock;

/// All definition kinds loaded from lang.rs (dynamic per language config)
static DEFINITION_KINDS: LazyLock<Vec<&'static str>> = LazyLock::new(all_definition_kinds);

/// Context cho extraction
pub struct ExtractContext {
    pub file_path: String,
    pub file_stem: String,
    /// Pass 1: Track all known node IDs (dedup Layer 1)
    pub known_nodes: HashMap<String, bool>,
    /// Rationale nodes seen (for Pass 3)
    pub rationale_seen: HashMap<String, bool>,
    /// Source bytes for text extraction
    pub source: Vec<u8>,
}

impl ExtractContext {
    /// Create new context
    pub fn new(file_path: &str, source: &[u8]) -> Self {
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
            rationale_seen: HashMap::new(),
            source: source.to_vec(),
        }
    }

    /// Add a known node
    pub fn add_known_node(&mut self, node_id: String) {
        self.known_nodes.insert(node_id.clone(), true);
    }

    /// Check if node is known
    pub fn is_known(&self, node_id: &str) -> bool {
        self.known_nodes.contains_key(node_id)
    }

    /// Check if rationale already exists
    pub fn has_rationale(&self, key: &str) -> bool {
        self.rationale_seen.contains_key(key)
    }

    /// Add rationale
    pub fn add_rationale(&mut self, key: String) {
        self.rationale_seen.insert(key, true);
    }
}

/// Extract from a file
pub fn extract_file(path: &Path, source: &str) -> anyhow::Result<ExtractionResult> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    // Use centralized language config from lang.rs
    let lang = match crate::lang::get_extension_lang(&ext) {
        Some(l) => l,
        None => {
            return simple_extract(source, path.to_string_lossy().as_ref());
        }
    };

    extract_with_language(source, path.to_string_lossy().as_ref(), lang)
}

/// Extract using specific language
fn extract_with_language(
    source: &str,
    file_path: &str,
    language: &str,
) -> anyhow::Result<ExtractionResult> {
    let source_bytes = source.as_bytes().to_vec();
    let mut ctx = ExtractContext::new(file_path, &source_bytes);

    let mut parser = Parser::new();

    // Use language pack to get tree-sitter Language
    let lang = match get_ts_language(language) {
        Some(l) => l,
        None => {
            eprintln!("Language '{}' not supported", language);
            return simple_extract(source, file_path);
        }
    };

    if let Err(e) = parser.set_language(&lang.into()) {
        eprintln!("Failed to set language {}: {}", language, e);
        return simple_extract(source, file_path);
    }

    extract_with_parser(source, file_path, &mut parser, &mut ctx, &source_bytes)
}

/// Extract using a pre-configured parser
fn extract_with_parser(
    source: &str,
    file_path: &str,
    parser: &mut Parser,
    ctx: &mut ExtractContext,
    source_bytes: &[u8],
) -> anyhow::Result<ExtractionResult> {
    let mut result = ExtractionResult::new();

    let tree = match parser.parse(source, None) {
        Some(t) => t,
        None => {
            return simple_extract(source, file_path);
        }
    };

    let root = tree.root_node();

    // ============================================================
    // PASS 1: Collect all nodes (definitions)
    // ============================================================
    let node_count_before = result.nodes.len();
    walk_tree_pass1(&root, ctx, &mut result, source_bytes);
    if result.nodes.len() > node_count_before {
        eprintln!(
            "  Found {} new definitions in {}",
            result.nodes.len() - node_count_before,
            file_path
        );
    }

    // ============================================================
    // PASS 2: Build edges (calls, imports)
    // ============================================================
    walk_tree_pass2(&root, ctx, &mut result, source_bytes);

    // ============================================================
    // PASS 3: Rationale Detection (Python only)
    // Extract docstrings and # NOTE: / # WHY: comments
    // ============================================================
    let ext = Path::new(file_path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    
    if ext == "py" {
        extract_python_rationale(&root, ctx, &mut result, source_bytes);
    }

    Ok(result)
}

/// Rationale prefixes to look for (matching Graphify)
const RATIONALE_PREFIXES: &[&str] = &[
    // Python/Shell style
    "# NOTE:",
    "# IMPORTANT:",
    "# HACK:",
    "# WHY:",
    "# RATIONALE:",
    "# TODO:",
    "# FIXME:",
    // C/C++/Java/JavaScript/Go style
    "// NOTE:",
    "// IMPORTANT:",
    "// HACK:",
    "// WHY:",
    "// RATIONALE:",
    "// TODO:",
    "// FIXME:",
    // Block comment style
    "/* NOTE:",
    "/* IMPORTANT:",
    "/* HACK:",
    "/* WHY:",
    "/* RATIONALE:",
    "/* TODO:",
    "/* FIXME:",
];

/// PASS 3: Extract Python docstrings and rationale comments
/// 
/// This pass extracts:
/// 1. Module-level docstrings
/// 2. Class docstrings  
/// 3. Function docstrings
/// 4. Rationale comments (# NOTE:, # WHY:, etc.)
/// 
/// Each rationale creates:
/// - A "rationale" node with file_type = "rationale"
/// - A "rationale_for" edge connecting it to the parent entity
fn extract_python_rationale(
    node: &TsNode,
    ctx: &mut ExtractContext,
    result: &mut ExtractionResult,
    source: &[u8],
) {
    let source_str = String::from_utf8_lossy(source);
    let lines: Vec<&str> = source_str.lines().collect();
    
    // Process rationale comments line by line
    for (lineno, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        
        for prefix in RATIONALE_PREFIXES {
            if trimmed.starts_with(prefix) {
                let rationale_text = trimmed.trim_start_matches(prefix).trim();
                if rationale_text.len() > 5 {
                    // Create rationale node ID
                    let rationale_key = format!("{}_rationale_{}", ctx.file_stem, lineno + 1);
                    
                    if !ctx.has_rationale(&rationale_key) {
                        ctx.add_rationale(rationale_key.clone());
                        
                        // Truncate long rationale text
                        let truncated = if rationale_text.len() > 200 {
                            format!("{}...", &rationale_text[..197])
                        } else {
                            rationale_text.to_string()
                        };
                        
                        // Add rationale node
                        let rationale_node = Node {
                            id: rationale_key.clone(),
                            label: truncated.clone(),
                            file_type: Some(crate::types::FileType::Rationale),
                            source_file: ctx.file_path.clone(),
                            source_location: format!("L{}", lineno + 1),
                            community: None,
                            node_type: Some("rationale".to_string()),
                            file_stem: Some(ctx.file_stem.clone()),
                        };
                        
                        // Connect to file node (module-level rationale)
                        let file_node_id = ctx.file_stem.clone();
                        result.add_node(rationale_node);
                        result.add_edge(Edge::new(
                            rationale_key,
                            file_node_id,
                            "rationale_for".to_string(),
                            Confidence::Extracted,
                        ));
                    }
                }
                break;
            }
        }
    }
    
    // Extract docstrings by walking the AST
    extract_docstrings(node, ctx, result, source);
}

/// Extract docstrings from class and function definitions
fn extract_docstrings(
    node: &TsNode,
    ctx: &mut ExtractContext,
    result: &mut ExtractionResult,
    source: &[u8],
) {
    let kind = node.kind();
    
    // Handle class definitions
    if kind == "class_definition" {
        // Check for class docstring
        let body = node.child_by_field_name("body");
        let name = node.child_by_field_name("name")
            .and_then(|n| extract_text(&n, source));
        if let (Some(name_str), Some(body_node)) = (name, body) {
            if let Some((docstring, line)) = extract_docstring_from_body(&body_node, source) {
                create_rationale_node(ctx, result, &docstring, line, &name_str, "class");
            }
        }
    }
    // Handle function definitions  
    else if kind == "function_definition" {
        let body = node.child_by_field_name("body");
        let name = node.child_by_field_name("name")
            .and_then(|n| extract_text(&n, source));
        if let (Some(name_str), Some(body_node)) = (name, body) {
            if let Some((docstring, line)) = extract_docstring_from_body(&body_node, source) {
                create_rationale_node(ctx, result, &docstring, line, &name_str, "function");
            }
        }
    }
    
    // Recurse into children
    let child_count = node.child_count();
    for i in 0..child_count {
        if let Ok(idx) = u32::try_from(i) {
            if let Some(child) = node.child(idx) {
                extract_docstrings(&child, ctx, result, source);
            }
        }
    }
}

/// Get class name and body node (for backward compatibility)
#[allow(dead_code)]
/// Extract docstring from a body node
fn extract_docstring_from_body(body: &TsNode, source: &[u8]) -> Option<(String, usize)> {
    let mut cursor = body.walk();
    for child in body.children(&mut cursor) {
        // Looking for expression_statement containing a string
        if child.kind() == "expression_statement" {
            let mut expr_cursor = child.walk();
            for expr_child in child.children(&mut expr_cursor) {
                if expr_child.kind() == "string" {
                    if let Ok(text) = expr_child.utf8_text(source) {
                        // Strip quotes
                        let trimmed = text
                            .trim_matches('"')
                            .trim_matches('\'')
                            .trim();
                        
                        // Check for triple quotes
                        let final_text = if trimmed.starts_with("\"\"\"") || trimmed.starts_with("'''") {
                            let quote_len = 3;
                            let end_quote_len = if trimmed.ends_with("\"\"\"") || trimmed.ends_with("'''") { 3 } else { 0 };
                            if trimmed.len() > quote_len * 2 + end_quote_len {
                                trimmed[quote_len..trimmed.len()-end_quote_len].to_string()
                            } else {
                                trimmed.to_string()
                            }
                        } else {
                            trimmed.to_string()
                        };
                        
                        // Only consider meaningful docstrings (>20 chars)
                        if final_text.len() > 20 {
                            let line = expr_child.start_position().row + 1;
                            return Some((final_text, line));
                        }
                    }
                }
            }
            // Found first expression statement but not a string - no docstring
            break;
        }
        // If we hit other statements, stop looking for docstring
        if child.kind() != "comment" {
            break;
        }
    }
    None
}

/// Create a rationale node for a docstring
fn create_rationale_node(
    ctx: &mut ExtractContext,
    result: &mut ExtractionResult,
    docstring: &str,
    line: usize,
    entity_name: &str,
    entity_type: &str,
) {
    // Create unique rationale ID
    let rationale_key = format!("{}_docstring_{}_{}", ctx.file_stem, entity_type, line);
    
    if !ctx.has_rationale(&rationale_key) {
        ctx.add_rationale(rationale_key.clone());
        
        // Truncate long docstrings
        let truncated = if docstring.len() > 200 {
            format!("{}...", &docstring[..197])
        } else {
            docstring.to_string()
        };
        
        // Add rationale node
        let rationale_node = Node {
            id: rationale_key.clone(),
            label: truncated.clone(),
            file_type: Some(crate::types::FileType::Rationale),
            source_file: ctx.file_path.clone(),
            source_location: format!("L{}", line),
            community: None,
            node_type: Some("docstring".to_string()),
            file_stem: Some(ctx.file_stem.clone()),
        };
        
        // Connect to the entity (file_stem:name for classes/functions)
        let target_id = format!("{}:{}", ctx.file_stem, entity_name);
        
        result.add_node(rationale_node);
        result.add_edge(Edge::new(
            rationale_key,
            target_id,
            "rationale_for".to_string(),
            Confidence::Extracted,
        ));
    }
}

/// Simple extraction for unsupported languages
fn simple_extract(source: &str, file_path: &str) -> anyhow::Result<ExtractionResult> {
    let mut result = ExtractionResult::new();
    let path = Path::new(file_path);
    let file_stem = path
        .file_stem()
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

/// Extract text from node
fn extract_text(node: &TsNode, source: &[u8]) -> Option<String> {
    node.utf8_text(source)
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// PASS 1: Collect all nodes (definitions)
/// 
/// Walks the AST and collects:
/// - function_definition → function node
/// - class_definition → class node
/// - method_definition → method node
/// - module → module node
/// - struct_declaration → struct node
/// - enum_declaration → enum node
/// 
/// Also creates edges for imports during this pass.
fn walk_tree_pass1(
    node: &TsNode,
    ctx: &mut ExtractContext,
    result: &mut ExtractionResult,
    source: &[u8],
) {
    let kind = node.kind();

    // Use definition kinds from lang.rs (dynamic - loaded from language configs)
    if DEFINITION_KINDS.contains(&kind) {
        let name = get_definition_name(node, source);
        if let Some(name) = name {
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

    // Handle imports - create edges from file to imported modules
    extract_import(node, ctx, result, source);

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

/// Extract import statements and create edges
fn extract_import(
    node: &TsNode,
    ctx: &mut ExtractContext,
    result: &mut ExtractionResult,
    source: &[u8],
) {
    let kind = node.kind();
    let file_node_id = ctx.file_stem.clone();

    // import_statement (Python, JavaScript, TypeScript)
    if kind == "import_statement" || kind == "import_from_statement" {
        let child_count = node.child_count();
        for i in 0..child_count {
            if let Some(child) = node.child(i as u32) {
                let child_kind = child.kind();
                // Python: dotted_name, identifier, module_name
                // JS/TS: string (e.g., "react" or "./module")
                if child_kind == "dotted_name"
                    || child_kind == "identifier"
                    || child_kind == "module_name"
                    || child_kind == "string"
                {
                    if let Ok(text) = child.utf8_text(source) {
                        // JS/TS: trim quotes and relative paths
                        let module = text
                            .trim()
                            .trim_matches('"')
                            .trim_matches('\'')
                            .trim_start_matches("./")
                            .trim_start_matches("../")
                            .trim_start_matches('.');
                        if !module.is_empty() && module != "*" {
                            let target_id = make_import_node_id(module);
                            let relation = if kind == "import_from_statement" {
                                "imports_from"
                            } else {
                                "imports"
                            };
                            result.add_edge(Edge::new(
                                file_node_id.clone(),
                                target_id,
                                relation.to_string(),
                                Confidence::Extracted,
                            ));
                        }
                    }
                }
            }
        }
    }
    // Java/Swift: import_declaration
    else if kind == "import_declaration" {
        // Java: identifier || scoped_identifier
        // Swift: identifier
        let child_count = node.child_count();
        for i in 0..child_count {
            if let Some(child) = node.child(i as u32) {
                let child_kind = child.kind();
                if child_kind == "identifier" || child_kind == "scoped_identifier" {
                    if let Ok(text) = child.utf8_text(source) {
                        let imported = text.trim();
                        if !imported.is_empty() && imported != "*" {
                            let target_id = make_import_node_id(imported);
                            result.add_edge(Edge::new(
                                file_node_id.clone(),
                                target_id,
                                "imports".to_string(),
                                Confidence::Extracted,
                            ));
                        }
                    }
                }
            }
        }
    }
    // Go: import_spec_list
    else if kind == "import_spec_list" {
        let child_count = node.child_count();
        for i in 0..child_count {
            if let Some(child) = node.child(i as u32) {
                if child.kind() == "import_spec" {
                    let gc_count = child.child_count();
                    for j in 0..gc_count {
                        if let Some(gc) = child.child(j as u32) {
                            if gc.kind() == "string" {
                                if let Ok(text) = gc.utf8_text(source) {
                                    let module = extract_go_import(text);
                                    if !module.is_empty() {
                                        let target_id = make_import_node_id(&module);
                                        result.add_edge(Edge::new(
                                            file_node_id.clone(),
                                            target_id,
                                            "imports".to_string(),
                                            Confidence::Extracted,
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    // C/C++: preproc_include
    else if kind == "preproc_include" {
        let child_count = node.child_count();
        for i in 0..child_count {
            if let Some(child) = node.child(i as u32) {
                if child.kind() == "string_literal" || child.kind() == "system_lib_string" {
                    if let Ok(text) = child.utf8_text(source) {
                        let module = text
                            .trim_matches('"')
                            .trim_matches('<')
                            .trim_matches('>')
                            .trim();
                        if !module.is_empty() {
                            let target_id = make_import_node_id(module);
                            result.add_edge(Edge::new(
                                file_node_id.clone(),
                                target_id,
                                "imports".to_string(),
                                Confidence::Extracted,
                            ));
                        }
                    }
                }
            }
        }
    }
    // C#: using_directive
    else if kind == "using_directive" {
        let child_count = node.child_count();
        for i in 0..child_count {
            if let Some(child) = node.child(i as u32) {
                if child.kind() == "identifier" || child.kind() == "qualified_name" {
                    if let Ok(text) = child.utf8_text(source) {
                        let module = text.trim();
                        if !module.is_empty() {
                            let imported = module.split('.').last().unwrap_or(module);
                            if !imported.is_empty() {
                                let target_id = make_import_node_id(imported);
                                result.add_edge(Edge::new(
                                    file_node_id.clone(),
                                    target_id,
                                    "imports".to_string(),
                                    Confidence::Extracted,
                                ));
                            }
                        }
                    }
                }
            }
        }
    }
    // Kotlin: import_header
    else if kind == "import_header" {
        if let Some(path) = node.child_by_field_name("path") {
            if let Ok(text) = path.utf8_text(source) {
                let module = text.trim();
                if !module.is_empty() {
                    let imported = module.split('.').last().unwrap_or(module);
                    let target_id = make_import_node_id(imported);
                    result.add_edge(Edge::new(
                        file_node_id.clone(),
                        target_id,
                        "imports".to_string(),
                        Confidence::Extracted,
                    ));
                }
            }
        }
    }
    // PHP: namespace_use_clause
    else if kind == "namespace_use_clause" {
        let child_count = node.child_count();
        for i in 0..child_count {
            if let Some(child) = node.child(i as u32) {
                if child.kind() == "qualified_name"
                    || child.kind() == "name"
                    || child.kind() == "identifier"
                {
                    if let Ok(text) = child.utf8_text(source) {
                        let module = text.trim().replace('\\', ".");
                        if !module.is_empty() {
                            let imported = module.split('.').last().unwrap_or(&module);
                            let target_id = make_import_node_id(imported);
                            result.add_edge(Edge::new(
                                file_node_id.clone(),
                                target_id,
                                "imports".to_string(),
                                Confidence::Extracted,
                            ));
                        }
                    }
                }
            }
        }
    }
    // Ruby: require, require_relative
    else if kind == "require" || kind == "require_relative" {
        if let Ok(text) = node.utf8_text(source) {
            let module = text
                .trim()
                .trim_start_matches("require")
                .trim_start_matches("require_relative")
                .trim()
                .trim_matches('\'')
                .trim_matches('"');
            if !module.is_empty() {
                let target_id = make_import_node_id(module);
                let relation = if kind == "require_relative" {
                    "imports_from"
                } else {
                    "imports"
                };
                result.add_edge(Edge::new(
                    file_node_id.clone(),
                    target_id,
                    relation.to_string(),
                    Confidence::Extracted,
                ));
            }
        }
    }
    // Scala: import
    else if kind == "import" || kind == "import_declaration" {
        let child_count = node.child_count();
        for i in 0..child_count {
            if let Some(child) = node.child(i as u32) {
                if child.kind() == "identifier" || child.kind() == "wildcard" {
                    if let Ok(text) = child.utf8_text(source) {
                        let module = text.trim();
                        if !module.is_empty() && module != "_" {
                            let target_id = make_import_node_id(module);
                            result.add_edge(Edge::new(
                                file_node_id.clone(),
                                target_id,
                                "imports".to_string(),
                                Confidence::Extracted,
                            ));
                        }
                    }
                }
            }
        }
    }
}

/// Make a node ID for an imported module
fn make_import_node_id(module: &str) -> String {
    module
        .trim()
        .trim_start_matches('.')
        .replace('\\', "_")
        .replace('/', "_")
        .replace('.', "_")
        .to_lowercase()
}

/// Extract Go import path from string literal
fn extract_go_import(text: &str) -> &str {
    let text = text.trim();
    text.trim_matches('"')
}

/// PASS 2: Build edges (call graph)
/// 
/// Walks the AST to find:
/// - call_expression → creates "calls" edge
/// - Confidence is EXTRACTED if callee is a known definition
/// - Confidence is INFERRED if callee is external/unknown
fn walk_tree_pass2(
    node: &TsNode,
    ctx: &mut ExtractContext,
    result: &mut ExtractionResult,
    source: &[u8],
) {
    let kind = node.kind();

    // Call expressions - look for function calls
    if kind == "call_expression" || kind == "call" || kind == "invocation" {
        if let Some((caller, callee, caller_name)) = extract_call(node, ctx, source) {
            let confidence = if ctx.is_known(&callee) {
                Confidence::Extracted
            } else {
                Confidence::Inferred
            };

            // Add caller node if not exists (for calls in global scope)
            if !ctx.is_known(&caller) {
                ctx.add_known_node(caller.clone());
                result.add_node(Node::new(
                    caller.clone(),
                    caller_name.clone(),
                    ctx.file_path.clone(),
                    "L?".to_string(),
                ));
            }

            // Add callee node if not exists (for inferred calls to external/unknown functions)
            if confidence == Confidence::Inferred && !ctx.is_known(&callee) {
                ctx.add_known_node(callee.clone());
                // Extract just the function name without file_stem prefix
                let callee_name = callee.split(':').last().unwrap_or(&callee).to_string();
                result.add_node(Node::new(
                    callee.clone(),
                    callee_name,
                    ctx.file_path.clone(),
                    "L?".to_string(),
                ));
            }

            result.add_edge(Edge::new(caller, callee, "calls".to_string(), confidence));
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
    // Common: name, identifier, function, declarator
    // Rust: type (for impl_item), name (for struct, enum)
    // TypeScript: name, body (for classes)
    let field_names = ["name", "identifier", "function", "declarator", "type", "id"];

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
        if !trimmed.is_empty() && trimmed.len() < 200 {
            // Split by whitespace and find the first valid identifier
            // Skip common keywords like "pub", "fn", "struct", "class", "def", etc.
            let keywords = [
                "pub", "fn", "func", "function", "def", "class", "struct", "enum",
                "impl", "trait", "type", "interface", "module", "const", "static",
                "async", "pub", "mut", "ref", "move", "unsafe", "extern", "crate",
            ];
            for word in trimmed.split_whitespace() {
                let word_clean: String = word
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '_')
                    .collect();
                if !word_clean.is_empty()
                    && !keywords.contains(&word_clean.as_str())
                    && (word_clean.chars().next().map(|c| c.is_alphabetic()).unwrap_or(false))
                {
                    return Some(word_clean);
                }
            }
        }
    }

    None
}

/// Extract call information - find function being called
/// Returns (caller_full, callee_full, caller_name)
fn extract_call(
    node: &TsNode,
    ctx: &ExtractContext,
    source: &[u8],
) -> Option<(String, String, String)> {
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
        let caller =
            get_enclosing_name_with_source(node, source).unwrap_or_else(|| "global".to_string());

        let callee = format!("{}:{}", ctx.file_stem, name);
        let caller_full = format!("{}:{}", ctx.file_stem, caller);

        return Some((caller_full, callee, caller));
    }

    None
}

/// Get enclosing definition name (with source for text extraction)
fn get_enclosing_name_with_source(node: &TsNode, source: &[u8]) -> Option<String> {
    let mut current = node.clone();

    while let Some(parent) = current.parent() {
        let kind = parent.kind();
        if kind == "function_definition"
            || kind == "method_definition"
            || kind == "class_definition"
            || kind == "function_declaration"
            || kind == "procedure_definition"
        {
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
