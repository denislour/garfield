luu y moi 1 task ben duoi, neu hoan thanh xong hay commit no, va nho luu y phai viet test, build thanh cong + test pass moi dc lam tiếp

1. check giúp tôi trong project các field, folder nào không sử dụng thì hãy remove dọn dẹp giúp tôi, ví dụ config.toml.exmaple và config 2 cai này có vẻ liên quan đến mcp nhưng tôi k sử dụng mcp


2. thay the toan bo louvain thanh leiden giup toi (theo ban nen tach ra 1 fiel leiden.rs) khong chi chua algroithm de noi khac goi
3. thay doi cluster thanh community cho phu hop voi algorithm
4. cac file test tôi muôn tổ chức lại theo
─────────────────────────────────────────────────────────────────────────────┤
   │                                                                             │
   │  PROJECT LỚN (> 15 modules):                                            │
   │  ┌─────────────────────────────────────────────────────────────────────┐  │
   │  │  tests/                                                              │  │
   │  │  ├── cluster/                                                        │  │
   │  │  │   ├── integration_test.rs                                        │  │
   │  │  │   └── fixtures/                                                  │  │
   │  │  │       └── sample_code.rs                                         │  │
   │  │  ├── semantic/                                                        │  │
   │  │  │   └── integration_test.rs                                        │  │
   │  │  ├── detect/                                                         │  │
   │  │  │   └── integration_test.rs                                        │  │
   │  │  └── e2e/                                                            │  │
   │  │      └── full_build_test.rs                                         │  │
   │  └─────────────────────────────────────────────────────────────────────┘  │
   │  → CÁCH 1: Thư mục con theo module ✅                                    │
   │                                                                             │

5. 3-TIER LAZY LOADING (Storage) ✅ DONE
─────────────────────────────────────────────────────────────────────────────────
Purpose: Tách biệt lưu trữ content để tối ưu BUILD vs QUERY

┌─────────────────────────────────────────────────────────────────────────────┐
│  TIER 1: Metadata    → QUERY (always in memory)                            │
│  TIER 2: Summary    → QUERY (fast, cached)                                │
│  TIER 3: Full Body → BUILD (extract, then DISCARDED)                     │
└─────────────────────────────────────────────────────────────────────────────┘

Build: TIER 1+2+3 → Extract body → Generate summary → Keep TIER 1+2
Query: TIER 1+2 → ENOUGH (95% queries)

┌─────────────────────────────────────────────────────────────────────────────┐
│  FILE STRUCTURE:                                                           │
│  garfield-out/                                                              │
│  ├── graph.json          # TIER 1: Metadata (SHIPPED)                     │
│  ├── file_summaries.json # TIER 2: Summaries (SHIPPED)                    │
│  └── body_cache/         # TIER 3: Full Bodies (NOT SHIPPED)               │
└─────────────────────────────────────────────────────────────────────────────┘

6. HYPEREDGE DETECTION (Relationship)
─────────────────────────────────────────────────────────────────────────────────
Purpose: Nhóm 3+ nodes cùng làm 1 việc (không cần LLM)

┌─────────────────────────────────────────────────────────────────────────────┐
│  ALGORITHMS:                                                               │
│  1. File-Based     → O(n) - Group nodes by source file                     │
│  2. Call Chain     → O(n²) - Find A→B→C→D chains                          │
│  3. Config Pattern → O(n) - K8s, Docker, Terraform                        │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│  SAFETY:                                                                    │
│  • Cycle detection (visited set)                                           │
│  • Size limits: 3-20 nodes/hyperedge                                       │
│  • Score threshold: >= 0.3                                                 │
│  • Deduplication by node set                                               │
└─────────────────────────────────────────────────────────────────────────────┘

```rust
// src/hyperedge.rs - CORE

pub fn detect_file_groups(graph: &GraphData) -> Vec<HyperedgeCandidate> {
    let mut by_file: HashMap<String, Vec<&Node>> = HashMap::new();
    for node in &graph.nodes {
        by_file.entry(node.source_file.clone()).or_default().push(node);
    }

    by_file.into_iter()
        .filter(|(_, nodes)| nodes.len() >= 3 && nodes.len() <= 50)
        .map(|(file, nodes)| HyperedgeCandidate {
            id: format!("file_{}", Path::new(&file).file_stem().unwrap_or_default().to_string_lossy()),
            label: format!("{} module", file),
            nodes: nodes.iter().map(|n| n.id.clone()).collect(),
            relation: "participate_in".to_string(),
            confidence: Confidence::Inferred,
            source_file: file,
            score: calculate_cohesion(&nodes, graph),
        })
        .collect()
}

pub fn detect_call_chains(graph: &GraphData) -> Vec<HyperedgeCandidate> {
    let adj: HashMap<&str, Vec<&str>> = graph.links.iter()
        .filter(|e| e.relation == "calls")
        .fold(HashMap::new(), |mut acc, edge| {
            acc.entry(edge.source.as_str()).or_default().push(edge.target.as_str()); acc
        });

    let mut candidates = Vec::new();
    for start in graph.nodes.iter().map(|n| n.id.as_str()) {
        find_chains_dfs(start, &adj, &mut HashSet::new(), &mut vec![start], graph, &mut candidates);
    }
    candidates
}

fn find_chains_dfs(current: &str, adj: &HashMap<&str, Vec<&str>>, visited: &mut HashSet<&str>, chain: &mut Vec<&str>, graph: &GraphData, candidates: &mut Vec<HyperedgeCandidate>) {
    visited.insert(current);
    if let Some(neighbors) = adj.get(current) {
        for &neighbor in neighbors {
            if !visited.contains(neighbor) {
                chain.push(neighbor);
                if chain.len() >= 3 && chain.len() <= 10 {
                    candidates.push(HyperedgeCandidate {
                        id: format!("chain_{}", candidates.len()),
                        label: format!("Call Chain ({})", chain.len()),
                        nodes: chain.iter().map(|s| s.to_string()).collect(),
                        relation: "call_chain".to_string(),
                        confidence: Confidence::Extracted,
                        score: chain_cohesion(chain, graph),
                        ..Default::default()
                    });
                }
                find_chains_dfs(neighbor, adj, visited, chain, graph, candidates);
                chain.pop();
            }
        }
    }
    visited.remove(current);
}

pub fn process_candidates(mut candidates: Vec<HyperedgeCandidate>) -> Vec<Hyperedge> {
    // Deduplicate
    let mut seen = HashSet::new();
    candidates.retain(|c| { let k = c.nodes.iter().sorted().join("|"); seen.insert(k) });
    // Filter
    candidates.retain(|c| c.score >= 0.3 && c.nodes.len() >= 3 && c.nodes.len() <= 20);
    candidates.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
    candidates.into_iter().map(|c| c.into_hyperedge()).collect()
}
```

┌─────────────────────────────────────────────────────────────────────────────┐
│  ORDER DOMAIN HYPEREDGE EXAMPLES:                                         │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  HYPEREDGE = "Nhóm functions cùng làm 1 DOMAIN LOGIC"                  │
│  (Không chỉ file-based, mà là CROSS-FILE business flow)                 │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐  │
│  │  1. CREATE ORDER FLOW:                                              │  │
│  │  ───────────────────────────────────────────────────────────────     │  │
│  │  create_order → validate_items → reserve_stock → apply_discount   │  │
│  │  → calculate_total → save_order → send_confirmation              │  │
│  │                                                                      │  │
│  │  Hyperedge: "order/create_order_flow"                            │  │
│  │  Files: order_service.py, inventory.py, discount.py, notify.py   │  │
│  │  Nodes: 7 functions (cross-file)                                 │  │
│  │  Confidence: 0.92                                                 │  │
│  └─────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐  │
│  │  2. INSERT COUPON FLOW:                                            │  │
│  │  ───────────────────────────────────────────────────────────────     │  │
│  │  insert_coupon → validate_coupon → check_usage → check_expiry    │  │
│  │  → calculate_discount → apply_discount → update_order           │  │
│  │                                                                      │  │
│  │  Hyperedge: "order/insert_coupon_flow"                           │  │
│  │  Files: order_service.py, discount.py                            │  │
│  │  Nodes: 7 functions (cross-file)                                  │  │
│  │  Confidence: 0.88                                                 │  │
│  └─────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐  │
│  │  3. PROCESS PAYMENT FLOW:                                          │  │
│  │  ───────────────────────────────────────────────────────────────     │  │
│  │  process_payment → authorize_card → validate_card → charge_card  │  │
│  │  → capture_payment → record_payment → send_receipt → update_order │  │
│  │                                                                      │  │
│  │  Hyperedge: "order/process_payment_flow"                          │  │
│  │  Files: payment.py, notification.py, order_service.py            │  │
│  │  Nodes: 8 functions (cross-file)                                   │  │
│  │  Confidence: 0.95                                                 │  │
│  └─────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐  │
│  │  4. FILE-BASED (Simple):                                           │  │
│  │  ───────────────────────────────────────────────────────────────     │  │
│  │  src/order_service.py                                             │  │
│  │  → Hyperedge: "order_service module"                              │  │
│  │  → Nodes: create_order, insert_coupon, calculate_total, ...       │  │
│  │  → Type: participate_in (INFERRED, 0.85)                         │  │
│  └─────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│  HYPEREDGE OUTPUT EXAMPLES:                                              │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐  │
│  │  FILE-BASED (TIER 1):                                              │  │
│  │  {                                                                 │  │
│  │    "id": "file_order_service",                                    │  │
│  │    "label": "order_service module",                              │  │
│  │    "nodes": ["create_order", "insert_coupon", "calculate_total"],│  │
│  │    "relation": "participate_in",                                 │  │
│  │    "confidence": "INFERRED",                                     │  │
│  │    "score": 0.85                                                  │  │
│  │  }                                                                 │  │
│  └─────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐  │
│  │  DOMAIN FLOW (Cross-file):                                         │  │
│  │  {                                                                 │  │
│  │    "id": "domain_create_order",                                   │  │
│  │    "label": "Create Order Flow",                                  │  │
│  │    "nodes": [                                                      │  │
│  │      "order_service.create_order",                                │  │
│  │      "inventory.validate_items",                                   │  │
│  │      "inventory.reserve_stock",                                    │  │
│  │      "discount.apply_discount",                                    │  │
│  │      "order_service.calculate_total",                             │  │
│  │      "order_service.save_order",                                  │  │
│  │      "notification.send_confirmation"                              │  │
│  │    ],                                                               │  │
│  │    "relation": "domain_flow",                                     │  │
│  │    "confidence": "EXTRACTED",                                     │  │
│  │    "score": 0.92,                                                  │  │
│  │    "flow_type": "create_order"                                   │  │
│  │  }                                                                 │  │
│  └─────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

7. COMBINED: 3-TIER + HYPEREDGE
─────────────────────────────────────────────────────────────────────────────────

┌─────────────────────────────────────────────────────────────────────────────┐
│  BUILD PIPELINE:                                                           │
│  INPUT → TIER1(Metadata) → TIER3(Extract Body) → TIER2(Summary) → HYPEREDGE │
│  OUTPUT: graph.json(T1+HYPER) + file_summaries.json(T2)                    │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│  QUERY PIPELINE:                                                           │
│  TIER1+TIER2 → Fast (metadata + summary)                                  │
│  TIER3 → On-demand (đọc trực tiếp từ source files)                         │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│  INTEGRATION:                                                              │
│  // src/lib.rs                                                             │
│  pub fn run_build(...) -> Result<BuildSummary> {                          │
│      let mut graph = build_graph(ast_extractions);                         │
│      let summaries = generate_file_summaries(&graph)?;                     │
│      save_file_summaries(&summaries, &output_path.join("file_summaries.json"))?;│
│      graph.hyperedges = hyperedge::detect_hyperedges(&graph, root_path)?;   │
│      to_json(&graph, &output_path.join("graph.json"))?;                    │
│      BuildSummary { hyperedges: graph.hyperedges.len(), ... }             │
│  }                                                                         │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│  CLI:                                                                      │
│  garfield build ./src                      # Full build + hyperedge         │
│  garfield build ./src --hyperedge=file    # File groups only               │
│  garfield build ./src --hyperedge=chain   # Call chains only               │
│  garfield build ./src --hyperedge=config  # Config patterns only           │
└─────────────────────────────────────────────────────────────────────────────┘

8. E2E TEST với mock trên ✅ DONE
─────────────────────────────────────────────────────────────────────────────────

┌─────────────────────────────────────────────────────────────────────────────┐
│  TEST STRUCTURE (tests/e2e/full_build_test.rs):                          │
│  ├── build_with_hyperedge()          # garfield build ./src              │
│  ├── query_by_function_name()        # garfield query "calculate_price" │
│  ├── query_by_file_summary()         # garfield summary "src/pricing.rs" │
│  ├── query_body_on_demand()         # garfield body "src/pricing.rs:fn"  │
│  ├── query_path()                   # garfield path "A" "B"              │
│  └── explain_hyperedge()           # garfield explain "pricing_module"    │
└─────────────────────────────────────────────────────────────────────────────┘

```rust
// tests/e2e/full_build_test.rs

#[cfg(test)]
mod tests {
    use garfield::GraphData;
    use std::path::Path;
    use std::fs;

    fn setup_test_project() -> PathBuf {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("pricing.rs"), r#"
            pub fn calculate_price(qty: i32, tier: &str, region: &str) -> f64 {
                let discount_tier = get_discount_tier(qty);
                let base_price = 10.0 * qty as f64;
                let discounted = apply_discount(base_price, discount_tier);
                compute_tax(discounted, region)
            }
            fn apply_discount(price: f64, tier: DiscountTier) -> f64 { price * 0.9 }
            fn compute_tax(amount: f64, region: &str) -> f64 { amount * 1.1 }
            fn get_discount_tier(quantity: i32) -> DiscountTier { DiscountTier::Bronze }
            enum DiscountTier { Bronze, Silver, Gold }
        "#).unwrap();
        tmp.into_path()
    }

    #[test]
    fn test_build_with_hyperedge() {
        let tmp = setup_test_project();
        let output = tmp.join("out");

        garfield::run_build(tmp.to_str().unwrap(), output.to_str().unwrap(), false).unwrap();

        let graph: GraphData = serde_json::from_str(&fs::read_to_string(output.join("graph.json")).unwrap()).unwrap();
        assert!(!graph.nodes.is_empty());
        assert!(!graph.hyperedges.is_empty()); // HYPEREDGE DETECTED

        let summaries: HashMap<String, FileSummary> = serde_json::from_str(
            &fs::read_to_string(output.join("file_summaries.json")).unwrap()
        ).unwrap();
        assert!(summaries.contains_key("src/pricing.rs"));
    }

    #[test]
    fn test_query_returns_tier1_tier2_hyperedge() {
        let output = run_build();
        let graph: GraphData = load_graph(output.join("graph.json"));
        let summaries = load_summaries(output.join("file_summaries.json"));

        let result = garfield::query("calculate_price", &graph, &summaries);

        assert!(result.node.is_some()); // TIER 1
        assert!(result.summary.is_some()); // TIER 2
        assert!(result.hyperedge.is_some()); // HYPEREDGE
    }

    #[test]
    fn test_summary_loads_tier2_only() {
        let summaries = load_summaries("garfield-out/file_summaries.json");
        let summary = garfield::get_file_summary("src/pricing.rs", &summaries).unwrap();
        assert_eq!(summary.function_count, 4);
    }

    #[test]
    fn test_body_on_demand_from_source() {
        // Body đọc từ source file, không cache
        let body = garfield::get_body("src/pricing.rs", "calculate_price").unwrap();
        assert!(body.contains("fn calculate_price"));
    }

    #[test]
    fn test_path_finding_uses_tier1() {
        let graph = load_graph("garfield-out/graph.json");
        let path = garfield::find_path("validate_order", "process_payment", &graph);
        assert!(path.len() >= 2);
    }

    #[test]
    fn test_hyperedge_explains_group() {
        let graph = load_graph("garfield-out/graph.json");
        let hyperedge = garfield::explain_hyperedge("pricing_module", &graph).unwrap();
        assert!(hyperedge.nodes.len() >= 3);
        assert!(hyperedge.score >= 0.3);
    }
}
```

┌─────────────────────────────────────────────────────────────────────────────┐
│  EXPECTED OUTPUT sau build:                                               │
│  garfield-out/                                                              │
│  ├── graph.json              # TIER 1 + HYPEREDGE                        │
│  ├── file_summaries.json     # TIER 2                                    │
│  └── GRAPH_REPORT.md         # Report                                    │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│  EXPECTED: query "calculate_price"                                       │
│  ┌─────────────────────────────────────────────────────────────────────┐  │
│  │  FUNCTION: calculate_price | FILE: src/pricing.rs:L42             │  │
│  │  SUMMARY (TIER 2): Computes final price with quantity discount    │  │
│  │  MODULE (HYPEREDGE): pricing module (4 functions)                  │  │
│  └─────────────────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────────┘

   de lam giảm thiểu code trong file rs


 Mock E2E: garfield build + query

 ```
   ╔═══════════════════════════════════════════════════════════════════════════════╗
   ║                        garfield build ./src                                   ║
   ╚═══════════════════════════════════════════════════════════════════════════════╝

   ┌─────────────────────────────────────────────────────────────────────────────┐
   │  OUTPUT: garfield-out/                                                       │
   │  ├── graph.json              # TIER 1: Metadata + HYPEREDGES                │
   │  ├── file_summaries.json    # TIER 2: File Summaries                       │
   │  └── GRAPH_REPORT.md         # Report                                       │
   └─────────────────────────────────────────────────────────────────────────────┘
 ```

 ────────────────────────────────────────────────────────────────────────────────

 ### 1. graph.json (TIER 1 + HYPEREDGE)

 ```json
   {
     "nodes": [
       {
         "id": "src/pricing.rs:calculate_price",
         "label": "calculate_price",
         "source_file": "src/pricing.rs",
         "source_location": "L42",
         "community": 5,
         "summary": "Computes final price with quantity discount"
       },
       {
         "id": "src/pricing.rs:apply_discount",
         "label": "apply_discount",
         "source_file": "src/pricing.rs",
         "source_location": "L78",
         "community": 5,
         "summary": "Applies percentage discount to price"
       },
       {
         "id": "src/pricing.rs:compute_tax",
         "label": "compute_tax",
         "source_file": "src/pricing.rs",
         "source_location": "L115",
         "community": 5,
         "summary": "Calculates tax based on region code"
       },
       {
         "id": "src/pricing.rs:get_discount_tier",
         "label": "get_discount_tier",
         "source_file": "src/pricing.rs",
         "source_location": "L156",
         "community": 5,
         "summary": "Returns tier (bronze|silver|gold) based on quantity"
       }
     ],
     "links": [
       {"source": "src/pricing.rs:calculate_price", "target": "src/pricing.rs:apply_discount", "relation": "calls", "confidence": "EXTRACTED"},
       {"source": "src/pricing.rs:calculate_price", "target": "src/pricing.rs:compute_tax", "relation": "calls", "confidence": "EXTRACTED"},
       {"source": "src/pricing.rs:calculate_price", "target": "src/pricing.rs:get_discount_tier", "relation": "calls", "confidence": "EXTRACTED"}
     ],
     "hyperedges": [
       {
         "id": "file_pricing",
         "label": "pricing module",
         "nodes": ["calculate_price", "apply_discount", "compute_tax", "get_discount_tier"],
         "relation": "participate_in",
         "confidence": "INFERRED",
         "confidence_score": 0.85,
         "source_file": "src/pricing.rs"
       },
       {
         "id": "chain_0",
         "label": "Call Chain (4)",
         "nodes": ["validate_order", "calculate_price", "apply_discount", "process_payment"],
         "relation": "call_chain",
         "confidence": "EXTRACTED",
         "confidence_score": 0.95,
         "source_file": "src/order.rs"
       }
     ],
     "metadata": {
       "total_nodes": 170,
       "total_edges": 162,
       "communities": 20,
       "hyperedges": 15
     }
   }
 ```

 ────────────────────────────────────────────────────────────────────────────────

 ### 2. file_summaries.json (TIER 2)

 ```json
   {
     "src/pricing.rs": {
       "filename": "src/pricing.rs",
       "summary": "Handles pricing calculations with quantity discounts and tier-based pricing",
       "function_count": 4,
       "functions": [
         "calculate_price(qty: i32, tier: &str, region: &str) -> f64",
         "apply_discount(price: f64, discount: f64) -> f64",
         "compute_tax(amount: f64, region: &str) -> f64",
         "get_discount_tier(quantity: i32) -> DiscountTier"
       ],
       "public_apis": ["calculate_price", "compute_tax"],
       "dependencies": ["src/models.rs", "src/config.rs"],
       "internal_functions": ["apply_discount", "get_discount_tier"]
     }
   }
 ```

 ────────────────────────────────────────────────────────────────────────────────

 ```
   ╔═══════════════════════════════════════════════════════════════════════════════╗
   ║                        garfield query "calculate_price"                        ║
   ╚═══════════════════════════════════════════════════════════════════════════════╝

   ┌─────────────────────────────────────────────────────────────────────────────┐
   │  TIER 1: Node found (metadata)                                              │
   │  TIER 2: File summary loaded (context)                                     │
   │  HYPEREDGE: "pricing module" group returned                                │
   └─────────────────────────────────────────────────────────────────────────────┘

   ═══════════════════════════════════════════════════════════════════════════════
     FUNCTION: calculate_price
     FILE:     src/pricing.rs:L42
     MODULE:   pricing module (HYPEREDGE)
   ═══════════════════════════════════════════════════════════════════════════════

     Summary:
     ┌─────────────────────────────────────────────────────────────────────────┐
     │  Computes final price with quantity discount                            │
     └─────────────────────────────────────────────────────────────────────────┘

     Callers (TIER 1 - edges):
     ┌─────────────────────────────────────────────────────────────────────────┐
     │  • src/order.rs:validate_order         → calls calculate_price         │
     │  • src/order.rs:process_payment        → calls calculate_price         │
     └─────────────────────────────────────────────────────────────────────────┘

     Calls to (TIER 1 - edges):
     ┌─────────────────────────────────────────────────────────────────────────┐
     │  • apply_discount    → calls (internal)                                │
     │  • compute_tax       → calls (internal)                                │
     │  • get_discount_tier → calls (internal)                                │
     └─────────────────────────────────────────────────────────────────────────┘

     Module: pricing (HYPEREDGE)
     ┌─────────────────────────────────────────────────────────────────────────┐
     │  Nhóm 3+ functions cùng làm việc:                                     │
     │  • calculate_price     ← YOU ARE HERE                                 │
     │  • apply_discount                                                   │
     │  • compute_tax                                                      │
     │  • get_discount_tier                                                │
     └─────────────────────────────────────────────────────────────────────────┘

     File context (TIER 2 - summary):
     ┌─────────────────────────────────────────────────────────────────────────┐
     │  Handles pricing calculations with quantity discounts and             │
     │  tier-based pricing. 4 functions, 2 public APIs.                       │
     │  Dependencies: src/models.rs, src/config.rs                          │
     └─────────────────────────────────────────────────────────────────────────┘
 ```

 ────────────────────────────────────────────────────────────────────────────────

 ```
   ╔═══════════════════════════════════════════════════════════════════════════════╗
   ║                        garfield summary "src/pricing.rs"                      ║
   ╚═══════════════════════════════════════════════════════════════════════════════╝

   ┌─────────────────────────────────────────────────────────────────────────────┐
   │  TIER 2: File summary loaded (ONLY)                                        │
   │  TIER 1: Not needed (just want overview)                                   │
   └─────────────────────────────────────────────────────────────────────────────┘

   ═══════════════════════════════════════════════════════════════════════════════
     FILE:     src/pricing.rs
     SUMMARY:  Handles pricing calculations with quantity discounts and
               tier-based pricing
     FUNCTIONS: 4
   ═══════════════════════════════════════════════════════════════════════════════

     Public APIs (TIER 2):
     ┌─────────────────────────────────────────────────────────────────────────┐
     │  • calculate_price(qty: i32, tier: &str, region: &str) -> f64        │
     │  • compute_tax(amount: f64, region: &str) -> f64                      │
     └─────────────────────────────────────────────────────────────────────────┘

     Internal (TIER 2):
     ┌─────────────────────────────────────────────────────────────────────────┐
     │  • apply_discount(price: f64, discount: f64) -> f64                  │
     │  • get_discount_tier(quantity: i32) -> DiscountTier                   │
     └─────────────────────────────────────────────────────────────────────────┘

     Dependencies (TIER 2):
     ┌─────────────────────────────────────────────────────────────────────────┐
     │  • src/models.rs                                                       │
     │  • src/config.rs                                                      │
     └─────────────────────────────────────────────────────────────────────────┘
 ```

 ────────────────────────────────────────────────────────────────────────────────

 ```
   ╔═══════════════════════════════════════════════════════════════════════════════╗
   ║               garfield body "src/pricing.rs:calculate_price"                 ║
   ╚═══════════════════════════════════════════════════════════════════════════════╝

   ┌─────────────────────────────────────────────────────────────────────────────┐
   │  TIER 3: On-demand load from source files (NOT cached)                     │
   │  Đọc trực tiếp từ src/pricing.rs gốc                                     │
   └─────────────────────────────────────────────────────────────────────────────┘

   ═══════════════════════════════════════════════════════════════════════════════
     FUNCTION: calculate_price
     FILE:     src/pricing.rs:L42-68
   ═══════════════════════════════════════════════════════════════════════════════

     Source:
     ┌─────────────────────────────────────────────────────────────────────────┐
     │  pub fn calculate_price(                                              │
     │      qty: i32,                                                        │
     │      tier: &str,                                                      │
     │      region: &str,                                                    │
     │  ) -> f64 {                                                           │
     │      // Get discount tier based on quantity                           │
     │      let discount_tier = get_discount_tier(qty);                      │
     │                                                                       │
     │      // Calculate base price                                          │
     │      let base_price = BASE_PRICE * qty as f64;                        │
     │                                                                       │
     │      // Apply tier discount                                           │
     │      let discounted = apply_discount(base_price, discount_tier);      │
     │                                                                       │
     │      // Add tax                                                       │
     │      let final_price = compute_tax(discounted, region);               │
     │                                                                       │
     │      final_price                                                     │
     │  }                                                                    │
     └─────────────────────────────────────────────────────────────────────────┘
 ```

 ────────────────────────────────────────────────────────────────────────────────

 ```
   ╔═══════════════════════════════════════════════════════════════════════════════╗
   ║                     garfield path "validate_order" "process_payment"          ║
   ╚═══════════════════════════════════════════════════════════════════════════════╝

   ┌─────────────────────────────────────────────────────────────────────────────┐
   │  TIER 1: BFS path finding (metadata only)                                   │
   │  HYPEREDGE: Call chain highlighted                                          │
   └─────────────────────────────────────────────────────────────────────────────┘

     PATH (3 hops):
     ┌─────────────────────────────────────────────────────────────────────────┐
     │  validate_order                                                        │
     │      │                                                                │
     │      ├──(calls)──→ calculate_price                                    │
     │      │                   │                                           │
     │      │                   ├──(calls)──→ apply_discount                 │
     │      │                   │                   │                        │
     │      │                   │                   └──(calls)──→ ...        │
     │      │                   │                                           │
     │      │                   └──(calls)──→ compute_tax                   │
     │      │                                   │                            │
     │      │                                   └──(calls)──→ ...            │
     │      │                                                                │
     │      └──(calls)──→ process_payment                                    │
     │                                                                       │
     │  FULL PATH: validate_order → calculate_price → apply_discount →        │
     │             compute_tax → process_payment (5 nodes)                   │
     └─────────────────────────────────────────────────────────────────────────┘

     HYPEREDGE FOUND: chain_0
     ┌─────────────────────────────────────────────────────────────────────────┐
     │  Call Chain (4): validate_order → calculate_price → apply_discount →     │
     │                  process_payment                                       │
     │  Confidence: 0.95 (EXTRACTED)                                         │
     └─────────────────────────────────────────────────────────────────────────┘
 ```

 ────────────────────────────────────────────────────────────────────────────────

 ```
   ╔═══════════════════════════════════════════════════════════════════════════════╗
   ║                 garfield explain "pricing_module"                             ║
   ╚═══════════════════════════════════════════════════════════════════════════════╝

   ┌─────────────────────────────────────────────────────────────────────────────┐
   │  HYPEREDGE: "pricing module"                                                │
   │  Detection: File-Based (automatic)                                         │
   │  Confidence: 0.85 (INFERRED)                                              │
   └─────────────────────────────────────────────────────────────────────────────┘

   ═══════════════════════════════════════════════════════════════════════════════
     HYPEREDGE: pricing module
     SOURCE:    src/pricing.rs (3-50 nodes)
     NODES:     4 functions
   ═══════════════════════════════════════════════════════════════════════════════

     MEMBERS:
     ┌─────────────────────────────────────────────────────────────────────────┐
     │  ┌─────────────────┬─────────────────────────────────────────────────┐ │
     │  │ Function        │ Summary                                       │ │
     │  ├─────────────────┼─────────────────────────────────────────────────┤ │
     │  │ calculate_price │ Computes final price with discount            │ │
     │  │ apply_discount  │ Applies percentage discount to price           │ │
     │  │ compute_tax     │ Calculates tax based on region code           │ │
     │  │ get_discount_tier │ Returns tier based on quantity             │ │
     │  └─────────────────┴─────────────────────────────────────────────────┘ │
     └─────────────────────────────────────────────────────────────────────────┘

     COHESION: 0.85
     ┌─────────────────────────────────────────────────────────────────────────┐
     │  Internal edges: 3 (calculate_price → apply_discount, compute_tax,     │
     │                   get_discount_tier)                                   │
     │  External edges: 1 (calculate_price ← validate_order)                  │
     └─────────────────────────────────────────────────────────────────────────┘
 ```

 ────────────────────────────────────────────────────────────────────────────────

 Tổng kết: TIER → OUTPUT mapping

 ```
   ┌─────────────────────────────────────────────────────────────────────────────┐
   │                        QUERY TYPE → TIER USAGE                             │
   ├─────────────────────────────────────────────────────────────────────────────┤
   │                                                                             │
   │  garfield query "fn_name"        → TIER 1 + TIER 2 + HYPEREDGE            │
   │  garfield summary "file"        → TIER 2 ONLY (fast!)                    │
   │  garfield body "file:fn"       → TIER 3 (on-demand from source)          │
   │  garfield path "A" "B"         → TIER 1 + HYPEREDGE                      │
   │  garfield explain "hyperedge"   → HYPEREDGE                              │
   │                                                                             │
   │  MEMORY:                                                                  │
   │  • TIER 1: ~50MB (always in memory)                                       │
   │  • TIER 2: ~50MB (cached)                                                 │
   │  • TIER 3: 0MB (on-demand, not cached)                                    │
   │                                                                             │
   └─────────────────────────────────────────────────────────────────────────────┘

---

## 9. INCREMENTAL BUILD
─────────────────────────────────────────────────────────────────────────────────
Purpose: Cache build, chỉ re-parse files thay đổi

┌─────────────────────────────────────────────────────────────────────────────┐
│  CURRENT (FULL BUILD):                                                    │
│  • Mỗi lần build: Parse tất cả files → Chậm với codebase lớn             │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│  INCREMENTAL (CACHED BUILD):                                              │
│  garfield-out/                                                              │
│  ├── graph.json                      # Graph data                          │
│  ├── file_summaries.json            # TIER 2                              │
│  └── cache/                         # THÊM MỚI                            │
│      ├── src_pricing_rs.json        # Hash + summary                      │
│      │    { "hash": "abc123", "summary": "pricing module", ... }         │
│      └── src_order_rs.json                                              │
└─────────────────────────────────────────────────────────────────────────────┘

Build flow:
1. Hash all source files (xxhash - fast)
2. Compare với cache/ timestamps
3. Chỉ re-parse files thay đổi (diff)
4. Merge vào graph.json (không overwrite)
5. Update cache/ với files mới

## 10. CROSS-FILE HYPEREDGE
─────────────────────────────────────────────────────────────────────────────────
Purpose: Module span nhiều files (auth = login.rs + token.rs + middleware.rs)

┌─────────────────────────────────────────────────────────────────────────────┐
│  CURRENT: Hyperedge chỉ trong 1 file (file-based)                        │
│  └── src/pricing.rs → Hyperedge "pricing module" (1 file = 1 module)    │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│  CROSS-FILE: Hyperedge span nhiều files có same module prefix            │
│                                                                             │
│  src/auth/login.rs                                                        │
│  src/auth/token.rs                                                        │
│  src/auth/middleware.rs                                                  │
│  src/auth/mod.rs                                                         │
│  → Tất cả cùng prefix "src/auth/" → HYPEREDGE "auth module"            │
└─────────────────────────────────────────────────────────────────────────────┘

Algorithms:
1. IMPORT CHAIN: A imports B imports C → Group together
2. MODULE PREFIX: Files cùng directory path → Group
3. NAMING PATTERN: *_service.rs, *_controller.rs → Group

## 11. ADVANCED QUERY + RIPGREP INTEGRATION
─────────────────────────────────────────────────────────────────────────────────

┌─────────────────────────────────────────────────────────────────────────────┐
│  RIPGREP vs GARFIELD (KHÔNG TRÙNG LẶP):                                 │
│  ┌─────────────────────────────────────────────────────────────────────┐  │
│  │  Ripgrep: "Tìm code Ở ĐÂU" (WHERE)                                │  │
│  │  Garfield: "Hiểu code LÀ GÌ, LÀM GÌ" (WHAT, WHY, HOW)           │  │
│  └─────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
│  COMPLEMENTARY WORKFLOW:                                                  │
│  1. garfield query "discount" → Tìm functions liên quan                  │
│  2. garfield explain "pricing" → Hiểu module context                    │
│  3. garfield path "A" "B" → Tìm đường đi giữa functions               │
│  4. rg "calculate_price" → Tìm tất cả lines chứa text                   │
└─────────────────────────────────────────────────────────────────────────────┘

Advanced query:
┌─────────────────────────────────────────────────────────────────────────────┐
│  garfield query "discount" --type=function    # Fuzzy match                 │
│  garfield query "price.*tax" --type=regex   # Regex                       │
│  garfield query --community=5               # By community                │
│  garfield query --hyperedge=pricing         # By hyperedge                │
│  garfield query --source=src/auth           # By path                      │
│  garfield query --rank=relevance           # Ranked results               │
└─────────────────────────────────────────────────────────────────────────────┘

## 12. LANGUAGE EXTENSIONS (Why Rust Only?)
─────────────────────────────────────────────────────────────────────────────────

┌─────────────────────────────────────────────────────────────────────────────┐
│  TẠI SAO CHỈ CÓ RUST?                                                    │
│  ┌─────────────────────────────────────────────────────────────────────┐  │
│  │  1. Tree-sitter parser: Mỗi ngôn ngữ cần 1 parser riêng         │  │
│  │  2. tree-sitter-rust: Có sẵn, stable                              │  │
│  │  3. Mvp: Rust project → Rust parser → 1 language first           │  │
│  └─────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
│  ĐỂ THÊM LANGUAGE MỚI CẦN:                                               │
│  ┌─────────────────────────────────────────────────────────────────────┐  │
│  │  1. tree-sitter-<lang> parser (thư viện)                          │  │
│  │  2. Cập nhật src/extract.rs                                       │  │
│  │  3. Cập nhật CLI detect language tự động                          │  │
│  │  4. Test với code mẫu                                             │  │
│  └─────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
│  LANGUAGES ĐƯỢC HỖ TRỢ TRONG TƯƠNG LAI:                                   │
│  ┌─────────────────────────────────────────────────────────────────────┐  │
│  │  Priority 1: Python, TypeScript, Go, Java, C/C++, Ruby           │  │
│  └─────────────────────────────────────────────────────────────────────┘  │
│                                                                             │
│  garfield build ./src --lang=rust,python,go   # Multi-language           │
└─────────────────────────────────────────────────────────────────────────────┘

---

### 12. LANGUAGE EXTENSIONS - SPECIFIC LANGUAGES
─────────────────────────────────────────────────────────────────────────────────

┌─────────────────────────────────────────────────────────────────────────────┐
│  LANGUAGES ĐƯỢC HỖ TRỢ:                                                  │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  1. PYTHON          (ML/Data engineering, Web)                             │
│  2. JAVASCRIPT/TS   (Web, Node.js)                                        │
│  3. RUBY            (Rails, Scripts)                                       │
│  4. GO              (Backend, CLI) - sau                                  │
│  5. JAVA            (Enterprise) - sau                                    │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘

### 12.1 PYTHON EXAMPLE

┌─────────────────────────────────────────────────────────────────────────────┐
│  PYTHON: order_service.py                                                 │
│  Tree-sitter Python parser để extract functions, classes, imports          │
└─────────────────────────────────────────────────────────────────────────────┘

```python
# order_service.py
from dataclasses import dataclass
from typing import List, Optional
from decimal import Decimal

@dataclass
class Order:
    id: str
    items: List[OrderItem]
    coupon_code: Optional[str] = None
    discount: Decimal = Decimal("0")

class OrderService:
    def create_order(self, customer_id: str, items: List[dict]) -> Order:
        """Create new order from cart items"""
        order = Order(
            id=self._generate_order_id(),
            items=[OrderItem(**item) for item in items]
        )
        self._apply_default_discount(order)
        self._save_order(order)
        self._notify_customer(order)
        return order

    def insert_coupon(self, order_id: str, coupon_code: str) -> bool:
        """Apply coupon code to existing order"""
        order = self._get_order(order_id)
        coupon = self._validate_coupon(coupon_code)
        if coupon:
            order.coupon_code = coupon_code
            order.discount = coupon.calculate_discount(order)
            self._save_order(order)
            return True
        return False

    def calculate_total(self, order_id: str) -> Decimal:
        """Calculate final total with all discounts"""
        order = self._get_order(order_id)
        subtotal = sum(item.price * item.quantity for item in order.items)
        discount = order.discount
        tax = self._calculate_tax(subtotal - discount)
        return subtotal - discount + tax

    def _generate_order_id(self) -> str:
        return f"ORD-{uuid.uuid4().hex[:8].upper()}"

    def _apply_default_discount(self, order: Order) -> None:
        """Apply bulk discount if > 5 items"""
        if len(order.items) > 5:
            order.discount = Decimal("0.1")  # 10% off

    def _validate_coupon(self, code: str) -> Optional[Coupon]:
        ...

    def _calculate_tax(self, amount: Decimal) -> Decimal:
        return amount * Decimal("0.08")  # 8% tax

    def _save_order(self, order: Order) -> None:
        ...

    def _get_order(self, order_id: str) -> Order:
        ...
```

┌─────────────────────────────────────────────────────────────────────────────┐
│  HYPEREDGE OUTPUT (Python):                                               │
└─────────────────────────────────────────────────────────────────────────────┘

```json
{
  "id": "file_order_service",
  "label": "order_service module",
  "nodes": [
    "OrderService.create_order",
    "OrderService.insert_coupon",
    "OrderService.calculate_total",
    "OrderService._generate_order_id",
    "OrderService._apply_default_discount",
    "OrderService._validate_coupon",
    "OrderService._calculate_tax",
    "OrderService._save_order",
    "OrderService._get_order"
  ],
  "relation": "participate_in",
  "confidence": "INFERRED",
  "source_file": "order_service.py"
}
```

┌─────────────────────────────────────────────────────────────────────────────┐
│  PYTHON PARSER CONFIG:                                                   │
└─────────────────────────────────────────────────────────────────────────────┘

```rust
// Thêm vào Cargo.toml
tree-sitter-python = "0.20"

// Trong extract.rs
pub fn detect_language(path: &Path) -> Language {
    match path.extension().and_then(|e| e.to_str()) {
        Some("py") => tree_sitter_python::LANGUAGE,
        Some("rs") => tree_sitter_rust::LANGUAGE,
        Some("js") | Some("ts") | Some("jsx") | Some("tsx") => tree_sitter_typescript::LANGUAGE,
        Some("rb") => tree_sitter_ruby::LANGUAGE,
        _ => tree_sitter_rust::LANGUAGE, // default
    }
}
```

---

### 12.2 JAVASCRIPT/TYPESCRIPT EXAMPLE

┌─────────────────────────────────────────────────────────────────────────────┐
│  TYPESCRIPT: orderService.ts                                              │
│  Tree-sitter TypeScript parser để extract functions, classes, interfaces   │
└─────────────────────────────────────────────────────────────────────────────┘

```typescript
// orderService.ts
interface OrderItem {
  productId: string;
  quantity: number;
  price: number;
}

interface Order {
  id: string;
  items: OrderItem[];
  couponCode?: string;
  discount: number;
  status: 'pending' | 'paid' | 'shipped';
}

export class OrderService {
  async createOrder(customerId: string, items: OrderItem[]): Promise<Order> {
    const order: Order = {
      id: this.generateOrderId(),
      items,
      discount: 0,
      status: 'pending'
    };

    this.applyDefaultDiscount(order);
    await this.saveOrder(order);
    await this.notifyCustomer(order);

    return order;
  }

  async insertCoupon(orderId: string, couponCode: string): Promise<boolean> {
    const order = await this.getOrder(orderId);
    const coupon = await this.validateCoupon(couponCode);

    if (coupon) {
      order.couponCode = couponCode;
      order.discount = coupon.calculateDiscount(order);
      await this.saveOrder(order);
      return true;
    }

    return false;
  }

  async calculateTotal(orderId: string): Promise<number> {
    const order = await this.getOrder(orderId);
    const subtotal = order.items.reduce((sum, item) => sum + item.price * item.quantity, 0);
    const tax = this.calculateTax(subtotal - order.discount);
    return subtotal - order.discount + tax;
  }

  private generateOrderId(): string {
    return `ORD-${Date.now().toString(36).toUpperCase()}`;
  }

  private applyDefaultDiscount(order: Order): void {
    if (order.items.length > 5) {
      order.discount = 0.1; // 10% off
    }
  }

  private async validateCoupon(code: string): Promise<Coupon | null> {
    // Validate coupon logic
    return null;
  }

  private calculateTax(amount: number): number {
    return amount * 0.08; // 8% tax
  }
}
```

┌─────────────────────────────────────────────────────────────────────────────┐
│  HYPEREDGE OUTPUT (TypeScript):                                           │
└─────────────────────────────────────────────────────────────────────────────┘

```json
{
  "id": "file_order_service",
  "label": "order_service module",
  "nodes": [
    "OrderService.createOrder",
    "OrderService.insertCoupon",
    "OrderService.calculateTotal",
    "OrderService.generateOrderId",
    "OrderService.applyDefaultDiscount",
    "OrderService.validateCoupon",
    "OrderService.calculateTax"
  ],
  "relation": "participate_in",
  "confidence": "INFERRED",
  "source_file": "orderService.ts"
}
```

---

### 12.3 RUBY EXAMPLE

┌─────────────────────────────────────────────────────────────────────────────┐
│  RUBY: order_service.rb                                                   │
│  Tree-sitter Ruby parser để extract methods, classes, modules              │
└─────────────────────────────────────────────────────────────────────────────┘

```ruby
# order_service.rb
class Order
  attr_accessor :id, :items, :coupon_code, :discount, :status

  def initialize(items:)
    @id = SecureRandom.hex(4).upcase
    @items = items
    @discount = 0
    @status = :pending
  end
end

class OrderService
  def create_order(customer_id:, items:)
    order = Order.new(items: items)
    apply_default_discount(order)
    save_order(order)
    notify_customer(order)
    order
  end

  def insert_coupon(order_id:, coupon_code:)
    order = get_order(order_id)
    coupon = validate_coupon(coupon_code)

    if coupon
      order.coupon_code = coupon_code
      order.discount = coupon.calculate_discount(order)
      save_order(order)
      true
    else
      false
    end
  end

  def calculate_total(order_id:)
    order = get_order(order_id)
    subtotal = order.items.sum { |item| item[:price] * item[:quantity] }
    tax = calculate_tax(subtotal - order.discount)
    subtotal - order.discount + tax
  end

  private

  def apply_default_discount(order)
    order.discount = 0.1 if order.items.length > 5
  end

  def validate_coupon(code)
    # Coupon validation logic
    nil
  end

  def calculate_tax(amount)
    amount * 0.08
  end

  def save_order(order)
    # Save to database
  end

  def get_order(order_id)
    # Get from database
  end
end
```

┌─────────────────────────────────────────────────────────────────────────────┐
│  HYPEREDGE OUTPUT (Ruby):                                                 │
└─────────────────────────────────────────────────────────────────────────────┘

```json
{
  "id": "file_order_service",
  "label": "order_service module",
  "nodes": [
    "Order.create_order",
    "Order.insert_coupon",
    "Order.calculate_total",
    "Order.apply_default_discount",
    "Order.validate_coupon",
    "Order.calculate_tax",
    "Order.save_order",
    "Order.get_order"
  ],
  "relation": "participate_in",
  "confidence": "INFERRED",
  "source_file": "order_service.rb"
}
```

---

## DOMAIN LOGIC HYPEREDGE EXAMPLES
─────────────────────────────────────────────────────────────────────────────────

┌─────────────────────────────────────────────────────────────────────────────┐
│  HYPEREDGE KHÔNG CHỈ LÀ FILE - MÀ LÀ DOMAIN LOGIC GROUP                  │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  Domain Hyperedge = "Nhóm functions cùng làm 1 USE CASE/BUSINESS FLOW"     │
│                                                                             │
│  Ví dụ:                                                                   │
│  • Create Order Flow: create_order → apply_discount → validate_coupon → save  │
│  • Payment Flow: process_payment → charge_card → send_receipt → update_order  │
│  • Shipping Flow: create_shipment → calculate_shipping → track_package       │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘

### DOMAIN HYPEREDGE 1: Create Order Flow

┌─────────────────────────────────────────────────────────────────────────────┐
│  FILES INVOLVED:                                                          │
│  • order_service.py                                                       │
│  • discount_service.py                                                    │
│  • inventory_service.py                                                   │
│  • notification_service.py                                                │
└─────────────────────────────────────────────────────────────────────────────┘

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        CREATE ORDER DOMAIN FLOW                            │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐     │
│  │ create_order    │────▶│ validate_items  │────▶│ reserve_stock   │     │
│  │                 │     │                 │     │                 │     │
│  │ (order_service)│     │ (inventory)     │     │ (inventory)     │     │
│  └─────────────────┘     └─────────────────┘     └─────────────────┘     │
│           │                                             │                   │
│           │                                             │                   │
│           ▼                                             ▼                   │
│  ┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐     │
│  │ apply_discount  │────▶│ validate_coupon │────▶│ calculate_total │     │
│  │                 │     │                 │     │                 │     │
│  │ (discount)      │     │ (discount)      │     │ (order_service) │     │
│  └─────────────────┘     └─────────────────┘     └─────────────────┘     │
│                                                            │               │
│                                                            │               │
│                                                            ▼               │
│                                                  ┌─────────────────┐        │
│                                                  │ save_order      │        │
│                                                  │                 │        │
│                                                  │ (order_service)│        │
│                                                  └─────────────────┘        │
│                                                            │               │
│                                                            ▼               │
│                                                  ┌─────────────────┐        │
│                                                  │ send_confirmation│       │
│                                                  │                 │        │
│                                                  │ (notification)  │        │
│                                                  └─────────────────┘        │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

┌─────────────────────────────────────────────────────────────────────────────┐
│  HYPEREDGE OUTPUT: Create Order Flow                                       │
└─────────────────────────────────────────────────────────────────────────────┘

```json
{
  "id": "domain_create_order",
  "label": "Create Order Flow",
  "nodes": [
    "order_service.create_order",
    "inventory.validate_items",
    "inventory.reserve_stock",
    "discount.apply_discount",
    "discount.validate_coupon",
    "order_service.calculate_total",
    "order_service.save_order",
    "notification.send_confirmation"
  ],
  "relation": "domain_flow",
  "confidence": "EXTRACTED",
  "confidence_score": 0.92,
  "source_file": "cross-file (8 nodes)",
  "detection_method": "call_chain_pattern",
  "flow_type": "create_order"
}
```

### DOMAIN HYPEREDGE 2: Insert Coupon Flow

┌─────────────────────────────────────────────────────────────────────────────┐
│  FILES INVOLVED:                                                          │
│  • order_service.py                                                       │
│  • discount_service.py                                                    │
│  • validation_service.py                                                  │
└─────────────────────────────────────────────────────────────────────────────┘

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        INSERT COUPON DOMAIN FLOW                            │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐     │
│  │ insert_coupon   │────▶│ validate_coupon │────▶│ check_usage     │     │
│  │                 │     │                 │     │                 │     │
│  │ (order_service) │     │ (discount)      │     │ (discount)      │     │
│  └─────────────────┘     └─────────────────┘     └─────────────────┘     │
│           │                       │                       │                   │
│           │                       │                       │                   │
│           ▼                       ▼                       ▼                   │
│  ┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐     │
│  │ apply_discount  │◀────│ calculate_discount│    │ check_expiry   │     │
│  │                 │     │                 │     │                 │     │
│  │ (discount)      │     │ (discount)      │     │ (discount)      │     │
│  └─────────────────┘     └─────────────────┘     └─────────────────┘     │
│           │                       │                                           │
│           │                       │                                           │
│           ▼                       ▼                                           │
│  ┌─────────────────┐     ┌─────────────────┐                              │
│  │ update_order    │     │ log_coupon_use  │                              │
│  │                 │     │                 │                              │
│  │ (order_service) │     │ (discount)      │                              │
│  └─────────────────┘     └─────────────────┘                              │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

┌─────────────────────────────────────────────────────────────────────────────┐
│  HYPEREDGE OUTPUT: Insert Coupon Flow                                      │
└─────────────────────────────────────────────────────────────────────────────┘

```json
{
  "id": "domain_insert_coupon",
  "label": "Insert Coupon Flow",
  "nodes": [
    "order_service.insert_coupon",
    "discount.validate_coupon",
    "discount.check_usage",
    "discount.check_expiry",
    "discount.calculate_discount",
    "discount.apply_discount",
    "order_service.update_order",
    "discount.log_coupon_use"
  ],
  "relation": "domain_flow",
  "confidence": "EXTRACTED",
  "confidence_score": 0.88,
  "source_file": "cross-file (8 nodes)",
  "detection_method": "call_chain_pattern",
  "flow_type": "insert_coupon"
}
```

### DOMAIN HYPEREDGE 3: Process Payment Flow

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        PROCESS PAYMENT DOMAIN FLOW                         │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐     │
│  │ process_payment │────▶│ authorize_card  │────▶│ charge_card     │     │
│  │                 │     │                 │     │                 │     │
│  │ (payment)       │     │ (payment)       │     │ (payment)       │     │
│  └─────────────────┘     └─────────────────┘     └─────────────────┘     │
│           │                       │                       │                   │
│           │                       ▼                       ▼                   │
│           │              ┌─────────────────┐     ┌─────────────────┐      │
│           │              │ validate_card   │     │ capture_payment │      │
│           │              │                 │     │                 │      │
│           │              │ (payment)       │     │ (payment)       │      │
│           │              └─────────────────┘     └─────────────────┘      │
│           │                                                     │           │
│           ▼                                                     ▼           │
│  ┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐      │
│  │ update_order    │◀────│ send_receipt    │◀────│ record_payment  │      │
│  │ (order_service) │     │ (notification)  │     │ (payment)       │      │
│  └─────────────────┘     └─────────────────┘     └─────────────────┘      │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

```json
{
  "id": "domain_process_payment",
  "label": "Process Payment Flow",
  "nodes": [
    "payment.process_payment",
    "payment.authorize_card",
    "payment.validate_card",
    "payment.charge_card",
    "payment.capture_payment",
    "payment.record_payment",
    "notification.send_receipt",
    "order_service.update_order"
  ],
  "relation": "domain_flow",
  "confidence": "EXTRACTED",
  "confidence_score": 0.95,
  "source_file": "cross-file (8 nodes)",
  "detection_method": "call_chain_pattern",
  "flow_type": "process_payment"
}
```

---

## CROSS-FILE DOMAIN HYPEREDGE ALGORITHM

```rust
/// Detect domain logic hyperedges (cross-file call chains)
pub fn detect_domain_hyperedges(graph: &GraphData) -> Vec<HyperedgeCandidate> {
    let mut candidates = Vec::new();

    // 1. Build call graph across all files
    let cross_file_calls: Vec<_> = graph.links.iter()
        .filter(|e| e.relation == "calls")
        .filter(|e| {
            // Filter: source file != target file
            let src_file = graph.nodes.iter()
                .find(|n| n.id == e.source)
                .map(|n| &n.source_file);
            let tgt_file = graph.nodes.iter()
                .find(|n| n.id == e.target)
                .map(|n| &n.source_file);
            src_file != tgt_file
        })
        .collect();

    // 2. Find chains across files
    // A(file1) → B(file2) → C(file3) → ...

    // 3. Group by domain pattern
    // Pattern: "create_*" → "validate_*" → "save_*" → "notify_*"

    let domain_patterns = vec![
        vec!["create", "validate", "reserve", "apply", "save", "send"],
        vec!["insert", "validate", "check", "apply", "update", "log"],
        vec!["process", "authorize", "validate", "charge", "capture", "record", "send"],
    ];

    // 4. Match functions to patterns

    candidates
}

/// Check if function name matches domain pattern
fn matches_domain_pattern(fn_name: &str, patterns: &[&str]) -> bool {
    let name_lower = fn_name.to_lowercase();
    patterns.iter().any(|p| name_lower.starts_with(p))
}
```

---

## CLI: Multi-Language + Domain Hyperedge

```
┌─────────────────────────────────────────────────────────────────────────────┐
│  CLI COMMANDS:                                                            │
│                                                                             │
│  garfield build ./src --lang=rust                        # Rust only      │
│  garfield build ./src --lang=python,js,rb              # Multi-language  │
│                                                                             │
│  garfield build ./src --hyperedge=domain                # Domain flows    │
│  garfield build ./src --hyperedge=file                  # File groups     │
│  garfield build ./src --hyperedge=chain                 # Call chains     │
│  garfield build ./src --hyperedge=all                   # All methods     │
│                                                                             │
│  garfield query --domain-flow=create_order            # Query by domain │
│  garfield query --domain-flow=insert_coupon           # Query by domain │
│  garfield query --domain-flow=process_payment         # Query by domain │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## SUMMARY: Language + Domain Hyperedge

```
┌─────────────────────────────────────────────────────────────────────────────┐
│  HYPEREDGE TYPES:                                                        │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  1. FILE-BASED (Simple):                                                  │
│     • Nodes trong cùng 1 file → 1 hyperedge                             │
│     • O(n), fast, low accuracy                                           │
│                                                                             │
│  2. CROSS-FILE (Module):                                                  │
│     • Nodes trong cùng directory/module → 1 hyperedge                   │
│     • O(n), medium speed, medium accuracy                               │
│                                                                             │
│  3. CALL CHAIN (Flow):                                                    │
│     • A→B→C→D chains → hyperedge                                       │
│     • O(n²), slow, high accuracy                                        │
│                                                                             │
│  4. DOMAIN LOGIC (Business Flow): ⭐ NEW                                │
│     • Functions cùng business flow (create_order, insert_coupon)        │
│     • Cross-file, pattern-based                                         │
│     • O(n²), high accuracy, business-relevant                           │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
