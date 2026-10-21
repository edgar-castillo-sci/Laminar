# Laminar

**Status: Specification phase (v0.1.0).** The API and crate structure are defined; implementations are planned for v0.2.0 onwards. See `ROADMAP.md` for milestones.

Laminar is a high-performance Rust suite for the generation, filtering, and analysis of honeycomb layered materials using the JAM framework.

## Architecture
The workspace is divided into three main crates:
- `jam-core`: Core data structures and parsing for JAM strings.
- `canonjamr`: High-performance canonical filtering (O(m) algorithm).
- `caugjamr`: Canonical augmentation (pruning) for efficient generation.

## Genealogy
This project is the logical conclusion of a research line:
1. **JAM** (Arcudia et al., 2023): The original string representation.
2. **canonJAM** (Python): Formalization of the symmetry group and O(m) filtering.
3. **caugJAM** (Python): Proof-of-concept for canonical augmentation.
4. **Laminar** (Rust): Production-grade, parallelized implementation for n ≥ 50 layers.
