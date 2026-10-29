# Changelog

All notable changes to Laminar are documented here.

The project is in **alpha** (`0.1.0-alpha.N`). The strict semver policy
of `spec/v0.1.0-alpha.1.md` §0 applies from `0.2.0-beta.1` onwards.
During alpha, public-type changes bump the alpha tag (e.g.,
`0.1.0-alpha.1` → `0.1.0-alpha.2`) and are recorded here with a
`**BREAKING**` marker.

---

## v0.1.0-alpha.4 — in development (Fase 1B)

Fase 1B adds parser, tests, and CIF coverage for the five options not
covered in Fase 1A: BPL (2), BBU (4), OneT (6), Janus1H (7), Janus1T (8).

### Fase 1B.4 — CIF Janus + 1T tests

**Added**
- 3 tests in `cif.rs`:
  - `cif_one_t_1t_mos2`: 1T MoS₂ (`[*j, +k, -k]`), verifies Mo at
    layer midplane and S at 4 distinct z levels (2 per layer × up/down).
  - `cif_janus1h_mossse`: Janus MoSSe 1H (`[+0, *j, (+k-l)]`),
    verifies S above Mo and Se below Mo.
  - `cif_janus1t_mossse`: Janus MoSSe 1T (`[*j, +k, -l]`), same
    z-checks as 1H.
- 4 tests in `reference_cell.rs` for Janus V2a behavior:
  - symmetric z (|z_up| == |z_down|) passes,
  - asymmetric z (e.g., S at 1.7, Se at -1.5) passes,
  - non-negative z_down fails,
  - z_up == z_down fails.
- Helper `mossse_janus_cell()` (species_map: j=Mo, k=S, l=Se).
- Helper `janus_cell(z_up, z_down)`.

### Fase 1B.3 — OneH uses *j + word() uses distinguished token

**Fixed**
- **OneH (option 5): metal is `*j` (fixed-site), not `+j`.** Per
  Tabla 7 of *Orbits as Multisets* (`[+0, *j, (+k-k)]`). The previous
  `+j` gave correct counts (the orbit is isomorphic) but incorrect
  chain representation and incorrect z-coordinate in CIF (metal
  displaced by `z_up` instead of in-plane).
- **`word()` uses the *distinguished token* per *Stacking as Words*
  p.4:** `*j` (Fixed) for options 5-8, `+0` (Empty) for options 1-4.
  Previously used `+0` for all options, which was wrong for 5-8.

**Changed**
- `validate_row(OneH)`: `1×Empty + 1×Fixed + 1×Buckled` (was `1×Up`).
- `all_rows(OneH)`: permute `[+0, *j, (+k-k)]` (was `[+0, +j, (+k-k)]`).
- `word()`: now option-aware (`Fixed` for 5-8, `Empty` for 1-4).

**Added**
- 3 new tests in `cells.rs`:
  - `word_uses_fixed_for_options_5_through_8`
  - `word_uses_empty_for_options_1_through_4`
  - updated `word_of_mos2_bilayer` (now `[0, 0]` instead of `[1, 1]`)
- 1 new test in `types.rs`: `parse_oneh_rejects_plus_j_metal`.
- `canon.rs` tests updated to `*j` for OneH (4 tests, no production
  code changed).

**Validated**
- 68 tests passing, 37/37 parity (counts unchanged: orbit is
  isomorphic with `*j` and `+j`; only representation differs).
- `Mo2S4` from `scripts/skeleton.sh` (Mo now at correct in-plane z).

### Fase 1B.2 — Parser OneT + Janus1H + Janus1T

**Added**
- `validate_row` for OneT, Janus1H, Janus1T.
- `all_rows`: OneT (6), Janus1H (12), Janus1T (12).
- 7 tests in `types.rs` + 6 tests in `children.rs` (including
  `fiber_theorem1_{one_t,janus1h,janus1t}`).
- `count.rs` example accepts all 8 options.
- `parity_canonjam.py` covers 37 counts (all 8 options).

**Implementation note**
- Options 6, 7, 8 use `*j` (fixed-site) for the metal, per Tabla 7
  of *Orbits as Multisets* and per the physics (metal is in-plane in
  1T TMDCs). Under `v = sign∘flip`, `+j` would map to `-j` (outside
  the row set) and break the group action; `*j` is invariant under
  `sign`, so the group action closes.
- Two test expectations corrected from `InvalidRowForOption` to
  `MismatchedLetters` (more specific, consistent with PL/BPL/BU/BBU).

**Validated**
- OneT: n=2..6 → 5, 21, 120, 666, 3960 (matches Paper 1, Table 3, BPL/BU).
- Janus1H: n=2..5 → 16, 144, 1776, 20736 (matches BBU).
- Janus1T: n=2..5 → 16, 144, 1776, 20736 (matches BBU).

### Fase 1B.1 — Parser BPL + BBU

**Added**
- `validate_row` for BPL (2×Up different letters) and BBU
  (Up + Down different letters, both orientations).
- `all_rows`: BPL (6), BBU (12).
- 7 tests in `types.rs` + 4 tests in `children.rs`.
- `count.rs` example accepts BPL and BBU.
- `parity_canonjam.py` includes BPL and BBU tables.

**Validated**
- BPL: n=2..6 → 5, 21, 120, 666, 3960 (matches Paper 1, Table 3).
- BBU: n=2..5 → 16, 144, 1776, 20736 (matches Paper 1, Table 3).

### Open
- **D8:** `pl_cell` uses `z_up = 0.01` (not 0.0) to satisfy V4.
  Refinement deferred to v0.2.0 (needs decision on V4 semantics for
  planar options).

---

## v0.1.0-alpha.3 — 2026-10-28 — Fase 1A complete

Parser, canonicalization, cell classifier, canonical augmentation, and
CIF export for options PL (1), BU (3), OneH (5). The theoretical kernel
of Papers 1-3 is fully implemented for these three options.

### Fase 1A summary

| Sub-fase | Delivered |
|---|---|
| 1A.1 | Parser PL/BU/OneH |
| 1A.2 | `kappa_fast` (Theorem 3) + column-major comparison |
| 1A.3 | Word-based cells (S3-palindromic predicate) |
| 1A.4 | `children` + `fiber` (Theorem 1) |
| 1A.5 | `augment_to` (Theorem 2) + parity vs paper tables |
| 1A.6 | `ReferenceCell` V1-V5 + CIF export |
| 1A.7 | Criterion benchmarks |

**57 tests** (41 core + 14 io + 2 e2e), 0 failures.
**Parity:** 15/15 counts match Paper 1, Table 3 and Paper 3, Table 4.
**Perf:** κ_fast 4.63–5.27× faster than κ_literal.

### Fase 1A.7 — Criterion benchmarks

**Added**
- `benches/canon.rs`: `kappa_literal` vs `kappa_fast` (PL/BU/OneH, k ∈ {4,6,8}), `sample_size(20)`.
- `benches/children.rs`: `children` + `fiber` (PL/BU/OneH, k ∈ {4,6}), `sample_size(10)`.
- `benches/augment.rs`: `augment_to` (PL/BU/OneH, n ∈ {3,4}), `sample_size(10)`.
  `n=5` skipped: ~500 ms/call for BU/OneH, unworkable for Criterion.
- `scripts/bench.sh`: `--quick` mode, filtered output, points to HTML report.
- `criterion = 0.5` as dev-dependency with `html_reports`.
- `docs/benchmarks.md` with measured results.

**Measured** (AMD Ryzen 5 4600H, release)
- κ_fast speedup vs κ_literal: **4.63–5.27×** (theoretical 6×).
- fiber vs children ratio: **7.88–8.74×**.
- augment_to n=3→4 growth: 3.16× (PL), 5.71× (BU), 5.61× (OneH).

### Fase 1A.6 — ReferenceCell V1-V5 + CIF tests

**Added**
- `ReferenceCellError::V2a`: symmetric buckled pairs (up == down)
  must have |z_up| == |z_down|.
- 10 tests in `reference_cell.rs` (V1, V2, V2a, V4, missing species).
- 4 tests in `cif.rs` (OneH MoS₂, PL monoelemental, BU buckled,
  invalid cell rejection). `count_atoms` verifies stoichiometry.
- `scripts/validate_cif.py`: external validation with pymatgen / ASE.

**Deferred**
- Janus1H (option 7) and 1T (option 6) CIF tests: parser support
  in Fase 1B (delivered in 1B.4).

### Fase 1A.5 — augment_to + parity vs paper tables

**Added**
- `children::augment_to(n, option)`: canonical augmentation
  (Paper 3, Theorem 2).
- `examples/count.rs`: `cargo run --example count -- <option> <n>`.
- `experiments/parity_canonjam.py`: parity vs paper tables.
- 4 tests: `augment_to_{pl,bu,oneh}_counts`,
  `augment_to_classes_are_canonical`,
  `burnside_pl_n6_is_70`,
  `literal_and_fast_agree_pl_k6_exhaustive`.

**Validated**
- PL: n=2..6 → 2, 4, 10, 25, 70 (matches Paper 1, Table 3).
- BU: n=2..6 → 5, 21, 120, 666, 3960.
- OneH: n=2..6 → 5, 21, 120, 666, 3960 (same as BPL/BU, per Paper 3,
  Table 4).
- PL n=6 = 70 independently verified via Burnside
  (`(3^6 + 3^3 + 3·1 + 3·3^3)/12 = 70`) and by exhaustive
  `kappa_literal == kappa_fast` on all 729 PL chains at k=6.

### Fase 1A.4 — children + fiber (Paper 3, Theorem 1)

**Added**
- `children.rs` (module under feature `children`):
  - `all_rows(option)`: PL (3 rows), BU (6), OneH (6).
  - `prepend(row, chain)`, `append(row, chain)`.
  - `children(chain)`: 2q chains (q prepends + q appends).
  - `fiber(chain)`: canonical children, deduplicated.
- Public re-exports in `lib.rs`.
- Feature `children` enabled by default; `children = ["canon"]`.
- 7 tests in `children.rs`, including `fiber_theorem1_random` (2160
  fiber comparisons across PL/BU/OneH, k ∈ {2,3,4}, all 12 g ∈ G).

**Validated**
- Theorem 1 of *Growing as Pruning* (fiber G-invariance) holds
  empirically for all tested chains and options.

### Fase 1A.3 — Word-based cells with S3-palindromic predicate

**Fixed**
- `cell_word` / `cell` now use **S3-palindromic** (∃σ ∈ S3 with
  σ(w) = w^rev), not strict-palindrome. This is the `pa` predicate
  of *Stacking as Words* (Definition 3). The previous strict-palindrome
  was under-counting PP and over-counting Pp/pP for k ≥ 3.
- `cell_nonemptiness_pattern` test: fixed k=2 expectations
  (pP(2) is nonempty, `[0,1]`; Ap(2) is empty, all length-2 words
  are S3-pal).

**Added**
- Public word-level API in `cells.rs`:
  - `word(chain) -> Vec<u8>` (column index of the distinguished token)
  - `is_palindromic_word(w)` (strict, Ramsdell H)
  - `is_s3_palindromic_word(w)` (Definition 3, the `pa` predicate)
  - `period_word(w)`, `is_periodic_word(w)`, `cell_word(w)`
- Chain-level wrappers: `is_s3_palindromic(chain)` (new), alongside
  existing `is_palindromic`, `period`, `is_periodic`, `cell`.
- Test `cell_nonemptiness_pattern`: exhaustive over all `3^k` words
  for k ∈ {2, 3, 4, 5, 6, 8} (7650 words), verifies Paper 2 Theorem 2.
- 8 tests in `cells.rs`.

### Fase 1A.2 — `kappa_fast` real (Theorem 3) + column-major fix

**Fixed**
- `kappa_literal` and `kappa_fast` now compare **column-major**
  (Definition 6 of *Orbits as Multisets*), not row-major. Equivalent
  for k=2 (both orders give the same minimum), divergent for k ≥ 3.
  This is the ordering used by `canonJAM`; the previous row-major
  would have failed bit-parity at k ≥ 3.

**Added**
- `JamChain::cols() -> [Vec<Token>; 3]`: the three columns as k-tuples.
- `kappa_fast` implements Theorem 3:
  `min(sort_cols(a), sort_cols(v(a)))`. Two evaluations of
  `sort_cols`, one comparison, no enumeration of the 12 group elements.
  O(k) with constant 2.
- 3 new tests in `canon.rs`:
  - `literal_and_fast_agree_on_mos2_bilayer_full` (exhaustive, OneH k=2, 36 chains)
  - `literal_and_fast_agree_pl_k3` (exhaustive, PL k=3, 27 chains)
  - `literal_and_fast_agree_random` (PL/BU/OneH at k ∈ {6,8,10,12}, N=100 each)
- `rand = 0.8` as a `[dev-dependencies]` in `laminar-core`.

### Fase 1A.1 — Parser for PL, BU, OneH

**Added**
- `validate_row` handles PL (2×Up same letter), BU (Up+Down same
  letter), OneH (Empty + Up + Buckled with `up == down`). *(OneH's
  `Up` was later changed to `Fixed` in Fase 1B.3, per Tabla 7.)*
- `ParseError::MismatchedLetters { row, option, first, second }` for
  PL/BU letter mismatches (e.g., `+j+0+k` in PL).
- `JamChain::from_rows(rows, option)` constructor for internal use
  (bypasses `parse` + validation; caller is responsible for validity).
- `describe_row(row) -> String` helper, produces `E=n, U=n, ...` for
  error messages.
- 7 new tests in `types.rs`: `parse_pl_rows`,
  `parse_pl_rejects_mismatched_letters`, `parse_pl_rejects_down_token`,
  `parse_bu_rows`, `parse_bu_rejects_mismatched_letters`,
  `parse_oneh_rows`, `from_rows_constructs_chain`.

**Changed**
- `ParseError::InvalidRowForOption.got`: `(usize, usize, usize)` →
  `String` (produced by `describe_row`).
- Convention: `parse_row` errors are wrapped in `Row { row, source }`;
  `validate_row` errors are returned **directly** with their own `row`
  field. This avoids `row N: row N: ...` in Display.

### Infrastructure

- `scripts/check.sh`: one-shot `fmt --check` + `build` + `test` +
  `skeleton.sh`, exits 0 only if all pass. First diagnostic when
  `cargo test` "does nothing" is `cargo test --no-run` (compiles tests
  without running them; `cargo build` does not).

---

## v0.1.0-alpha.2 — 2026-10-28 — Fase 0.5 complete

Walking skeleton for MoS₂ 1H bilayer, 5-crate workspace, and fixes for
the four bugs found during validation.

### Added
- `crates/laminar-io/` with `ReferenceCell`, `SiteOccupancy` (via `Token`),
  `to_cif`, and the V1–V5 validation rules.
- `crates/laminar-markov/` (stub, Fase 6).
- `crates/laminar-cli/` (stub, Fase 2).
- `crates/skeleton/` — walking skeleton for MoS₂ 1H bilayer.
- `crates/skeleton/tests/e2e.rs` — end-to-end test.
- `scripts/skeleton.sh` — one-shot build + run + external validation.
- `Token` implements `Ord` via an explicit `rank()`, per Definition 6
  of *Orbits as Multisets*.

### Changed
- **BREAKING:** renamed `laminar_core::Option` → `laminar_core::JamOption`
  to avoid shadowing `std::option::Option` from the prelude. All internal
  uses updated (types, group, canon, cells, skeleton, e2e test).
- `kappa_literal` compares by `a.rows().cmp(b.rows())` (`Token::Ord`),
  not by `a.raw().cmp(b.raw())` (ASCII string order).
- `to_cif` signature: third argument (`space_group`) is now `_space_group`
  (ignored in v0.1.0). The CIF always uses `P 1` for explicit atoms.
- Workspace version: `0.1.0-alpha.1` → `0.1.0-alpha.2`.

### Fixed
- **Bug 1:** `to_cif` wrote `_symmetry_space_group_name_H-M 'P 63/m m c'`
  with 6 explicit atoms. ASE interpreted the 6 atoms as the asymmetric
  unit and applied the 24 symmetry ops of P6₃/mmc, producing Mo4S8 instead
  of Mo2S4. **Fix:** always write `P 1` for explicit-atom CIF. A symmetric
  variant (asymmetric unit + non-trivial space group) is deferred to v0.2.0.
- **Bug 2:** `kappa_literal` used `raw().cmp()`, which orders by ASCII
  characters. ASCII has `'(' (0x28) < '*' (0x2A) < '+' (0x2B) < '-' (0x2D)`,
  so `(+k-k)` sorted before `*j`, contradicting Definition 6 of
  *Orbits as Multisets* (Fixed ≺ Buckled ≺ Empty ≺ Up ≺ Down).
  **Fix:** `Token::Ord` with the paper's ranks.
  Canonical of MoS₂ 1H bilayer: `(+k-k)+0+j/(+k-k)+0+j`.
- **Bug 3:** `enum Cell` variant `pP` triggered `non_camel_case_types`.
  **Fix:** `#[allow(non_camel_case_types)]` with a comment explaining the
  mixed-case encoding (uppercase = yes, lowercase = no).
- **Bug 4:** `Option` shadowing in `partial_cmp` return type.
  **Fix:** resolved by rename to `JamOption` (see Changed).

### Removed
- Old crates `jam-core`, `canonjamr`, `caugjamr` (merged into `laminar-core`).

---

## v0.1.0-alpha.1 — 2026-10-28 — Initial spec

- 5-crate architecture (core, io, markov, cli, py).
- Feature flags in core (`canon`, `children`, `cells`).
- `ReferenceCell` with `SiteOccupancy`.
- MCMC spec with Independent Sampler fallback.
- `JamChain` as a lightweight value type (no cached metadata).
- Time budget: 18–20 weeks.

### Open at this snapshot
- **D1:** Venue for Paper 4 (JOSS vs SoftwareX). Deadline: end of Fase 5.
- **D2:** `--augment-mode strict` in v0.2.0? Deadline: end of Fase 6.
- **D3:** Full space group or only P6₃/mmc / P6̄m2? Deadline: end of Fase 1.5.
- **D4:** `laminar-markov` without `laminar-io`? Deadline: end of Fase 4.
- **D5:** `to_ase()` via PyO3 or pure Python on `to_cif()`? Deadline: end of Fase 4.
- **D6:** Keep `JamOption` or rename. **Resolved:** keep `JamOption`.
- **D7:** Word projection for options 6 and 8 (no `+0` token).
  **Resolved:** use `*j` (Fixed) per *Stacking as Words* p.4.
