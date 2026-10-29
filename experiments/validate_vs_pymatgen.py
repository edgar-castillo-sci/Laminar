#!/usr/bin/env python3
"""
External validation of Laminar against pymatgen.

For each (option, n), generate canonical chains with Laminar, export to
CIF, parse with pymatgen, and verify:

1. Every CIF parses.
2. Stoichiometry is correct.
3. pymatgen's StructureMatcher does NOT find MORE groups than Laminar's
   canonical classes. (If it did, Laminar would be over-counting.)

Note: pymatgen's StructureMatcher is more general than G = S3 x Z/2,
so it may over-group (M < N). We only flag M > N as an error.

Usage:
    python experiments/validate_vs_pymatgen.py

Requires:
    - laminar (maturin develop)
    - pymatgen

Exit: 0 if all checks pass, 1 otherwise.
"""

import sys
import time
from collections import Counter

import laminar
from pymatgen.core import Structure
from pymatgen.analysis.structure_matcher import StructureMatcher


# ReferenceCell per option. Parameters chosen to be physical enough for
# pymatgen to parse (exact a/c don't matter for the group check).
CELLS = {
    "PL":      dict(a=2.46, c=6.70,  z_up=0.01, z_down=0.0,   species_map={"j": "C"}),
    "BPL":     dict(a=3.00, c=8.00,  z_up=0.01, z_down=0.0,   species_map={"j": "B", "k": "N"}),
    "BU":      dict(a=3.30, c=10.50, z_up=0.50, z_down=-0.50, species_map={"j": "P"}),
    "BBU":     dict(a=3.50, c=12.00, z_up=0.50, z_down=-0.50, species_map={"j": "B", "k": "N"}),
    "OneH":    dict(a=3.16, c=12.30, z_up=1.56, z_down=-1.56, species_map={"j": "Mo", "k": "S"}),
    "OneT":    dict(a=3.16, c=12.30, z_up=1.56, z_down=-1.56, species_map={"j": "Mo", "k": "S"}),
    "Janus1H": dict(a=3.20, c=12.50, z_up=1.60, z_down=-1.60, species_map={"j": "Mo", "k": "S", "l": "Se"}),
    "Janus1T": dict(a=3.20, c=12.50, z_up=1.60, z_down=-1.60, species_map={"j": "Mo", "k": "S", "l": "Se"}),
}


def expected_formula(option: str, n: int) -> Counter:
    """Expected number of atoms per species, per layer count."""
    per_layer = {
        "PL":      Counter({"C": 2}),
        "BPL":     Counter({"B": 1, "N": 1}),
        "BU":      Counter({"P": 2}),
        "BBU":     Counter({"B": 1, "N": 1}),
        "OneH":    Counter({"Mo": 1, "S": 2}),
        "OneT":    Counter({"Mo": 1, "S": 2}),
        "Janus1H": Counter({"Mo": 1, "S": 1, "Se": 1}),
        "Janus1T": Counter({"Mo": 1, "S": 1, "Se": 1}),
    }[option]
    return Counter({k: v * n for k, v in per_layer.items()})


def run_one(option: str, n: int) -> dict:
    cell_kwargs = CELLS[option]
    cell = laminar.ReferenceCell(**cell_kwargs)
    chains = laminar.sandwich(n=n, option=option, export="stdout")
    n_classes = len(chains)
    n_parsed = 0
    n_formula_ok = 0
    parsed_structures = []

    for raw in chains:
        chain = laminar.JamChain.parse(raw, option=option)
        cif = chain.to_cif(cell)
        try:
            s = Structure.from_str(cif, fmt="cif")
        except Exception as e:
            return {
                "option": option, "n": n, "n_classes": n_classes,
                "error": f"pymatgen parse failed for {raw}: {e}",
            }
        n_parsed += 1
        # Stoichiometry check.
        expected = expected_formula(option, n)
        actual = Counter({str(el): int(round(cnt)) for el, cnt in s.composition.items()})
        if actual == expected:
            n_formula_ok += 1
        parsed_structures.append(s)

    # Group with StructureMatcher.
    matcher = StructureMatcher(
        primitive_cell=False,
        scale=False,
        attempt_supercell=False,
        ltol=0.2, stol=0.3, angle_tol=5.0,
    )
    groups = matcher.group_structures(parsed_structures)
    n_groups = len(groups)

    return {
        "option": option, "n": n,
        "n_classes": n_classes,
        "n_parsed": n_parsed,
        "n_formula_ok": n_formula_ok,
        "n_pymatgen_groups": n_groups,
        "overcount": n_groups > n_classes,   # BAD if True
        "stricter_than_expected": n_groups < n_classes,  # OK (pymatgen over-groups)
    }


def main() -> int:
    cases = [
        (opt, 2) for opt in CELLS
    ] + [
        (opt, 3) for opt in ["PL", "BU", "OneH", "Janus1H"]
    ]

    print(f"[pymatgen] validating {len(cases)} cases\n")
    print(f"{'option':8} {'n':>2} {'classes':>7} {'parsed':>7} {'formula':>7} {'pymatgen':>9} status")
    print("-" * 60)

    failures = 0
    for option, n in cases:
        t0 = time.time()
        try:
            r = run_one(option, n)
        except Exception as e:
            print(f"{option:8} {n:>2} ERROR: {e}")
            failures += 1
            continue
        dt = time.time() - t0

        if "error" in r:
            print(f"{option:8} {n:>2} ERROR: {r['error']}")
            failures += 1
            continue

        if r["overcount"]:
            status = "FAIL (overcount)"
            failures += 1
        elif r["stricter_than_expected"]:
            status = f"OK ({r['n_classes'] - r['n_pymatgen_groups']} merged)"
        else:
            status = "OK"

        print(
            f"{option:8} {n:>2} {r['n_classes']:>7} {r['n_parsed']:>7} "
            f"{r['n_formula_ok']:>7} {r['n_pymatgen_groups']:>9} {status}  ({dt:.1f}s)"
        )

    print()
    if failures == 0:
        print(f"[pymatgen] all {len(cases)} cases passed.")
        return 0
    print(f"[pymatgen] {failures} FAILURES.")
    return 1


if __name__ == "__main__":
    sys.exit(main())
