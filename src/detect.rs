//! File detection module

use std::path::Path;
use walkdir::WalkDir;
use crate::types::{DetectedFile, DetectStats, FileType};

/// Code extensions
const CODE_EXTENSIONS: &[&str] = &[
    // Python
    "py", "pyi", "pyw",
    // JavaScript/TypeScript
    "js", "mjs", "cjs", "jsx", "ts", "tsx",
    // Go
    "go",
    // Rust
    "rs",
    // Java
    "java",
    // C/C++
    "c", "h", "cpp", "hpp", "cc", "cxx", "hxx",
    // Ruby
    "rb",
    // C#
    "cs",
    // Kotlin
    "kt", "kts",
    // Scala
    "scala",
    // PHP
    "php",
    // Swift
    "swift",
    // Lua
    "lua",
    // Zig
    "zig",
    // PowerShell
    "ps1", "psm1",
    // Elixir
    "ex", "exs",
    // Objective-C
    "m", "mm",
    // Julia
    "jl",
    // TOML
    "toml",
    // YAML
    "yaml", "yml",
    // JSON
    "json",
];

/// Markdown extensions
const MARKDOWN_EXTENSIONS: &[&str] = &[
    "md", "mdx", "markdown",
    "txt",
];

/// Detect files trong directory
pub fn detect(root: &Path) -> anyhow::Result<Vec<DetectedFile>> {
    let mut files = Vec::new();
    
    for entry in WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        
        // Skip directories
        if !path.is_file() {
            continue;
        }
        
        // Skip hidden files
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.starts_with('.') {
                continue;
            }
        }
        
        // Get extension
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .unwrap_or_default();
        
        // Skip if no extension
        if ext.is_empty() {
            continue;
        }
        
        // Classify
        let file_type = classify_extension(&ext);
        
        // Get size
        let size_bytes = entry.metadata().map(|m| m.len()).unwrap_or(0);
        
        files.push(DetectedFile {
            path: path.to_path_buf(),
            file_type,
            extension: ext,
            size_bytes,
        });
    }
    
    Ok(files)
}

/// Classify extension -> FileType
fn classify_extension(ext: &str) -> FileType {
    if CODE_EXTENSIONS.iter().any(|e| *e == ext) {
        FileType::Code
    } else if MARKDOWN_EXTENSIONS.iter().any(|e| *e == ext) {
        FileType::Markdown
    } else {
        FileType::Binary
    }
}

/// Filter only code files
pub fn filter_code_files(files: &[DetectedFile]) -> Vec<DetectedFile> {
    files
        .iter()
        .filter(|f| f.file_type == FileType::Code)
        .cloned()
        .collect()
}

/// Get stats
pub fn get_stats(files: &[DetectedFile]) -> DetectStats {
    let code = files.iter().filter(|f| f.file_type == FileType::Code).count();
    let markdown = files.iter().filter(|f| f.file_type == FileType::Markdown).count();
    let binary = files.iter().filter(|f| f.file_type == FileType::Binary).count();
    
    DetectStats {
        total: files.len(),
        code,
        markdown,
        binary,
    }
}

/// Print detection summary
pub fn print_summary(files: &[DetectedFile]) {
    let stats = get_stats(files);
    
    println!("Detection Summary:");
    println!("  Total: {} files", stats.total);
    println!("  Code: {} files", stats.code);
    println!("  Markdown: {} files", stats.markdown);
    println!("  Other: {} files", stats.binary);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_code() {
        assert_eq!(classify_extension("py"), FileType::Code);
        assert_eq!(classify_extension("rs"), FileType::Code);
        assert_eq!(classify_extension("js"), FileType::Code);
        assert_eq!(classify_extension("go"), FileType::Code);
    }

    #[test]
    fn test_classify_markdown() {
        assert_eq!(classify_extension("md"), FileType::Markdown);
        assert_eq!(classify_extension("txt"), FileType::Markdown);
    }

    #[test]
    fn test_classify_binary() {
        assert_eq!(classify_extension("png"), FileType::Binary);
        assert_eq!(classify_extension("jpg"), FileType::Binary);
        assert_eq!(classify_extension("pdf"), FileType::Binary);
    }
}
