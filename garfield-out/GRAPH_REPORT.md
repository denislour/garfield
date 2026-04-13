# Graph Report - /home/jake/Compa/garfield  (2026-04-27)

## Corpus Check
- ⚠️ Small corpus: 16 files · ~24046 words
  Graph may not add much value for small codebases.

## Summary
- 498 nodes · 137 edges · 361 communities detected
- Extraction: 70% EXTRACTED · 29% INFERRED · 0% AMBIGUOUS · INFERRED: 41 edges (avg confidence: 0.85)
- Token cost: 0 input · 0 output (no LLM used)

## God Nodes (most connected - your core abstractions)
1. `global` - 21 edges
   📁 ./src/extract.rs · source: extract:global
2. `global` - 19 edges
   📁 ./src/analyze.rs · source: analyze:global
3. `global` - 17 edges
   📁 ./src/lib.rs · source: lib:global
4. `global` - 14 edges
   📁 ./src/serve.rs · source: serve:global
5. `global` - 11 edges
   📁 ./src/detect.rs · source: detect:global
6. `global` - 10 edges
   📁 ./src/hyperedge.rs · source: hyperedge:global
7. `global` - 9 edges
   📁 ./src/cache.rs · source: cache:global
8. `global` - 9 edges
   📁 ./src/main.rs · source: main:global
9. `global` - 7 edges
   📁 ./src/community.rs · source: community:global
10. `global` - 6 edges
   📁 ./src/build.rs · source: build:global

## Surprising Connections (you probably didn't know these)
- None detected - all connections are within the same source files.

## Communities

### 0 "Src" (22 nodes) 🔴
**Cohesion:** 0.09

**Key concepts:** Some, create_rationale_node, extract_call, extract_docstring_from_body, extract_docstrings, extract_file, extract_go_import, extract_import, extract_name_from_line, extract_python_rationale

### 1 "Src" (20 nodes) 🔴
**Cohesion:** 0.10

**Key concepts:** Some, calculate_betweenness, calculate_cohesion_scores, calculate_surprise_score, count_community_sizes, count_confidence, create_test_graph, find_cross_community_surprises, find_cross_file_surprises, find_god_nodes

### 2 "Src" (18 nodes) 🔴
**Cohesion:** 0.11

**Key concepts:** Some, add_communities, build_graph, cluster, detect, detect_hyperedges, estimate_word_count, extract_files, filter_code_files, from_json

### 3 "Src" (15 nodes) 🔴
**Cohesion:** 0.13

**Key concepts:** Some, bfs, build_adjacency, create_test_graph, dfs, extract_function_body, find_edge, find_shortest_path, find_source_file, get_node

### 4 "Src" (12 nodes) 🔴
**Cohesion:** 0.17

**Key concepts:** Some, classify_extension, count_words, estimate_word_count, get_stats, glob_to_regex, global, is_ignored, is_noise_dir, is_sensitive

### 5 "Src" (11 nodes) 🔴
**Cohesion:** 0.18

**Key concepts:** calculate_chain_cohesion, calculate_cohesion, create_test_graph, detect_call_chains, detect_config_patterns, detect_file_groups, detect_hyperedges, find_chains_dfs, get_first_file, global

### 6 "Src" (10 nodes) 🟡
**Cohesion:** 0.20

**Key concepts:** generate_extension_ts, generate_garfield_section, generate_mcp_config, global, install_agent, install_claude_agent, install_cursor_agent, install_pi_agent, remove_garfield_section, uninstall_agent

### 7 "Src" (10 nodes) 🟡
**Cohesion:** 0.20

**Key concepts:** Some, check_cache, compute_hash, extract_md_body, get_cache_dir, global, load_cached, save_cached, update_cache

### 8 "Src" (8 nodes) 🟡
**Cohesion:** 0.25

**Key concepts:** Some, add_communities, calculate_cohesion, cluster, create_test_graph, global, leiden_communities, split_oversized

### 9 "Src" (7 nodes) 🟡
**Cohesion:** 0.29

**Key concepts:** add_communities, build_graph, cluster, dedup_edges, global, merge_extractions, split_oversized

## Knowledge Gaps

### 🔌 Isolated Nodes

These have ≤1 connection - possible documentation gaps:

- `Analysis`
- `ConfidenceStats`
- `EdgeChange`
- `GodNode`
- `GraphDiff`

### 📉 Thin Communities

Too small to be meaningful - may be noise:

- `Src` (324 nodes)
- `Src` (233 nodes)
- `Src` (253 nodes)
- `Src` (341 nodes)
- `Src` (76 nodes)
- `Src` (180 nodes)
- `Src` (327 nodes)
- `Src` (65 nodes)
- `Src` (216 nodes)
- `Src` (139 nodes)
- `Src` (19 nodes)
- `Src` (193 nodes)
- `Src` (200 nodes)
- `Src` (148 nodes)
- `Src` (350 nodes)
- `Src` (235 nodes)
- `Src` (245 nodes)
- `Src` (93 nodes)
- `Src` (114 nodes)
- `Src` (63 nodes)
- `Src` (82 nodes)
- `Src` (343 nodes)
- `Src` (125 nodes)
- `Src` (201 nodes)
- `Src` (177 nodes)
- `Src` (255 nodes)
- `Src` (134 nodes)
- `Src` (345 nodes)
- `Src` (234 nodes)
- `Src` (267 nodes)
- `Src` (113 nodes)
- `Src` (116 nodes)
- `Src` (359 nodes)
- `Src` (115 nodes)
- `Src` (223 nodes)
- `Src` (157 nodes)
- `Src` (144 nodes)
- `Src` (260 nodes)
- `Src` (126 nodes)
- `Src` (184 nodes)
- `Src` (69 nodes)
- `Src` (29 nodes)
- `Src` (215 nodes)
- `Src` (150 nodes)
- `Src` (299 nodes)
- `Src` (26 nodes)
- `Src` (349 nodes)
- `Src` (21 nodes)
- `Src` (70 nodes)
- `Src` (337 nodes)
- `Src` (161 nodes)
- `Src` (242 nodes)
- `Src` (140 nodes)
- `Src` (25 nodes)
- `Src` (264 nodes)
- `Src` (261 nodes)
- `Src` (83 nodes)
- `Src` (293 nodes)
- `Src` (297 nodes)
- `Src` (222 nodes)
- `Src` (46 nodes)
- `Src` (135 nodes)
- `Src` (71 nodes)
- `Src` (156 nodes)
- `Src` (314 nodes)
- `Src` (45 nodes)
- `Src` (287 nodes)
- `Src` (192 nodes)
- `Src` (99 nodes)
- `Src` (84 nodes)
- `Src` (194 nodes)
- `Src` (146 nodes)
- `Src` (24 nodes)
- `Src` (49 nodes)
- `Src` (14 nodes)
- `Src` (36 nodes)
- `Src` (87 nodes)
- `Src` (95 nodes)
- `Src` (285 nodes)
- `Src` (80 nodes)
- `Src` (311 nodes)
- `Src` (219 nodes)
- `Src` (106 nodes)
- `Src` (304 nodes)
- `Src` (220 nodes)
- `Src` (334 nodes)
- `Src` (282 nodes)
- `Src` (278 nodes)
- `Src` (212 nodes)
- `Src` (92 nodes)
- `Src` (97 nodes)
- `Src` (153 nodes)
- `Src` (149 nodes)
- `Src` (312 nodes)
- `Src` (196 nodes)
- `Src` (94 nodes)
- `Src` (250 nodes)
- `Src` (217 nodes)
- `Src` (54 nodes)
- `Src` (204 nodes)
- `Src` (20 nodes)
- `Src` (178 nodes)
- `Src` (132 nodes)
- `Src` (118 nodes)
- `Src` (143 nodes)
- `Src` (41 nodes)
- `Src` (72 nodes)
- `Src` (152 nodes)
- `Src` (195 nodes)
- `Src` (32 nodes)
- `Src` (103 nodes)
- `Src` (355 nodes)
- `Src` (188 nodes)
- `Src` (100 nodes)
- `Src` (306 nodes)
- `Src` (159 nodes)
- `Src` (214 nodes)
- `Src` (301 nodes)
- `Src` (101 nodes)
- `Src` (230 nodes)
- `Src` (290 nodes)
- `Src` (30 nodes)
- `Src` (57 nodes)
- `Src` (40 nodes)
- `Src` (117 nodes)
- `Src` (358 nodes)
- `Src` (38 nodes)
- `Src` (309 nodes)
- `Src` (239 nodes)
- `Src` (331 nodes)
- `Src` (254 nodes)
- `Src` (185 nodes)
- `Src` (136 nodes)
- `Src` (47 nodes)
- `Src` (286 nodes)
- `Src` (213 nodes)
- `Src` (170 nodes)
- `Src` (205 nodes)
- `Src` (210 nodes)
- `Src` (321 nodes)
- `Src` (259 nodes)
- `Src` (273 nodes)
- `Src` (342 nodes)
- `Src` (128 nodes)
- `Src` (165 nodes)
- `Src` (244 nodes)
- `Src` (353 nodes)
- `Src` (16 nodes)
- `Src` (279 nodes)
- `Src` (186 nodes)
- `Src` (303 nodes)
- `Src` (122 nodes)
- `Src` (130 nodes)
- `Src` (206 nodes)
- `Src` (283 nodes)
- `Src` (270 nodes)
- `Src` (198 nodes)
- `Src` (88 nodes)
- `Src` (108 nodes)
- `Src` (252 nodes)
- `Src` (68 nodes)
- `Src` (35 nodes)
- `Src` (123 nodes)
- `Src` (60 nodes)
- `Src` (218 nodes)
- `Src` (329 nodes)
- `Src` (336 nodes)
- `Src` (73 nodes)
- `Src` (61 nodes)
- `Src` (280 nodes)
- `Src` (52 nodes)
- `Src` (288 nodes)
- `Src` (58 nodes)
- `Src` (340 nodes)
- `Src` (59 nodes)
- `Src` (129 nodes)
- `Src` (55 nodes)
- `Src` (171 nodes)
- `Src` (262 nodes)
- `Src` (104 nodes)
- `Src` (42 nodes)
- `Src` (263 nodes)
- `Src` (351 nodes)
- `Src` (346 nodes)
- `Src` (258 nodes)
- `Src` (56 nodes)
- `Src` (43 nodes)
- `Src` (296 nodes)
- `Src` (112 nodes)
- `Src` (203 nodes)
- `Src` (110 nodes)
- `Src` (289 nodes)
- `Src` (18 nodes)
- `Src` (120 nodes)
- `Src` (107 nodes)
- `Src` (74 nodes)
- `Src` (162 nodes)
- `Src` (77 nodes)
- `Src` (190 nodes)
- `Src` (131 nodes)
- `Src` (224 nodes)
- `Src` (243 nodes)
- `Src` (352 nodes)
- `Src` (151 nodes)
- `Src` (249 nodes)
- `Src` (27 nodes)
- `Src` (53 nodes)
- `Src` (202 nodes)
- `Src` (276 nodes)
- `Src` (51 nodes)
- `Src` (305 nodes)
- `Src` (137 nodes)
- `Src` (228 nodes)
- `Src` (315 nodes)
- `Src` (310 nodes)
- `Src` (248 nodes)
- `Src` (17 nodes)
- `Src` (328 nodes)
- `Src` (15 nodes)
- `Src` (197 nodes)
- `Src` (138 nodes)
- `Src` (174 nodes)
- `Src` (37 nodes)
- `Src` (265 nodes)
- `Src` (241 nodes)
- `Src` (111 nodes)
- `Src` (232 nodes)
- `Src` (172 nodes)
- `Src` (182 nodes)
- `Src` (284 nodes)
- `Src` (271 nodes)
- `Src` (90 nodes)
- `Src` (121 nodes)
- `Src` (347 nodes)
- `Src` (325 nodes)
- `Src` (79 nodes)
- `Src` (226 nodes)
- `Src` (191 nodes)
- `Src` (96 nodes)
- `Src` (207 nodes)
- `Src` (300 nodes)
- `Src` (50 nodes)
- `Src` (102 nodes)
- `Src` (166 nodes)
- `Src` (175 nodes)
- `Src` (163 nodes)
- `Src` (44 nodes)
- `Src` (62 nodes)
- `Src` (34 nodes)
- `Src` (344 nodes)
- `Src` (85 nodes)
- `Src` (31 nodes)
- `Src` (28 nodes)
- `Src` (320 nodes)
- `Src` (292 nodes)
- `Src` (187 nodes)
- `Src` (133 nodes)
- `Src` (160 nodes)
- `Src` (333 nodes)
- `Src` (339 nodes)
- `Src` (236 nodes)
- `Src` (227 nodes)
- `Src` (168 nodes)
- `Src` (119 nodes)
- `Src` (169 nodes)
- `Src` (86 nodes)
- `Src` (291 nodes)
- `Src` (208 nodes)
- `Src` (294 nodes)
- `Src` (164 nodes)
- `Src` (142 nodes)
- `Src` (354 nodes)
- `Src` (266 nodes)
- `Src` (229 nodes)
- `Src` (173 nodes)
- `Src` (356 nodes)
- `Src` (275 nodes)
- `Src` (251 nodes)
- `Src` (199 nodes)
- `Src` (78 nodes)
- `Src` (211 nodes)
- `Src` (323 nodes)
- `Src` (295 nodes)
- `Src` (39 nodes)
- `Src` (158 nodes)
- `Src` (238 nodes)
- `Src` (348 nodes)
- `Src` (91 nodes)
- `Src` (360 nodes)
- `Src` (105 nodes)
- `Src` (23 nodes)
- `Src` (269 nodes)
- `Src` (231 nodes)
- `Src` (335 nodes)
- `Src` (154 nodes)
- `Src` (246 nodes)
- `Src` (332 nodes)
- `Src` (317 nodes)
- `Src` (319 nodes)
- `Src` (240 nodes)
- `Src` (302 nodes)
- `Src` (181 nodes)
- `Src` (209 nodes)
- `Src` (189 nodes)
- `Src` (109 nodes)
- `Src` (221 nodes)
- `Src` (75 nodes)
- `Src` (147 nodes)
- `Src` (127 nodes)
- `Src` (167 nodes)
- `Src` (48 nodes)
- `Src` (274 nodes)
- `Src` (67 nodes)
- `Src` (330 nodes)
- `Src` (272 nodes)
- `Src` (141 nodes)
- `Src` (66 nodes)
- `Src` (281 nodes)
- `Src` (307 nodes)
- `Src` (308 nodes)
- `Src` (179 nodes)
- `Src` (316 nodes)
- `Src` (357 nodes)
- `Src` (298 nodes)
- `Src` (338 nodes)
- `Src` (326 nodes)
- `Src` (81 nodes)
- `Src` (124 nodes)
- `Src` (183 nodes)
- `Src` (257 nodes)
- `Src` (155 nodes)
- `Src` (22 nodes)
- `Src` (322 nodes)
- `Src` (89 nodes)
- `Src` (33 nodes)
- `Src` (64 nodes)
- `Src` (277 nodes)
- `Src` (225 nodes)
- `Src` (145 nodes)
- `Src` (318 nodes)
- `Src` (98 nodes)
- `Src` (247 nodes)
- `Src` (176 nodes)
- `Src` (256 nodes)
- `Src` (237 nodes)
- `Src` (313 nodes)
- `Src` (268 nodes)

## 💡 Suggested Questions

Questions the graph is uniquely positioned to answer:

### 1. verify inferred

**Q:** Are the 3 inferred relationships involving `global` (e.g. with `get_ts_language` and `Ok`) actually correct?

**Why:** `global` has 3 INFERRED edges - model-reasoned connections that need verification.

### 2. verify inferred

**Q:** Are the 2 inferred relationships involving `global` (e.g. with `order` and `Some`) actually correct?

**Why:** `global` has 2 INFERRED edges - model-reasoned connections that need verification.

### 3. verify inferred

**Q:** Are the 16 inferred relationships involving `global` (e.g. with `detect` and `filter_code_files`) actually correct?

**Why:** `global` has 16 INFERRED edges - model-reasoned connections that need verification.

### 4. verify inferred

**Q:** Are the 2 inferred relationships involving `global` (e.g. with `Some` and `Ok`) actually correct?

**Why:** `global` has 2 INFERRED edges - model-reasoned connections that need verification.

### 5. isolated nodes

**Q:** What connects `Analysis`, `ConfidenceStats`, `EdgeChange` to the rest of the system?

**Why:** 3 weakly-connected nodes found - possible documentation gaps or missing edges.

### 6. low cohesion

**Q:** Should `Src` be split into smaller, more focused modules?

**Why:** Cohesion score 0.09 - nodes in this community are weakly interconnected.

### 7. low cohesion

**Q:** Should `Src` be split into smaller, more focused modules?

**Why:** Cohesion score 0.11 - nodes in this community are weakly interconnected.

