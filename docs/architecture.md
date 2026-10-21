# Architecture

Laminar is organized as a Cargo workspace to ensure modularity and strict separation of concerns.

## Workspace Structure

- **`jam-core`**: Minimal dependencies. Handles the parsing of JAM strings into memory-efficient arrays, and implements the 12 symmetry operations.
- **`canonjamr`**: Depends on `jam-core`. Calculates the canonical key by applying the group operations and finding the lexicographical minimum. Uses `HashMap` for O(1) lookups.
- **`caugjamr`**: Depends on `canonjamr`. Implements the tree-search for canonical augmentation. This crate is designed to be parallelized using `rayon`.

## Design Goals
1. **Zero-cost abstractions**: Use Rust's type system to ensure correctness at compile time.
2. **Parallelism by default**: The generation of candidates in `caugjamr` is embarrassingly parallel.
3. **Interoperability**: Provide a clean C-FFI or PyO3 bindings in the future to interface with the Python ecosystem (`canonJAM` and `caugJAM`).
