# Roadmap

## v0.1.0 (Current) - Specification
- Define workspace and crate structure.
- Define public APIs via `todo!()` stubs.

## v0.2.0 - Core & Filtering
- Implement `jam-core` parsing and symmetry operations.
- Implement `canonjamr` canonical keys and O(m) filtering.
- Benchmark against `canonJAM` (Python).

## v0.3.0 - Augmentation
- Implement `caugjamr` pruning algorithm.
- Benchmark against `caugJAM` (Python).

## v0.4.0 - Parallelization
- Introduce Rayon for parallel candidate generation.
- Target: \(n = 50\) layers in reasonable time.

## v1.0.0 - Monte Carlo & Analysis
- Add Monte Carlo sampling for infinite layer limits.
- Finalize documentation and API stability.
