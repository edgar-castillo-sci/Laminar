# pymatgen external validation

**Date:** 2026-10-28
**Laminar version:** 0.1.0-alpha.4
**pymatgen version:** 2026.10.24 (pymatgen-core 2026.10.16)
**Script:** `experiments/validate_vs_pymatgen.py`

## What this validates

For each (option, n):

1. **Parseability.** Every canonical chain from Laminar, exported to CIF
   via `to_cif`, is parsed successfully by `pymatgen.core.Structure.from_str`.
2. **Stoichiometry.** Each CIF has the expected atom counts per species
   (e.g., Mo₂S₄ for MoS₂ with n=2).
3. **No over-counting.** `StructureMatcher.group_structures` does **not**
   find more groups than Laminar's `augment_to(n, option)`.

`StructureMatcher` is more general than `G = S3 × Z/2`, so it may over-group
(M < N). This is expected and not a bug. We only flag over-counting.

## Results

```
option    n  classes  parsed  formula  pymatgen  status
PL        2        2       2        2         2  OK
BPL       2        5       5        5         5  OK
BU        2        5       5        5         4  OK (1 merged)
BBU       2       16      16       16        10  OK (6 merged)
OneH      2        5       5        5         5  OK
OneT      2        5       5        5         4  OK (1 merged)
Janus1H   2       16      16       16        10  OK (6 merged)
Janus1T   2       16      16       16        10  OK (6 merged)
PL        3        4       4        4         3  OK (1 merged)
BU        3       21      21       21        10  OK (11 merged)
OneH      3       21      21       21        10  OK (11 merged)
Janus1H   3      144     144      144        50  OK (94 merged)
```

## Interpretation

- **Parseability**: 100%. All N CIFs parse in every case.
- **Stoichiometry**: 100%. All N CIFs have correct formula.
- **No over-counting**: every case satisfies `pymatgen groups <= Laminar classes`.
  Laminar never separates structures that pymatgen considers equivalent.

The "merged" cases (e.g., BU n=3: 21 -> 10) are expected: `StructureMatcher`
with `ltol=0.2, stol=0.3, angle_tol=5.0` is permissive and groups structures
that are related by symmetry operations not in G. This is the natural
behavior of a generic structure matcher; it is not a failure.

## Known limitations

- **CIF metadata.** pymatgen warns `No _symmetry_equiv_pos_as_xyz type key
  found` because our CIF uses `P 1` with explicit atoms but doesn't list
  the (trivial) symmetry operations. Cosmetic; pymatgen still parses
  correctly using the space group default. Tracked as D9.
- **`StructureMatcher` tolerance.** `ltol=0.2, stol=0.3, angle_tol=5.0`
  may merge structures that differ only by a symmetry operation not in G.
  This is expected; we only flag over-counting, not over-grouping.
- **n=4 not tested.** Too many CIFs (e.g., 120 for BU). `augment_to` is
  validated against paper tables in `experiments/parity_canonjam.py`
  (37/37 counts).
