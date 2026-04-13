//! Java language extraction integration test

use garfield::extract_file;
use tempfile::tempdir;

fn extract_from_source(source: &str) -> Vec<String> {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("Test.java");
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
fn test_java_class() {
    let source = r#"
public class OrderService {
    private String customer;
    
    public OrderService(String customer) {
        this.customer = customer;
    }
    
    public Order createOrder(String[] items) {
        Order order = new Order();
        for (String item : items) {
            if (item != null) {
                order.addItem(item);
            }
        }
        return order;
    }
    
    private void sendConfirmation(Order order) {
        System.out.println("Confirmed!");
    }
}

class Order {
    private List<String> items;
    
    public Order() {
        this.items = new ArrayList<>();
    }
    
    public void addItem(String item) {
        this.items.add(item);
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
