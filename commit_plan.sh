#!/bin/bash

echo "╔═══════════════════════════════════════════════════════════════════════════════╗"
echo "║                        GIT COMMIT PLAN                                       ║"
echo "╚═══════════════════════════════════════════════════════════════════════════════╝"
echo ""

# Commit 1: Dependencies
echo "────────────────────────────────────────────────────────────────────────────────"
echo "1. chore: upgrade dependencies"
echo "────────────────────────────────────────────────────────────────────────────────"
git add Cargo.toml Cargo.lock
git commit -m "chore: upgrade dependencies"

# Commit 2: Types
echo "────────────────────────────────────────────────────────────────────────────────"
echo "2. feat(types): add hyperedge types and enhanced edge structure"
echo "────────────────────────────────────────────────────────────────────────────────"
git add src/types.rs
git commit -m "feat(types): add hyperedge types and enhanced edge structure"

# Commit 3: Cluster (Louvain)
echo "────────────────────────────────────────────────────────────────────────────────"
echo "3. refactor(cluster): replace label propagation with Louvain algorithm"
echo "────────────────────────────────────────────────────────────────────────────────"
git add src/cluster.rs
git commit -m "refactor(cluster): replace label propagation with Louvain algorithm"

# Commit 4: Build
echo "────────────────────────────────────────────────────────────────────────────────"
echo "4. feat(build): add hyperedge support and improve edge deduplication"
echo "────────────────────────────────────────────────────────────────────────────────"
git add src/build.rs
git commit -m "feat(build): add hyperedge support and improve edge deduplication"

# Commit 5: Detect
echo "────────────────────────────────────────────────────────────────────────────────"
echo "5. refactor(detect): improve file type classification"
echo "────────────────────────────────────────────────────────────────────────────────"
git add src/detect.rs
git commit -m "refactor(detect): improve file type classification"

# Commit 6: Extract
echo "────────────────────────────────────────────────────────────────────────────────"
echo "6. refactor(extract): improve AST extraction logic"
echo "────────────────────────────────────────────────────────────────────────────────"
git add src/extract.rs
git commit -m "refactor(extract): improve AST extraction logic"

# Commit 7: Cache
echo "────────────────────────────────────────────────────────────────────────────────"
echo "7. refactor(cache): improve caching mechanism"
echo "────────────────────────────────────────────────────────────────────────────────"
git add src/cache.rs
git commit -m "refactor(cache): improve caching mechanism"

# Commit 8: Analyze
echo "────────────────────────────────────────────────────────────────────────────────"
echo "8. refactor(analyze): improve analysis algorithms"
echo "────────────────────────────────────────────────────────────────────────────────"
git add src/analyze.rs
git commit -m "refactor(analyze): improve analysis algorithms"

# Commit 9: Serve
echo "────────────────────────────────────────────────────────────────────────────────"
echo "9. refactor(serve): improve API endpoints"
echo "────────────────────────────────────────────────────────────────────────────────"
git add src/serve.rs
git commit -m "refactor(serve): improve API endpoints"

# Commit 10: Report
echo "────────────────────────────────────────────────────────────────────────────────"
echo "10. refactor(report): improve report generation"
echo "────────────────────────────────────────────────────────────────────────────────"
git add src/report.rs
git commit -m "refactor(report): improve report generation"

# Commit 11: Export
echo "────────────────────────────────────────────────────────────────────────────────"
echo "11. refactor(export): improve export formats"
echo "────────────────────────────────────────────────────────────────────────────────"
git add src/export.rs
git commit -m "refactor(export): improve export formats"

# Commit 12: Validate
echo "────────────────────────────────────────────────────────────────────────────────"
echo "12. refactor(validate): improve validation logic"
echo "────────────────────────────────────────────────────────────────────────────────"
git add src/validate.rs
git commit -m "refactor(validate): improve validation logic"

# Commit 13: Lib
echo "────────────────────────────────────────────────────────────────────────────────"
echo "13. refactor(lib): restructure library exports"
echo "────────────────────────────────────────────────────────────────────────────────"
git add src/lib.rs
git commit -m "refactor(lib): restructure library exports"

# Commit 14: Main
echo "────────────────────────────────────────────────────────────────────────────────"
echo "14. refactor(main): improve CLI interface"
echo "────────────────────────────────────────────────────────────────────────────────"
git add src/main.rs
git commit -m "refactor(main): improve CLI interface"

# Commit 15: Tests
echo "────────────────────────────────────────────────────────────────────────────────"
echo "15. test: enhance integration tests"
echo "────────────────────────────────────────────────────────────────────────────────"
git add tests/integration_test.rs
git commit -m "test: enhance integration tests"

# Commit 16: Docs
echo "────────────────────────────────────────────────────────────────────────────────"
echo "16. docs: update SKILL.md documentation"
echo "────────────────────────────────────────────────────────────────────────────────"
git add SKILL.md agents/pi/SKILL.md agents/pi/index.ts
git commit -m "docs: update SKILL.md documentation"

# Commit 17: Reports
echo "────────────────────────────────────────────────────────────────────────────────"
echo "17. chore: update generated reports"
echo "────────────────────────────────────────────────────────────────────────────────"
git add garfield-out/GRAPH_REPORT.md
git commit -m "chore: update generated reports"

echo ""
echo "╔═══════════════════════════════════════════════════════════════════════════════╗"
echo "║                        COMMIT COMPLETED!                                     ║"
echo "╚═══════════════════════════════════════════════════════════════════════════════╝"
echo ""
git log --oneline -17
