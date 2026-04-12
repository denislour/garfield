# Graph Report - /home/jake/Compa/garfield  (2026-04-26)

## Corpus Check
- ⚠️ Small corpus: 31 files · ~47126 words
  Graph may not add much value for small codebases.

## Summary
- 170 nodes · 162 edges · 20 communities detected
- Extraction: 12% EXTRACTED · 87% INFERRED · 0% AMBIGUOUS · INFERRED: 142 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output (no LLM used)

## God Nodes (most connected - your core abstractions)
1. `global` - 22 edges
   📁 ./src/extract.rs · source: extract:global
2. `global` - 19 edges
   📁 ./src/analyze.rs · source: analyze:global
3. `global` - 17 edges
   📁 ./src/lib.rs · source: lib:global
4. `global` - 11 edges
   📁 ./src/detect.rs · source: detect:global
5. `global` - 11 edges
   📁 ./src/serve.rs · source: serve:global
6. `global` - 10 edges
   📁 ./tests/integration_test.rs · source: integration_test:global
7. `global` - 9 edges
   📁 ./src/main.rs · source: main:global
8. `global` - 9 edges
   📁 ./src/cache.rs · source: cache:global
9. `runGarfield` - 8 edges
   📁 ./agents/pi/index.ts · source: index:runGarfield
10. `global` - 7 edges
   📁 ./src/cluster.rs · source: cluster:global

## Surprising Connections (you probably didn't know these)
- None detected - all connections are within the same source files.

## Communities

### 0 "Src" (23 nodes) 🔴
**Cohesion:** 0.09

**Key concepts:** Some, create_rationale_node, extract_call, extract_docstring_from_body, extract_docstrings, extract_file, extract_go_import, extract_import, extract_name_from_line, extract_python_rationale

### 1 "Src" (20 nodes) 🔴
**Cohesion:** 0.10

**Key concepts:** Some, calculate_betweenness, calculate_cohesion_scores, calculate_surprise_score, count_community_sizes, count_confidence, create_test_graph, find_cross_community_surprises, find_cross_file_surprises, find_god_nodes

### 2 "Src" (18 nodes) 🔴
**Cohesion:** 0.11

**Key concepts:** Some, add_communities, build_graph, cluster, detect, estimate_word_count, extract_files, filter_code_files, from_json, generate_report

### 4 "Src" (12 nodes) 🔴
**Cohesion:** 0.17

**Key concepts:** Some, classify_extension, count_words, estimate_word_count, get_stats, glob_to_regex, global, is_ignored, is_noise_dir, is_sensitive

### 5 "Pi" (12 nodes) 🔴
**Cohesion:** 0.14

**Key concepts:** String, buildGraph, execute, existsSync, explainNode, findPath, isGarfieldAvailable, loadGraph, onUpdate, reject

### 3 "Src" (12 nodes) 🔴
**Cohesion:** 0.17

**Key concepts:** Some, bfs, build_adjacency, create_test_graph, dfs, find_edge, find_shortest_path, get_node, global, graph_stats

### 6 "Tests" (11 nodes) 🔴
**Cohesion:** 0.18

**Key concepts:** Some, analyze, build_graph, cluster, detect, extract_file, find_god_nodes, find_shortest_path, global, score_nodes

### 8 "Src" (10 nodes) 🟡
**Cohesion:** 0.20

**Key concepts:** Some, check_cache, compute_hash, extract_md_body, get_cache_dir, global, load_cached, save_cached, update_cache

### 7 "Src" (10 nodes) 🟡
**Cohesion:** 0.20

**Key concepts:** generate_extension_ts, generate_garfield_section, generate_mcp_config, global, install_agent, install_claude_agent, install_cursor_agent, install_pi_agent, remove_garfield_section, uninstall_agent

### 9 "Src" (8 nodes) 🟡
**Cohesion:** 0.25

**Key concepts:** Some, add_communities, calculate_cohesion, cluster, create_test_graph, global, louvain_communities, split_oversized

## Knowledge Gaps

### 🔌 Isolated Nodes

These have ≤1 connection - possible documentation gaps:

- `Some`
- `calculate_betweenness`
- `calculate_cohesion_scores`
- `calculate_surprise_score`
- `count_community_sizes`

### 📉 Thin Communities

Too small to be meaningful - may be noise:

- `Src` (17 nodes)
- `Src` (16 nodes)
- `Src` (15 nodes)
- `Pi` (19 nodes)
- `Pi` (18 nodes)

## 💡 Suggested Questions

Questions the graph is uniquely positioned to answer:

### 1. bridge node

**Q:** Why does `reject` connect `Pi` to `Pi`?

**Why:** High betweenness centrality (0.00641025641025641) - this node is a cross-community bridge.

### 2. verify inferred

**Q:** Are the 22 inferred relationships involving `global` (e.g. with `find_language_by_ext` and `simple_extract`) actually correct?

**Why:** `global` has 22 INFERRED edges - model-reasoned connections that need verification.

### 3. verify inferred

**Q:** Are the 19 inferred relationships involving `global` (e.g. with `find_god_nodes` and `find_surprising_connections`) actually correct?

**Why:** `global` has 19 INFERRED edges - model-reasoned connections that need verification.

### 4. verify inferred

**Q:** Are the 17 inferred relationships involving `global` (e.g. with `detect` and `filter_code_files`) actually correct?

**Why:** `global` has 17 INFERRED edges - model-reasoned connections that need verification.

### 5. verify inferred

**Q:** Are the 11 inferred relationships involving `global` (e.g. with `build_adjacency` and `find_edge`) actually correct?

**Why:** `global` has 11 INFERRED edges - model-reasoned connections that need verification.

### 6. verify inferred

**Q:** Are the 11 inferred relationships involving `global` (e.g. with `glob_to_regex` and `Some`) actually correct?

**Why:** `global` has 11 INFERRED edges - model-reasoned connections that need verification.

### 7. isolated nodes

**Q:** What connects `Some`, `calculate_betweenness`, `calculate_cohesion_scores` to the rest of the system?

**Why:** 3 weakly-connected nodes found - possible documentation gaps or missing edges.

