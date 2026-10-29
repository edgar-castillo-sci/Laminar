#!/usr/bin/env bash
# scripts/bench.sh — run all Criterion benchmarks.
set -euo pipefail

echo "[bench] running cargo bench -p laminar-core..."
cargo bench -p laminar-core 2>&1 | tee /tmp/laminar-bench.log | tail -80

echo ""
echo "Full log: /tmp/laminar-bench.log"
echo "HTML report: target/criterion/report/index.html"
