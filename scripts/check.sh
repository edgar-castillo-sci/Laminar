#!/usr/bin/env bash
# scripts/check.sh — one-shot build + test + skeleton.
set -uo pipefail

FAIL=0

echo "==> cargo fmt --check"
if ! cargo fmt --check; then
    echo "    [FAIL] formatting"
    FAIL=1
else
    echo "    [OK]"
fi

echo "==> cargo build --workspace"
if ! cargo build --workspace 2>&1 | grep -E "^(error|warning)"; then
    echo "    [OK] no errors/warnings"
else
    echo "    [FAIL] build"
    FAIL=1
fi

echo "==> cargo test --workspace"
if cargo test --workspace 2>&1 | tail -5 | grep -q "test result: ok"; then
    echo "    [OK] all tests pass"
else
    echo "    [FAIL] tests"
    cargo test --workspace 2>&1 | grep -E "FAILED|panicked|^failures:" | head -10
    FAIL=1
fi

echo "==> scripts/skeleton.sh"
if ./scripts/skeleton.sh 2>&1 | tail -3 | grep -q "ASE OK: Mo2S4"; then
    echo "    [OK] skeleton passes"
else
    echo "    [FAIL] skeleton"
    FAIL=1
fi

if [ "$FAIL" -eq 0 ]; then
    echo
    echo "ALL CHECKS PASSED"
    exit 0
else
    echo
    echo "SOME CHECKS FAILED"
    exit 1
fi
