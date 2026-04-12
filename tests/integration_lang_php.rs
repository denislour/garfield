//! PHP language extraction integration test

use garfield::extract_file;
use tempfile::tempdir;

fn extract_from_source(source: &str) -> Vec<String> {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("order.php");
    std::fs::write(&file_path, source).unwrap();
    let content = std::fs::read_to_string(&file_path).unwrap();
    extract_file(&file_path, &content).unwrap()
        .nodes.iter()
        .map(|n| n.label.clone())
        .collect()
}

fn contains_any(nodes: &[String], candidates: &[&str]) -> bool {
    candidates.iter().any(|c| nodes.iter().any(|n| n.contains(c)))
}

#[test]
fn test_php_class() {
    let source = r#"<?php
// OrderService class
class OrderService {
    private string $customer;
    
    public function __construct(string $customer) {
        $this->customer = $customer;
    }
    
    public function createOrder(array $items): Order {
        $order = new Order();
        foreach ($items as $item) {
            if ($item) {
                $order->addItem($item);
            }
        }
        return $order;
    }
    
    private function sendConfirmation(Order $order): void {
        echo "Confirmed!";
    }
}

// Order class
class Order {
    private array $items = [];
    
    public function addItem(string $item): void {
        $this->items[] = $item;
    }
}
"#;
    let nodes = extract_from_source(source);
    
    assert!(contains_any(&nodes, &["OrderService", "class"]), 
        "Should find OrderService class, got {:?}", nodes);
    assert!(contains_any(&nodes, &["createOrder", "function"]), 
        "Should find createOrder method, got {:?}", nodes);
    assert!(contains_any(&nodes, &["sendConfirmation", "function"]), 
        "Should find sendConfirmation method");
    assert!(contains_any(&nodes, &["Order", "class"]), 
        "Should find Order class");
    assert!(contains_any(&nodes, &["addItem", "function"]), 
        "Should find addItem method");
}
