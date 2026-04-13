//! Rust language extraction integration test

use garfield::extract_file;
use tempfile::tempdir;

fn extract_from_source(source: &str) -> Vec<String> {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("order.rs");
    std::fs::write(&file_path, source).unwrap();
    let content = std::fs::read_to_string(&file_path).unwrap();
    extract_file(&file_path, &content)
        .unwrap()
        .nodes
        .iter()
        .map(|n| n.label.clone())
        .collect()
}

#[test]
fn test_rust_struct() {
    let source = r#"
pub struct OrderService {
    customer: String,
}

impl OrderService {
    pub fn new(customer: &str) -> Self {
        Self { customer: customer.to_string() }
    }
    
    pub fn create_order(&mut self, items: Vec<String>) -> Order {
        let mut order = Order::new();
        for item in items {
            if !item.is_empty() {
                order.add_item(item);
            }
        }
        order
    }
    
    fn send_confirmation(&self, order: &Order) {
        println!("Confirmed!");
    }
}

pub struct Order {
    items: Vec<String>,
}

impl Order {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }
    
    pub fn add_item(&mut self, item: String) {
        self.items.push(item);
    }
}
"#;
    let nodes = extract_from_source(source);
    assert!(
        nodes.contains(&"OrderService".to_string()),
        "Should find OrderService"
    );
    assert!(nodes.contains(&"Order".to_string()), "Should find Order");
    assert!(
        nodes.contains(&"create_order".to_string()),
        "Should find create_order"
    );
    assert!(
        nodes.contains(&"send_confirmation".to_string()),
        "Should find send_confirmation"
    );
    assert!(
        nodes.contains(&"add_item".to_string()),
        "Should find add_item"
    );
}
