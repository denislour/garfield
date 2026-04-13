//! Integration tests for language support

use garfield::{get_extension_lang, get_ts_language, lang::CommentStyle, LANG_CONFIGS};

#[test]
fn test_all_languages_have_config() {
    let langs = vec![
        "rust",
        "python",
        "ruby",
        "java",
        "go",
        "javascript",
        "typescript",
        "c",
        "cpp",
        "scala",
        "lua",
        "php",
        "bash",
        "zig",
        "elixir",
        "kotlin",
        "swift",
    ];

    for lang in langs {
        assert!(
            LANG_CONFIGS.contains_key(lang),
            "Language '{}' should have config",
            lang
        );
    }
}

#[test]
fn test_extension_parsing() {
    // Note: get_extension_lang expects the extension without the dot
    assert_eq!(get_extension_lang("rs"), Some("rust"));
    assert_eq!(get_extension_lang("py"), Some("python"));
    assert_eq!(get_extension_lang("js"), Some("javascript"));
    assert_eq!(get_extension_lang("ts"), Some("typescript"));
    assert_eq!(get_extension_lang("go"), Some("go"));
    assert_eq!(get_extension_lang("java"), Some("java"));
    assert_eq!(get_extension_lang("rb"), Some("ruby"));
    assert_eq!(get_extension_lang("php"), Some("php"));
    assert_eq!(get_extension_lang("lua"), Some("lua"));
    assert_eq!(get_extension_lang("zig"), Some("zig"));
    assert_eq!(get_extension_lang("ex"), Some("elixir"));
    assert_eq!(get_extension_lang("kt"), Some("kotlin"));
    assert_eq!(get_extension_lang("swift"), Some("swift"));
}

#[test]
fn test_unknown_extension() {
    assert_eq!(get_extension_lang("xyz"), None);
    assert_eq!(get_extension_lang("abc"), None);
    assert_eq!(get_extension_lang(""), None);
}

#[test]
fn test_tree_sitter_language_loading() {
    for lang in ["rust", "python", "javascript", "typescript"] {
        let lang_config = LANG_CONFIGS.get(lang);
        assert!(
            lang_config.is_some(),
            "Language '{}' should have config",
            lang
        );

        let ts_lang = get_ts_language(lang);
        assert!(
            ts_lang.is_some(),
            "Language '{}' should have tree-sitter grammar",
            lang
        );
    }
}

#[test]
fn test_language_node_kinds_not_empty() {
    for lang in LANG_CONFIGS.keys() {
        let config = LANG_CONFIGS.get(lang).unwrap();
        assert!(
            !config.node_kinds.is_empty(),
            "Language '{}' should have node_kinds",
            lang
        );
    }
}

#[test]
fn test_language_comment_styles() {
    // Test that C-style languages have the comment_style field
    for lang in ["rust", "java", "go", "javascript", "typescript", "c", "cpp"] {
        let config = LANG_CONFIGS.get(lang).unwrap();
        assert!(
            matches!(config.comment_style, CommentStyle::CStyle),
            "Language '{}' should be C-style comments",
            lang
        );
    }

    // Test that Hash-style languages have the correct style
    for lang in ["python", "ruby", "lua", "bash"] {
        let config = LANG_CONFIGS.get(lang).unwrap();
        assert!(
            matches!(config.comment_style, CommentStyle::Hash),
            "Language '{}' should be Hash-style comments",
            lang
        );
    }
}

#[test]
fn test_rust_is_supported() {
    assert!(LANG_CONFIGS.contains_key("rust"));
    assert!(get_ts_language("rust").is_some());
}

#[test]
fn test_python_is_supported() {
    assert!(LANG_CONFIGS.contains_key("python"));
    assert!(get_ts_language("python").is_some());
}
