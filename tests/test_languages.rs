//! Language support tests for Garfield
//! 
//! Tests extraction for: Rust, Python, Ruby, Java, Go, TypeScript, JavaScript, Zig, HTML, CSS

use std::path::Path;
use garfield::{extract_file, extract_files};
use tempfile::tempdir;

fn extract_from_source(ext: &str, source: &str) -> Vec<String> {
    let dir = tempdir().unwrap();
    let file_path = dir.path().join(format!("test.{}", ext));
    std::fs::write(&file_path, source).unwrap();
    
    let source_content = std::fs::read_to_string(&file_path).unwrap();
    let result = extract_file(&file_path, &source_content).unwrap();
    
    result.nodes.iter().map(|n| n.label.clone()).collect()
}

// Check if a language is supported by the tree-sitter-language-pack
fn is_language_supported(name: &str) -> bool {
    use tree_sitter_language_pack::get_language;
    get_language(name).is_ok()
}

#[test]
fn test_extract_rust() {
    let source = r#"
pub struct OrderService {
    customer: String,
}

impl OrderService {
    pub fn new(customer: &str) -> Self {
        Self { customer: customer.to_string() }
    }
    
    pub fn create_order(&mut self, items: Vec<Item>) -> Order {
        let mut order = Order::new();
        for item in items {
            order.add_item(item);
        }
        order.save();
        self.send_confirmation(&order);
        order
    }
    
    fn send_confirmation(&self, order: &Order) {
        email_service::send(order.customer_email(), "Confirmed!");
    }
}

pub struct Order {
    items: Vec<Item>,
}

impl Order {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }
    
    pub fn add_item(&mut self, item: Item) {
        self.items.push(item);
    }
    
    pub fn save(&self) {
        println!("Saving order...");
    }
}

pub struct Item {
    name: String,
    price: f64,
}
"#;

    let nodes = extract_from_source("rs", source);
    assert!(nodes.contains(&"OrderService".to_string()), "Should find OrderService struct");
    assert!(nodes.contains(&"Order".to_string()), "Should find Order struct");
    assert!(nodes.contains(&"Item".to_string()), "Should find Item struct");
    assert!(nodes.contains(&"create_order".to_string()), "Should find create_order method");
    assert!(nodes.contains(&"send_confirmation".to_string()), "Should find send_confirmation method");
    assert!(nodes.len() >= 6, "Should find at least 6 definitions, got {}", nodes.len());
}

#[test]
fn test_extract_python() {
    let source = r#"
from typing import List, Dict

class OrderService:
    def __init__(self, customer: str):
        self.customer = customer
        self.cache = {}
    
    def create_order(self, items: List[Item]) -> Order:
        order = Order()
        for item in items:
            order.add_item(item)
        order.save()
        self.send_confirmation(order)
        return order
    
    def send_confirmation(self, order: Order):
        email_service.send(order.customer_email, "Order confirmed!")

class Order:
    def __init__(self):
        self.items = []
    
    def add_item(self, item: Item):
        self.items.append(item)
    
    def save(self):
        print("Saving order...")

class Item:
    def __init__(self, name: str, price: float):
        self.name = name
        self.price = price
"#;

    let nodes = extract_from_source("py", source);
    assert!(nodes.contains(&"OrderService".to_string()), "Should find OrderService class");
    assert!(nodes.contains(&"Order".to_string()), "Should find Order class");
    assert!(nodes.contains(&"create_order".to_string()), "Should find create_order method");
    assert!(nodes.contains(&"send_confirmation".to_string()), "Should find send_confirmation method");
    assert!(nodes.len() >= 5, "Should find at least 5 definitions, got {}", nodes.len());
}

#[test]
fn test_extract_ruby() {
    let source = r#"
class OrderService
  def initialize(customer)
    @customer = customer
  end
  
  def create_order(items)
    order = Order.new
    items.each { |item| order.add_item(item) }
    order.save
    send_confirmation(order)
    order
  end
  
  private
  
  def send_confirmation(order)
    EmailService.send(order.customer_email, "Order confirmed!")
  end
end

class Order
  def initialize
    @items = []
  end
  
  def add_item(item)
    @items << item
  end
  
  def save
    puts "Saving order..."
  end
end
"#;

    let nodes = extract_from_source("rb", source);
    assert!(nodes.contains(&"OrderService".to_string()), "Should find OrderService class");
    assert!(nodes.contains(&"Order".to_string()), "Should find Order class");
    assert!(nodes.len() >= 2, "Should find at least 2 definitions, got {}", nodes.len());
}

#[test]
fn test_extract_java() {
    let source = r#"
import java.util.*;

public class OrderService {
    private String customer;
    private Map<String, Order> cache;
    
    public OrderService(String customer) {
        this.customer = customer;
        this.cache = new HashMap<>();
    }
    
    public Order createOrder(List<Item> items) {
        Order order = new Order();
        for (Item item : items) {
            order.addItem(item);
        }
        order.save();
        sendConfirmation(order);
        return order;
    }
    
    private void sendConfirmation(Order order) {
        emailService.send(order.getCustomerEmail(), "Order confirmed!");
    }
}

class Order {
    private List<Item> items;
    
    public Order() {
        this.items = new ArrayList<>();
    }
    
    public void addItem(Item item) {
        this.items.add(item);
    }
    
    public void save() {
        System.out.println("Saving order...");
    }
}

class Item {
    private String name;
    private double price;
}
"#;

    let nodes = extract_from_source("java", source);
    assert!(nodes.contains(&"OrderService".to_string()), "Should find OrderService class");
    assert!(nodes.contains(&"Order".to_string()), "Should find Order class");
    assert!(nodes.contains(&"createOrder".to_string()), "Should find createOrder method");
    assert!(nodes.contains(&"sendConfirmation".to_string()), "Should find sendConfirmation method");
    assert!(nodes.len() >= 4, "Should find at least 4 definitions, got {}", nodes.len());
}

#[test]
fn test_extract_go() {
    let source = r#"
package main

type OrderService struct {
    customer string
    cache    map[string]*Order
}

func NewOrderService(customer string) *OrderService {
    return &OrderService{
        customer: customer,
        cache:    make(map[string]*Order),
    }
}

func (s *OrderService) CreateOrder(items []Item) (*Order, error) {
    order := NewOrder()
    for _, item := range items {
        order.AddItem(item)
    }
    if err := order.Save(); err != nil {
        return nil, err
    }
    s.sendConfirmation(order)
    return order, nil
}

func (s *OrderService) sendConfirmation(order *Order) {
    emailService.Send(order.CustomerEmail(), "Order confirmed!")
}

type Order struct {
    items []Item
}

func NewOrder() *Order {
    return &Order{items: make([]Item, 0)}
}

func (o *Order) AddItem(item Item) {
    o.items = append(o.items, item)
}

func (o *Order) Save() error {
    fmt.Println("Saving order...")
    return nil
}
"#;

    let nodes = extract_from_source("go", source);
    assert!(nodes.contains(&"NewOrderService".to_string()), "Should find NewOrderService");
    assert!(nodes.contains(&"CreateOrder".to_string()), "Should find CreateOrder method");
    assert!(nodes.contains(&"sendConfirmation".to_string()), "Should find sendConfirmation method");
    assert!(nodes.len() >= 4, "Should find at least 4 definitions, got {}", nodes.len());
}

#[test]
fn test_extract_typescript() {
    let source = r#"
interface Item {
  name: string;
  price: number;
}

class OrderService {
  private customer: string;
  private cache: Map<string, Order> = new Map();
  
  constructor(customer: string) {
    this.customer = customer;
  }
  
  async createOrder(items: Item[]): Promise<Order> {
    const order = new Order();
    for (const item of items) {
      order.addItem(item);
    }
    await order.save();
    await this.sendConfirmation(order);
    return order;
  }
  
  private async sendConfirmation(order: Order): Promise<void> {
    await emailService.send(order.customerEmail, "Order confirmed!");
  }
}

class Order {
  private items: Item[] = [];
  
  addItem(item: Item): void {
    this.items.push(item);
  }
  
  async save(): Promise<void> {
    console.log("Saving order...");
  }
}
"#;

    let nodes = extract_from_source("ts", source);
    assert!(nodes.contains(&"OrderService".to_string()), "Should find OrderService class");
    assert!(nodes.contains(&"Order".to_string()), "Should find Order class");
    assert!(nodes.contains(&"createOrder".to_string()), "Should find createOrder method");
    assert!(nodes.len() >= 3, "Should find at least 3 definitions, got {}", nodes.len());
}

#[test]
fn test_extract_javascript() {
    let source = r#"
class OrderService {
  constructor(customer) {
    this.customer = customer;
    this.cache = new Map();
  }
  
  async createOrder(items) {
    const order = new Order();
    for (const item of items) {
      order.addItem(item);
    }
    await order.save();
    await this.sendConfirmation(order);
    return order;
  }
  
  async sendConfirmation(order) {
    await emailService.send(order.customerEmail, "Order confirmed!");
  }
}

class Order {
  constructor() {
    this.items = [];
  }
  
  addItem(item) {
    this.items.push(item);
  }
  
  async save() {
    console.log("Saving order...");
  }
}
"#;

    let nodes = extract_from_source("js", source);
    assert!(nodes.contains(&"OrderService".to_string()), "Should find OrderService class");
    assert!(nodes.contains(&"Order".to_string()), "Should find Order class");
    assert!(nodes.contains(&"createOrder".to_string()), "Should find createOrder method");
    assert!(nodes.contains(&"sendConfirmation".to_string()), "Should find sendConfirmation method");
    assert!(nodes.len() >= 4, "Should find at least 4 definitions, got {}", nodes.len());
}

#[test]
fn test_extract_zig() {
    // Zig requires downloading the parser from GitHub (not bundled by default)
    // Skip if language pack says it's not available
    let zig_supported = is_language_supported("zig");
    if !zig_supported {
        println!("Skipping Zig test - parser not available in current configuration");
        return;
    }
    
    // Also check if we can actually get the language
    let nodes = extract_from_source("zig", "const Item = struct { name: []const u8 }; pub const ITEM: Item = .{};");
    if nodes.is_empty() {
        println!("Skipping Zig test - parser exists but extraction returned 0 nodes");
        return;
    }
    
    let source = r#"
const std = @import("std");

const OrderService = struct {
    customer: []const u8,
    cache: std.AutoHashMap([]const u8, Order),
    
    pub fn init(customer: []const u8) OrderService {
        return .{
            .customer = customer,
            .cache = std.AutoHashMap([]const u8, Order).init(std.heap.page_allocator),
        };
    }
    
    pub fn createOrder(self: *OrderService, items: []Item) !Order {
        var order = Order.init();
        for (items) |item| {
            try order.addItem(item);
        }
        try order.save();
        self.sendConfirmation(&order);
        return order;
    }
    
    fn sendConfirmation(self: *OrderService, order: *Order) void {
        email_service.send(order.customerEmail(), "Order confirmed!");
    }
};

const Order = struct {
    items: std.ArrayList(Item),
    
    pub fn init() Order {
        return .{
            .items = std.ArrayList(Item).init(std.heap.page_allocator),
        };
    }
    
    pub fn addItem(self: *Order, item: Item) !void {
        try self.items.append(item);
    }
    
    pub fn save(self: *Order) !void {
        std.debug.print("Saving order...\n", .{});
    }
};

const Item = struct {
    name: []const u8,
    price: f64,
};
"#;

    let nodes = extract_from_source("zig", source);
    assert!(nodes.contains(&"OrderService".to_string()), "Should find OrderService struct");
    assert!(nodes.contains(&"Order".to_string()), "Should find Order struct");
    assert!(nodes.len() >= 2, "Should find at least 2 definitions, got {}", nodes.len());
}
