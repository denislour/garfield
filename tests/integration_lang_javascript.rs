//! JavaScript language extraction integration test

use garfield::extract_file;
use tempfile::tempdir;

fn extract_from_source(source: &str) -> Vec<String> {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("order.js");
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
fn test_javascript_class() {
    let source = r#"
class OrderService {
    constructor(customer) {
        this.customer = customer;
    }
    
    createOrder(items) {
        const order = new Order();
        for (const item of items) {
            if (item) {
                order.addItem(item);
            }
        }
        return order;
    }
    
    sendConfirmation(order) {
        console.log("Confirmed!");
    }
}

class Order {
    constructor() {
        this.items = [];
    }
    
    addItem(item) {
        this.items.push(item);
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
        nodes.contains(&"createOrder".to_string()),
        "Should find createOrder"
    );
    assert!(
        nodes.contains(&"sendConfirmation".to_string()),
        "Should find sendConfirmation"
    );
    assert!(
        nodes.contains(&"addItem".to_string()),
        "Should find addItem"
    );
}
