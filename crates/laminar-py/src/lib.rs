//! Python bindings for Laminar (PyO3).
//!
//! Build with:
//!     cd crates/laminar-py
//!     maturin develop --release
//!
//! The `python` feature gates all PyO3 code. It is off by default so
//! `cargo build --workspace` does not require Python at build time.

#[cfg(feature = "python")]
mod bindings;

#[cfg(feature = "python")]
pub use bindings::*;
