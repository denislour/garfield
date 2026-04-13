//! Scala language extraction integration test

use garfield::extract_file;
use tempfile::tempdir;

fn extract_from_source(source: &str) -> Vec<String> {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("Order.scala");
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
fn test_scala_class() {
    let source = r#"
class OrderService(customer: String) {
    def createOrder(items: List[String]): Order = {
        val order = new Order()
        for (item <- items) {
            if (item != null) {
                order.addItem(item)
            }
        }
        order
    }
    
    def sendConfirmation(order: Order): Unit = {
        println("Confirmed!")
    }
}

class Order {
    private var items: List[String] = Nil
    
    def addItem(item: String): Unit = {
        items = items :+ item
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
