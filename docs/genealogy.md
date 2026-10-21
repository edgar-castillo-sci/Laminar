# Project Genealogy

This document traces the intellectual and code lineage of the Laminar project.

1. **JAM (2023)**: The foundational paper by Arcudia et al. introduced the JAM string notation to represent honeycomb multilayers. The original implementation (`xjam2`) used pairwise comparison for duplicate detection.
2. **canonJAM (2026)**: Formalized the symmetry group (order 12) and introduced an \(O(m)\) canonicalization algorithm in Python. This proved that the pairwise bottleneck could be eliminated.
3. **caugJAM (2026)**: Extended the canonicalization into a pruning strategy (canonical augmentation), proving that only unique canonical representatives need to be extended to generate the next layer.
4. **Laminar (Future)**: The Rust implementation that takes these mathematical insights and executes them at scale, enabling the exploration of systems with 50+ layers.
