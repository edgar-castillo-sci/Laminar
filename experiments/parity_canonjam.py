#!/usr/bin/env python3
"""
Parity check: Laminar's canonical class counts vs. the published
JAM tables (Paper 1, Table 3; Paper 3, Table 4).

If canonJAM is installed (`pip install canonJAM`), the script also
compares against it directly. If not, it falls back to the hardcoded
tables from the papers.

Usage:
    python3 experiments/parity_canonjam.py

Exit code: 0 if all counts match, 1 otherwise.
"""

import subprocess
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
BIN = REPO_ROOT / "target" / "release" / "examples" / "count"

# Paper 1, Table 3 (PL) and Paper 3, Table 4 (BU / OneH).
#
# NOTE on PL n=6: the paper lists 66, but the explicit Burnside
# computation gives 70:
#   (3^6 + 3^3 + 3 + 3*3^3) / 12 = (729 + 27 + 3 + 81)/12 = 70.
# Laminar gives 70, and this is corroborated by exhaustive
# kappa_literal == kappa_fast on all 729 PL chains at k=6.
# We treat 70 as correct; see docs/discrepancies.md for details.
EXPECTED = {
    "PL":   {2: 2, 3: 4, 4: 10, 5: 25, 6: 70},
    "BU":   {2: 5, 3: 21, 4: 120, 5: 666, 6: 3960},
    "OneH": {2: 5, 3: 21, 4: 120, 5: 666, 6: 3960},
}


def run_laminar(option: str, n: int) -> int:
    """Run the Laminar example binary and parse the integer output."""
    if not BIN.exists():
        sys.exit(
            f"[parity] {BIN} not found. Build it first:\n"
            f"    cargo build --release --examples -p laminar-core"
        )
    out = subprocess.check_output([str(BIN), option, str(n)], text=True).strip()
    return int(out)


def run_canonjam(option: str, n: int) -> int | None:
    """Try to run canonJAM. Return None if not installed."""
    try:
        import canonJAM  # noqa: F401
    except ImportError:
        return None
    # canonJAM API: canonJAM.count(n, option="PL") or similar.
    # Adjust once the version is pinned.
    try:
        return int(canonJAM.count(n, option=option))
    except Exception as e:
        print(f"[parity] canonJAM call failed for {option} n={n}: {e}")
        return None


def main() -> int:
    have_canonjam = False
    try:
        import canonJAM  # noqa: F401
        have_canonjam = True
    except ImportError:
        print("[parity] canonJAM not installed; comparing to paper tables only.")
        print("[parity]   pip install canonJAM    # for direct comparison")

    failures = 0
    print(f"{'option':6} {'n':>2}  {'laminar':>8}  {'paper':>6}  {'canonJAM':>8}  status")
    print("-" * 55)
    for option, table in EXPECTED.items():
        for n, expected in table.items():
            got = run_laminar(option, n)
            cj = run_canonjam(option, n) if have_canonjam else None
            ok = got == expected and (cj is None or cj == expected)
            status = "OK" if ok else "FAIL"
            cj_str = "-" if cj is None else str(cj)
            print(f"{option:6} {n:>2}  {got:>8}  {expected:>6}  {cj_str:>8}  {status}")
            if not ok:
                failures += 1

    if failures == 0:
        print(f"\n[parity] all {sum(len(t) for t in EXPECTED.values())} counts match.")
        return 0
    print(f"\n[parity] {failures} FAILURES.")
    return 1


if __name__ == "__main__":
    sys.exit(main())
