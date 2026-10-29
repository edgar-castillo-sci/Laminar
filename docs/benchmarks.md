# Benchmarks (v0.1.0-alpha.3)

**Hardware:** AMD Ryzen 5 4600H, Fedora 44, Rust 1.98.1, release profile.
**Date:** 2026-10-28.
**Command:** `cargo bench -p laminar-core` (Criterion 0.5, `html_reports`).

## κ: literal vs fast

| Opción | k | literal | fast | Speedup |
|---|---|---|---|---|
| PL | 4 | 782.66 µs | 168.99 µs | 4.63× |
| PL | 6 | 1301.8 µs | 280.64 µs | 4.64× |
| PL | 8 | 1982.0 µs | 376.20 µs | 5.27× |
| BU | 4 | 907.87 µs | 185.82 µs | 4.89× |
| BU | 6 | 1523.4 µs | 323.51 µs | 4.71× |
| BU | 8 | 2192.7 µs | 429.02 µs | 5.11× |
| OneH | 4 | 1299.5 µs | 273.76 µs | 4.75× |
| OneH | 6 | 2134.7 µs | 410.49 µs | 5.20× |
| OneH | 8 | 3012.3 µs | 599.49 µs | 5.02× |

**Theoretical speedup:** 6× (12 evaluations → 2).
**Measured:** ~4.6–5.3×. The gap is `sort_cols` allocation cost
(`Vec<Token>` × 3). Fase 2+ may narrow it via buffer reuse.

## children vs fiber

| Opción | k | children | fiber | Ratio |
|---|---|---|---|---|
| PL | 4 | 91.9 µs | 803.5 µs | 8.74× |
| PL | 6 | 137.9 µs | 1127.3 µs | 8.18× |
| BU | 4 | 218.6 µs | 1722.7 µs | 7.88× |
| BU | 6 | 288.1 µs | 2452.9 µs | 8.52× |
| OneH | 4 | 291.8 µs | 2312.0 µs | 7.92× |
| OneH | 6 | 378.9 µs | 3190.4 µs | 8.42× |

`fiber` calls `kappa_fast` per child, so the ~8× cost is expected
(κ + hashing + allocation per child).

## augment_to growth

| Opción | n=3 | n=4 | n=4/n=3 |
|---|---|---|---|
| PL | 67.8 µs | 214.0 µs | 3.16× |
| BU | 327.9 µs | 1874.2 µs | 5.71× |
| OneH | 444.7 µs | 2493.6 µs | 5.61× |

Growth is consistent with class counts (PL: 3→10, BU/OneH: 5→120).
PL grows slower because q=3 (vs q=6).

## Notes

- `--quick` mode (`cargo bench -- --quick`) reduces total time to ~30s
  at the cost of precision. Use it during development.
- Full run: ~5 min. Use for `docs/benchmarks.md` (this file).
- Gnuplot not installed → plotters backend. HTML report at
  `target/criterion/report/index.html`.
