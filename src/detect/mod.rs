//! File detection module - matches Graphify detect.py
//!
//! ## Detect Flow
//!
//! 1. Walk directory tree using walkdir
//! 2. Skip noise directories (venv, node_modules, .git, etc.)
//! 3. Load .graphifyignore patterns
//! 4. Skip sensitive files (secrets, credentials) - with LOGGING
//! 5. Classify files by extension
//! 6. Generate stats and warnings
//! 7. Return detected files with warnings

use crate::types::{DetectStats, DetectedFile, FileType};
use regex::Regex;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use walkdir::WalkDir;

/// Noise directory names to skip
const SKIP_DIRS: &[&str] = &[
    "venv",
    ".venv",
    "env",
    ".env",
    "node_modules",
    "__pycache__",
    ".git",
    "dist",
    "build",
    "target",
    "out",
    "site-packages",
    "lib64",
    ".pytest_cache",
    ".mypy_cache",
    ".ruff_cache",
    ".tox",
    ".eggs",
    "*.egg-info",
    ".hg",
    ".svn",
    ".bzr",
    "vendor",
    ".cargo",
];

/// Sensitive file patterns to skip
const SENSITIVE_PATTERNS: &[&str] = &[
    r"\.env$",
    r"\.envrc$",
    r"\.pem$",
    r"\.key$",
    r"\.p12$",
    r"\.pfx$",
    r"\.cert$",
    r"\.crt$",
    r"\.der$",
    r"\.p8$",
    r"credential",
    r"secret",
    r"passwd",
    r"password",
    r"token",
    r"private_key",
    r"id_rsa",
    r"id_dsa",
    r"id_ecdsa",
    r"id_ed25519",
    r"\.netrc$",
    r"\.pgpass$",
    r"\.htpasswd$",
    r"aws_credentials",
    r"gcloud_credentials",
    r"service\.account",
];

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
const MARKDOWN_EXTENSIONS: &[&str] = &["md", "mdx", "markdown", "txt"];

/// Detection result with warnings
#[derive(Debug)]
pub struct DetectResult {
    pub files: Vec<DetectedFile>,
    pub stats: DetectStats,
    pub word_count: usize,
    pub warnings: Vec<String>,
    pub sensitive_files_skipped: Vec<String>,
}

impl DetectResult {
    /// Generate corpus verdict based on word count
    pub fn corpus_verdict(&self) -> String {
        if self.word_count < 50_000 {
            "⚠️ Corpus may be too small (<50K words) - graph may not add much value".to_string()
        } else if self.word_count > 500_000 {
            format!(
                "⚠️ Large corpus ({} words) - may have high token cost",
                self.word_count
            )
        } else {
            "✓ Corpus size is appropriate".to_string()
        }
    }
}

/// Load .graphifyignore patterns from root and ancestor directories
fn load_graphifyignore(root: &Path) -> Vec<Regex> {
    let mut patterns: Vec<Regex> = Vec::new();
    let mut current = root.to_path_buf();

    loop {
        let ignore_file = current.join(".graphifyignore");
        if ignore_file.exists() {
            if let Ok(content) = std::fs::read_to_string(&ignore_file) {
                let file_path = ignore_file.to_string_lossy().to_string();
                let line_count = content.lines().count();
                let mut valid_count = 0;
                
                for line in content.lines() {
                    let line = line.trim();
                    if line.is_empty() || line.starts_with('#') {
                        continue;
                    }
                    let pattern = glob_to_regex(line);
                    if let Ok(re) = Regex::new(&pattern) {
                        patterns.push(re);
                        valid_count += 1;
                    }
                }
                
                if valid_count > 0 {
                    eprintln!("  Loaded {} patterns from {} ({} lines)", 
                        valid_count, file_path, line_count);
                }
            }
        }

        // Stop at git repo root
        if current.join(".git").exists() {
            break;
        }

        // Stop at filesystem root
        let parent = current.parent().map(|p| p.to_path_buf());
        match parent {
            Some(p) if p != current => current = p,
            _ => break,
        }
    }

    patterns
}

/// Convert glob pattern to regex
fn glob_to_regex(pattern: &str) -> String {
    let mut result = String::from("^");
    let p = pattern.trim_end_matches('/');

    for c in p.chars() {
        match c {
            '*' => result.push_str(".*"),
            '?' => result.push('.'),
            '.' => result.push_str("\\."),
            '/' => result.push_str("[/\\\\]"),
            '[' => result.push('['),
            ']' => result.push(']'),
            _ if c.is_alphanumeric() || c == '_' || c == '-' => result.push(c),
            _ => result.push_str(&regex::escape(&c.to_string())),
        }
    }

    result.push('$');
    result
}

/// Check if path is ignored by .graphifyignore
fn is_ignored(path: &Path, root: &Path, patterns: &[Regex]) -> bool {
    if patterns.is_empty() {
        return false;
    }

    let rel_path = match path.strip_prefix(root) {
        Ok(p) => p,
        Err(_) => path,
    };

    let rel_str = rel_path.to_string_lossy().replace('\\', "/");

    for pattern in patterns {
        if pattern.is_match(&rel_str)
            || pattern.is_match(path.file_name().unwrap_or_default().to_str().unwrap_or(""))
        {
            return true;
        }
    }

    false
}

/// Check if path is a noise directory
fn is_noise_dir(name: &str) -> bool {
    if SKIP_DIRS.contains(&name) {
        return true;
    }
    // Check *_venv, *_env patterns
    if name.ends_with("_venv") || name.ends_with("_env") {
        return true;
    }
    if name.ends_with(".egg-info") {
        return true;
    }
    false
}

/// Check if path is sensitive (may contain secrets) - returns reason if sensitive
fn is_sensitive(path: &Path) -> Option<String> {
    let path_str = path.to_string_lossy();
    let name = path.file_name().unwrap_or_default().to_string_lossy();

    for pattern in SENSITIVE_PATTERNS {
        if let Ok(re) = Regex::new(pattern) {
            if re.is_match(&path_str) || re.is_match(&name) {
                // Return reason for logging
                return Some(format!("matches pattern '{}'", pattern));
            }
        }
    }

    None
}

/// Detect files in directory with detailed logging
pub fn detect(root: &Path) -> anyhow::Result<DetectResult> {
    let root = root.to_path_buf();
    let ignore_patterns = load_graphifyignore(&root);
    let mut files = Vec::new();
    let mut warnings = Vec::new();
    let mut sensitive_skipped = Vec::new();
    
    // Track counts
    let total_entries = AtomicUsize::new(0);
    let skipped_noise = AtomicUsize::new(0);
    let skipped_ignored = AtomicUsize::new(0);
    let skipped_hidden = AtomicUsize::new(0);

    for entry in WalkDir::new(&root)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        total_entries.fetch_add(1, Ordering::Relaxed);
        let path = entry.path();

        // Skip directories
        if !path.is_file() {
            continue;
        }

        // Skip hidden files
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.starts_with('.') {
                skipped_hidden.fetch_add(1, Ordering::Relaxed);
                continue;
            }
        }

        // Skip noise directories
        let mut in_noise_dir = false;
        if let Some(parent) = path.parent() {
            for part in parent.strip_prefix(&root).unwrap_or(parent).components() {
                let dir_name = part.as_os_str().to_string_lossy();
                if is_noise_dir(&dir_name) {
                    skipped_noise.fetch_add(1, Ordering::Relaxed);
                    in_noise_dir = true;
                    break;
                }
            }
        }
        if in_noise_dir {
            continue;
        }

        // Check .graphifyignore
        if is_ignored(path, &root, &ignore_patterns) {
            skipped_ignored.fetch_add(1, Ordering::Relaxed);
            continue;
        }

        // Check sensitive files - LOG WITH REASON
        if let Some(reason) = is_sensitive(path) {
            let path_str = path.to_string_lossy();
            sensitive_skipped.push(format!("  - {} ({})", path_str, reason));
            continue;
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

    // Generate warnings
    let total = total_entries.load(Ordering::Relaxed);
    let noise_skipped = skipped_noise.load(Ordering::Relaxed);
    let ignored_skipped = skipped_ignored.load(Ordering::Relaxed);
    let hidden_skipped = skipped_hidden.load(Ordering::Relaxed);
    
    // Log summary
    eprintln!("\n📁 Detection Summary:");
    eprintln!("  Total entries scanned: {}", total);
    eprintln!("  Hidden files skipped: {}", hidden_skipped);
    eprintln!("  Noise directories skipped: {}", noise_skipped);
    eprintln!("  .graphifyignore patterns skipped: {}", ignored_skipped);
    
    if !sensitive_skipped.is_empty() {
        eprintln!("  🔒 Sensitive files skipped: {}", sensitive_skipped.len());
        for skipped in sensitive_skipped.iter().take(5) {
            eprintln!("    {}", skipped);
        }
        if sensitive_skipped.len() > 5 {
            eprintln!("    ... and {} more", sensitive_skipped.len() - 5);
        }
    }

    // Calculate stats
    let stats = get_stats(&files);
    let word_count = estimate_word_count(&files);

    // Generate corpus warnings (matching Graphify)
    if stats.code < 5 {
        warnings.push("⚠️ Very few code files found - graph may be too sparse".to_string());
    }
    if stats.code > 200 {
        warnings.push(format!(
            "⚠️ Many code files ({} > 200) - extraction may take longer",
            stats.code
        ));
    }
    if word_count < 50_000 && stats.code > 0 {
        warnings.push(
            "⚠️ Corpus may be too small for meaningful graph (<50K words)".to_string()
        );
    }
    if word_count > 500_000 {
        warnings.push(
            "⚠️ Large corpus (>500K words) - consider splitting into smaller projects".to_string()
        );
    }

    // Log warnings
    if !warnings.is_empty() {
        eprintln!("\n⚠️  Warnings:");
        for warning in &warnings {
            eprintln!("  {}", warning);
        }
    }

    // Log final stats
    eprintln!("\n📊 File Classification:");
    eprintln!("  Code files: {} ({} words)", stats.code, word_count);
    eprintln!("  Markdown: {}", stats.markdown);
    eprintln!("  Other: {}", stats.binary);
    eprintln!("  Total usable: {}", files.len());

    Ok(DetectResult {
        files,
        stats,
        word_count,
        warnings,
        sensitive_files_skipped: sensitive_skipped,
    })
}

/// Classify extension -> FileType
pub fn classify_extension(ext: &str) -> FileType {
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
    let code = files
        .iter()
        .filter(|f| f.file_type == FileType::Code)
        .count();
    let markdown = files
        .iter()
        .filter(|f| f.file_type == FileType::Markdown)
        .count();
    let binary = files
        .iter()
        .filter(|f| f.file_type == FileType::Binary)
        .count();

    DetectStats {
        total: files.len(),
        code,
        markdown,
        binary,
    }
}

/// Estimate word count from files
/// For code: count alphanumeric sequences
/// For docs: more accurate word count
pub fn estimate_word_count(files: &[DetectedFile]) -> usize {
    let mut total_words = 0;

    for file in files {
        let path = &file.path;

        // Try to read file content
        if let Ok(content) = std::fs::read_to_string(path) {
            let words = count_words(&content, &file.extension);
            total_words += words;
        } else {
            // Fallback: estimate from file size
            // Average 1 word = 5 bytes for code
            total_words += file.size_bytes as usize / 5;
        }
    }

    total_words
}

/// Count words in content
fn count_words(content: &str, ext: &str) -> usize {
    // For markdown/text, count actual words
    if matches!(ext, "md" | "markdown" | "txt" | "rst") {
        return content.split_whitespace().count();
    }

    // For code, count identifiers and comments
    // Simple heuristic: split by whitespace and filter alphanumeric
    let mut count = 0;
    let mut in_word = false;

    for c in content.chars() {
        if c.is_alphanumeric() || c == '_' {
            if !in_word {
                count += 1;
                in_word = true;
            }
        } else {
            in_word = false;
        }
    }

    count
}

/// Print detection summary (legacy function)
pub fn print_summary(files: &[DetectedFile]) {
    let stats = get_stats(files);
    let words = estimate_word_count(files);

    println!("Detection Summary:");
    println!("  Total: {} files", stats.total);
    println!("  Code: {} files ({} words)", stats.code, words);
    println!("  Markdown: {} files", stats.markdown);
    println!("  Other: {} files", stats.binary);
}
