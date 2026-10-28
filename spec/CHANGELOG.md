# Changelog

All notable changes to Laminar are documented here.

The project is in **alpha** (0.1.0-alpha.N). The strict semver policy
of `spec/v0.1.0-alpha.1.md` §0 applies from `0.2.0-beta.1` onwards.
During alpha, public-type changes bump the alpha tag (e.g.,
`0.1.0-alpha.1` → `0.1.0-alpha.2`) and are recorded here with a
`**BREAKING**` marker.

## v0.1.0-alpha.2 — 2026-10-28 — Fase 0.5 complete

### Added
- `crates/laminar-io/` with `ReferenceCell`, `SiteOccupancy` (via `Token`),
  `to_cif`, and the V1–V5 validation rules.
- `crates/laminar-markov/` (stub, Fase 6).
- `crates/laminar-cli/` (stub, Fase 2).
- `crates/skeleton/` — walking skeleton for MoS₂ 1H bilayer.
- `crates/skeleton/tests/e2e.rs` — end-to-end test.
- `scripts/skeleton.sh` — one-shot build + run + external validation.
- `Token` now implements `Ord` via an explicit `rank()`, per
  Definition 6 of *Orbits as Multisets*.

### Changed
- **BREAKING:** renamed `laminar_core::Option` → `laminar_core::JamOption`
  to avoid shadowing `std::option::Option` from the prelude. All internal
  uses updated (types, group, canon, cells, skeleton, e2e test).
- `kappa_literal` now compares by `a.rows().cmp(b.rows())` (`Token::Ord`),
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

### Removed
- Old crates `jam-core`, `canonjamr`, `caugjamr` (merged into `laminar-core`).

### Open
- **D3:** Export full space group or only P6₃/mmc / P6̄m2?
  Deadline: end of Fase 1.5.
- **D6:** Whether to keep `JamOption` or rename to something shorter in a
  future version. **Decision:** keep `JamOption`; the shadowing cost of
  `Option` is higher than the naming cost of `JamOption`.

---

## v0.1.0-alpha.1 — 2026-10-28 — Initial spec

- 5-crate architecture (core, io, markov, cli, py).
- Feature flags in core (`canon`, `children`, `cells`).
- `ReferenceCell` with `SiteOccupancy`.
- MCMC spec with Independent Sampler fallback.
- `JamChain` as a lightweight value type (no cached metadata).
- Time budget: 18–20 weeks.
