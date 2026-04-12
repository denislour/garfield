# Graph Report - /home/jake/Compa/garfield  (2026-04-26)

## Corpus Check
- ⚠️ Small corpus: 15 files · ~22543 words
  Graph may not add much value for small codebases.

## Summary
- 144 nodes · 129 edges · 15 communities detected
- Extraction: 0% EXTRACTED · 100% INFERRED · 0% AMBIGUOUS · INFERRED: 129 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output (no LLM used)

## God Nodes (most connected - your core abstractions)
1. `global` - 22 edges
   📁 ./src/extract.rs · source: extract:global
2. `global` - 19 edges
   📁 ./src/analyze.rs · source: analyze:global
3. `global` - 18 edges
   📁 ./src/lib.rs · source: lib:global
4. `global` - 11 edges
   📁 ./src/detect.rs · source: detect:global
5. `global` - 11 edges
   📁 ./src/serve.rs · source: serve:global
6. `global` - 9 edges
   📁 ./src/cache.rs · source: cache:global
7. `global` - 9 edges
   📁 ./src/main.rs · source: main:global
8. `global` - 7 edges
   📁 ./src/community.rs · source: community:global
9. `global` - 6 edges
   📁 ./src/summary.rs · source: summary:global
10. `global` - 6 edges
   📁 ./src/build.rs · source: build:global

## Surprising Connections (you probably didn't know these)
- None detected - all connections are within the same source files.

## Communities

### 0 "Src" (23 nodes) 🔴
**Cohesion:** 0.09

**Key concepts:** Some, create_rationale_node, extract_call, extract_docstring_from_body, extract_docstrings, extract_file, extract_go_import, extract_import, extract_name_from_line, extract_python_rationale

### 1 "Src" (20 nodes) 🔴
**Cohesion:** 0.10

**Key concepts:** Some, calculate_betweenness, calculate_cohesion_scores, calculate_surprise_score, count_community_sizes, count_confidence, create_test_graph, find_cross_community_surprises, find_cross_file_surprises, find_god_nodes

### 2 "Src" (19 nodes) 🔴
**Cohesion:** 0.11

**Key concepts:** Some, add_communities, build_graph, cluster, detect, estimate_word_count, extract_files, filter_code_files, from_json, generate_file_summaries

### 4 "Src" (12 nodes) 🔴
**Cohesion:** 0.17

**Key concepts:** Some, classify_extension, count_words, estimate_word_count, get_stats, glob_to_regex, global, is_ignored, is_noise_dir, is_sensitive

### 3 "Src" (12 nodes) 🔴
**Cohesion:** 0.17

**Key concepts:** Some, bfs, build_adjacency, create_test_graph, dfs, find_edge, find_shortest_path, get_node, global, graph_stats

### 5 "Src" (10 nodes) 🟡
**Cohesion:** 0.20

**Key concepts:** generate_extension_ts, generate_garfield_section, generate_mcp_config, global, install_agent, install_claude_agent, install_cursor_agent, install_pi_agent, remove_garfield_section, uninstall_agent

### 6 "Src" (10 nodes) 🟡
**Cohesion:** 0.20

**Key concepts:** Some, check_cache, compute_hash, extract_md_body, get_cache_dir, global, load_cached, save_cached, update_cache

### 7 "Src" (8 nodes) 🟡
**Cohesion:** 0.25

**Key concepts:** Some, add_communities, calculate_cohesion, cluster, create_test_graph, global, leiden_communities, split_oversized

### 9 "Src" (7 nodes) 🟡
**Cohesion:** 0.29

**Key concepts:** Some, extract_function_body, generate_file_level_summary, generate_file_summaries, get_file_summary, global, is_function_like

### 8 "Src" (7 nodes) 🟡
**Cohesion:** 0.29

**Key concepts:** add_communities, build_graph, cluster, dedup_edges, global, merge_extractions, split_oversized

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

- `Src` (13 nodes)
- `Src` (14 nodes)

## 💡 Suggested Questions

Questions the graph is uniquely positioned to answer:

### 1. verify inferred

**Q:** Are the 22 inferred relationships involving `global` (e.g. with `find_language_by_ext` and `simple_extract`) actually correct?

**Why:** `global` has 22 INFERRED edges - model-reasoned connections that need verification.

### 2. verify inferred

**Q:** Are the 19 inferred relationships involving `global` (e.g. with `find_god_nodes` and `find_surprising_connections`) actually correct?

**Why:** `global` has 19 INFERRED edges - model-reasoned connections that need verification.

### 3. verify inferred

**Q:** Are the 18 inferred relationships involving `global` (e.g. with `detect` and `filter_code_files`) actually correct?

**Why:** `global` has 18 INFERRED edges - model-reasoned connections that need verification.

### 4. verify inferred

**Q:** Are the 11 inferred relationships involving `global` (e.g. with `glob_to_regex` and `Some`) actually correct?

**Why:** `global` has 11 INFERRED edges - model-reasoned connections that need verification.

### 5. verify inferred

**Q:** Are the 11 inferred relationships involving `global` (e.g. with `build_adjacency` and `find_edge`) actually correct?

**Why:** `global` has 11 INFERRED edges - model-reasoned connections that need verification.

### 6. isolated nodes

**Q:** What connects `Some`, `calculate_betweenness`, `calculate_cohesion_scores` to the rest of the system?

**Why:** 3 weakly-connected nodes found - possible documentation gaps or missing edges.

### 7. low cohesion

**Q:** Should `Src` be split into smaller, more focused modules?

**Why:** Cohesion score 0.10 - nodes in this community are weakly interconnected.

