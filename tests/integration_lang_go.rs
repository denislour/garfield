//! Go language extraction integration test

use garfield::extract_file;
use tempfile::tempdir;

fn extract_from_source(source: &str) -> Vec<String> {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("order.go");
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
fn test_go_struct() {
    let source = r#"
package main

type OrderService struct {
    customer string
}

func NewOrderService(customer string) *OrderService {
    return &OrderService{customer: customer}
}

func (s *OrderService) CreateOrder(items []string) *Order {
    order := NewOrder()
    for _, item := range items {
        if item != "" {
            order.AddItem(item)
        }
    }
    return order
}

func (s *OrderService) SendConfirmation(order *Order) {
    println("Confirmed!")
}

type Order struct {
    items []string
}

func NewOrder() *Order {
    return &Order{items: make([]string, 0)}
}

func (o *Order) AddItem(item string) {
    o.items = append(o.items, item)
}
"#;
    let nodes = extract_from_source(source);
    assert!(
        nodes.contains(&"NewOrderService".to_string()),
        "Should find NewOrderService"
    );
    assert!(
        nodes.contains(&"CreateOrder".to_string()),
        "Should find CreateOrder"
    );
    assert!(
        nodes.contains(&"SendConfirmation".to_string()),
        "Should find SendConfirmation"
    );
    assert!(
        nodes.contains(&"NewOrder".to_string()),
        "Should find NewOrder"
    );
    assert!(
        nodes.contains(&"AddItem".to_string()),
        "Should find AddItem"
    );
}
