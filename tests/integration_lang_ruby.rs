//! Ruby language extraction integration test

use garfield::extract_file;
use tempfile::tempdir;

fn extract_from_source(source: &str) -> Vec<String> {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("test.rb");
    std::fs::write(&file_path, source).unwrap();
    let content = std::fs::read_to_string(&file_path).unwrap();
    extract_file(&file_path, &content).unwrap()
        .nodes.iter()
        .map(|n| n.label.clone())
        .collect()
}

#[test]
fn test_ruby_class() {
    let source = r#"
class OrderService
  def initialize(customer)
    @customer = customer
  end
  
  def create_order(items)
    items.each do |item|
      if item
        order = Order.new
        order.add_item(item)
      end
    end
    order
  end
  
  def send_confirmation(order)
    puts "Confirmed!"
  end
end

class Order
  def initialize
    @items = []
  end
  
  def add_item(item)
    @items << item
  end
end
"#;
    let nodes = extract_from_source(source);
    assert!(nodes.contains(&"OrderService".to_string()), "Should find OrderService");
    assert!(nodes.contains(&"Order".to_string()), "Should find Order");
    assert!(nodes.contains(&"create_order".to_string()), "Should find create_order");
    assert!(nodes.contains(&"send_confirmation".to_string()), "Should find send_confirmation");
    assert!(nodes.contains(&"add_item".to_string()), "Should find add_item");
}
