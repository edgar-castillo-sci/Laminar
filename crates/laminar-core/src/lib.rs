//! Core types and algorithms for JAM chains.
//!
//! Implements the theoretical kernel of Papers 1-3:
//! - `types`: JamChain, Token, Option, Cell, Stabilizer
//! - `group`: G = S3 × Z/2, action, orbits
//! - `canon`: κ (Theorem 3 of *Orbits as Multisets*)
//! - `cells`: PP/Pp/pP/Ap (Section 3 of *Stacking as Words*)

pub mod group;
pub mod types;

#[cfg(feature = "canon")]
pub mod canon;

#[cfg(feature = "cells")]
pub mod cells;

pub use group::{Group, GroupElement, Perm3};
pub use types::{Cell, JamChain, JamOption, ParseError, Stabilizer, Token};
