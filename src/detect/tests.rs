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

    #[test]
    fn test_glob_to_regex() {
        let pattern = glob_to_regex("*.py");
        assert!(Regex::new(&pattern).is_ok());

        let pattern2 = glob_to_regex("src/**");
        assert!(Regex::new(&pattern2).is_ok());
    }

    #[test]
    fn test_is_sensitive() {
        let env_path = Path::new("/project/.env");
        assert!(is_sensitive(env_path).is_some());
        
        let normal_path = Path::new("/project/main.py");
        assert!(is_sensitive(normal_path).is_none());
    }
}
