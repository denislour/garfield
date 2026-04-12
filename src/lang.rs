//! Language configuration for Garfield
//! 
//! Centralized language definitions using tree-sitter-language-pack

use tree_sitter_language_pack::Language;
use std::collections::HashMap;

/// Language configuration
#[derive(Clone)]
pub struct LangConfig {
    pub name: &'static str,
    pub extensions: Vec<&'static str>,
    pub comment_style: CommentStyle,
    pub import_kinds: Vec<&'static str>,
    pub function_kinds: Vec<&'static str>,
}

#[derive(Clone, Copy, Debug)]
pub enum CommentStyle {
    Hash,      // # comment (Python, Ruby, Shell)
    CStyle,    // // comment (C, C++, Java, Go, JS, TS)
    Pascal,    // (* *) (Pascal, OCaml)
}

impl LangConfig {
    /// Get language config by name
    pub fn from_name(name: &str) -> Option<&'static LangConfig> {
        LANG_CONFIGS.get(name)
    }

    /// Get all supported language names
    pub fn all_languages() -> Vec<&'static str> {
        LANG_CONFIGS.keys().cloned().collect()
    }

    /// Check if language is supported
    pub fn is_supported(name: &str) -> bool {
        LANG_CONFIGS.contains_key(name)
    }
}

/// All language configurations
pub static LANG_CONFIGS: LazyLock<HashMap<&'static str, LangConfig>> = LazyLock::new(|| {
    let mut m = HashMap::new();

    // Rust
    m.insert("rust", LangConfig {
        name: "rust",
        extensions: vec!["rs"],
        comment_style: CommentStyle::CStyle,
        import_kinds: vec!["use_declaration"],
        function_kinds: vec!["function_item", "function_declaration"],
    });

    // Python
    m.insert("python", LangConfig {
        name: "python",
        extensions: vec!["py", "pyi", "pyw"],
        comment_style: CommentStyle::Hash,
        import_kinds: vec!["import_statement", "import_from_statement"],
        function_kinds: vec!["function_definition", "async_function_definition"],
    });

    // Ruby
    m.insert("ruby", LangConfig {
        name: "ruby",
        extensions: vec!["rb"],
        comment_style: CommentStyle::Hash,
        import_kinds: vec!["require", "require_relative", "load"],
        function_kinds: vec!["method", "singleton_method", "block"],
    });

    // Java
    m.insert("java", LangConfig {
        name: "java",
        extensions: vec!["java"],
        comment_style: CommentStyle::CStyle,
        import_kinds: vec!["import_declaration"],
        function_kinds: vec!["method_declaration", "constructor_declaration"],
    });

    // Go
    m.insert("go", LangConfig {
        name: "go",
        extensions: vec!["go"],
        comment_style: CommentStyle::CStyle,
        import_kinds: vec!["import_declaration"],
        function_kinds: vec!["function_declaration", "method_declaration"],
    });

    // JavaScript
    m.insert("javascript", LangConfig {
        name: "javascript",
        extensions: vec!["js", "mjs", "cjs", "jsx"],
        comment_style: CommentStyle::CStyle,
        import_kinds: vec!["import_statement", "import_clause"],
        function_kinds: vec!["function_declaration", "arrow_function"],
    });

    // TypeScript
    m.insert("typescript", LangConfig {
        name: "typescript",
        extensions: vec!["ts", "tsx"],
        comment_style: CommentStyle::CStyle,
        import_kinds: vec!["import_statement", "import_clause"],
        function_kinds: vec!["function_declaration", "arrow_function", "method_definition"],
    });

    // C
    m.insert("c", LangConfig {
        name: "c",
        extensions: vec!["c", "h"],
        comment_style: CommentStyle::CStyle,
        import_kinds: vec!["preproc_include"],
        function_kinds: vec!["function_definition"],
    });

    // C++
    m.insert("cpp", LangConfig {
        name: "cpp",
        extensions: vec!["cpp", "cc", "cxx", "hpp", "hh", "hxx"],
        comment_style: CommentStyle::CStyle,
        import_kinds: vec!["using_declaration", "preproc_include"],
        function_kinds: vec!["function_definition", "method_definition"],
    });

    // Scala
    m.insert("scala", LangConfig {
        name: "scala",
        extensions: vec!["scala"],
        comment_style: CommentStyle::CStyle,
        import_kinds: vec!["import_declaration"],
        function_kinds: vec!["function_definition", "method_definition"],
    });

    // Bash/Shell
    m.insert("bash", LangConfig {
        name: "bash",
        extensions: vec!["sh", "bash", "zsh"],
        comment_style: CommentStyle::Hash,
        import_kinds: vec![],
        function_kinds: vec!["function_definition"],
    });

    m
});

use std::sync::LazyLock;

/// Get tree-sitter Language from language name
pub fn get_ts_language(name: &str) -> Option<Language> {
    use tree_sitter_language_pack::get_language;
    
    // Map common names
    let mapped = match name {
        "ts" | "tsx" => "typescript",
        "js" | "jsx" | "mjs" | "cjs" => "javascript",
        "py" => "python",
        "rs" => "rust",
        "rb" => "ruby",
        other => other,
    };
    
    get_language(mapped).ok()
}

/// Get file extension → language name mapping
pub fn get_extension_lang(ext: &str) -> Option<&'static str> {
    for (lang, config) in LANG_CONFIGS.iter() {
        if config.extensions.iter().any(|e| *e == ext) {
            return Some(lang);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_support() {
        assert!(LangConfig::is_supported("rust"));
        let config = LangConfig::from_name("rust");
        assert!(config.is_some());
        assert_eq!(config.unwrap().name, "rust");
    }

    #[test]
    fn test_python_support() {
        assert!(LangConfig::is_supported("python"));
        let lang = get_ts_language("python");
        assert!(lang.is_some());
    }

    #[test]
    fn test_ruby_support() {
        assert!(LangConfig::is_supported("ruby"));
        let lang = get_ts_language("ruby");
        assert!(lang.is_some());
    }

    #[test]
    fn test_java_support() {
        assert!(LangConfig::is_supported("java"));
        let lang = get_ts_language("java");
        assert!(lang.is_some());
    }

    #[test]
    fn test_go_support() {
        assert!(LangConfig::is_supported("go"));
        let lang = get_ts_language("go");
        assert!(lang.is_some());
    }

    #[test]
    fn test_extension_mapping() {
        assert_eq!(get_extension_lang("py"), Some("python"));
        assert_eq!(get_extension_lang("rb"), Some("ruby"));
        assert_eq!(get_extension_lang("go"), Some("go"));
        assert_eq!(get_extension_lang("java"), Some("java"));
    }
}
