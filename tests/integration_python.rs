//! Python language extraction integration test

use garfield::extract_file;
use tempfile::tempdir;

fn extract_from_source(source: &str) -> Vec<String> {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.py");
    std::fs::write(&file_path, source).unwrap();
    let content = std::fs::read_to_string(&file_path).unwrap();
    extract_file(&file_path, &content).unwrap()
        .nodes.iter()
        .map(|n| n.label.clone())
        .collect()
}

#[test]
fn test_python_class() {
    let source = r#"
class OrderService:
    def __init__(self, customer):
        self.customer = customer
    
    def create_order(self, items):
        for item in items:
            if item:
                order = Order()
                order.add_item(item)
        return order
    
    def send_confirmation(self, order):
        print("Confirmed!")

class Order:
    def __init__(self):
        self.items = []
    
    def add_item(self, item):
        self.items.append(item)
"#;
    let nodes = extract_from_source(source);
    assert!(nodes.contains(&"OrderService".to_string()), "Should find OrderService");
    assert!(nodes.contains(&"Order".to_string()), "Should find Order");
    assert!(nodes.contains(&"create_order".to_string()), "Should find create_order");
    assert!(nodes.contains(&"send_confirmation".to_string()), "Should find send_confirmation");
    assert!(nodes.contains(&"add_item".to_string()), "Should find add_item");
}
