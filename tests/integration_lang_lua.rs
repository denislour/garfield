//! Lua language extraction integration test

use garfield::extract_file;
use tempfile::tempdir;

fn extract_from_source(source: &str) -> Vec<String> {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("order.lua");
    std::fs::write(&file_path, source).unwrap();
    let content = std::fs::read_to_string(&file_path).unwrap();
    extract_file(&file_path, &content)
        .unwrap()
        .nodes
        .iter()
        .map(|n| n.label.clone())
        .collect()
}

fn contains_any(nodes: &[String], candidates: &[&str]) -> bool {
    candidates
        .iter()
        .any(|c| nodes.iter().any(|n| n.contains(c)))
}

#[test]
fn test_lua_function() {
    let source = r#"
-- OrderService module
local OrderService = {}

function OrderService.new(customer)
    local self = {}
    self.customer = customer
    setmetatable(self, { __index = OrderService })
    return self
end

function OrderService:create_order(items)
    local order = Order.new()
    for i, item in ipairs(items) do
        if item then
            order:add_item(item)
        end
    end
    return order
end

function OrderService:send_confirmation(order)
    print("Confirmed!")
end

-- Order module
local Order = {}

function Order.new()
    local self = {}
    self.items = {}
    setmetatable(self, { __index = Order })
    return self
end

function Order:add_item(item)
    table.insert(self.items, item)
end

return OrderService
"#;
    let nodes = extract_from_source(source);

    // Lua uses colon syntax: OrderService:create_order
    assert!(
        contains_any(&nodes, &["OrderService.new", "new"]),
        "Should find new function"
    );
    assert!(
        contains_any(&nodes, &["OrderService:create_order", "create_order"]),
        "Should find create_order, got {:?}",
        nodes
    );
    assert!(
        contains_any(
            &nodes,
            &["OrderService:send_confirmation", "send_confirmation"]
        ),
        "Should find send_confirmation"
    );
    assert!(
        contains_any(&nodes, &["Order.new", "new"]),
        "Should find Order.new"
    );
    assert!(
        contains_any(&nodes, &["Order:add_item", "add_item"]),
        "Should find add_item"
    );
}
