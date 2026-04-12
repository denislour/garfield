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
