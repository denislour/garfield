//! TypeScript language extraction integration test

use garfield::extract_file;
use tempfile::tempdir;

fn extract_from_source(source: &str) -> Vec<String> {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("order.ts");
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
fn test_typescript_class() {
    let source = r#"
class OrderService {
    private customer: string;
    
    constructor(customer: string) {
        this.customer = customer;
    }
    
    createOrder(items: string[]): Order {
        const order = new Order();
        for (const item of items) {
            if (item) {
                order.addItem(item);
            }
        }
        return order;
    }
    
    private sendConfirmation(order: Order): void {
        console.log("Confirmed!");
    }
}

class Order {
    private items: string[] = [];
    
    addItem(item: string): void {
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
