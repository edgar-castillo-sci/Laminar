//! Core types and algorithms for JAM chains.
//!
//! Implements the theoretical kernel of Papers 1-3:
//! - `types`: JamChain, Token, JamOption, Cell, Stabilizer
//! - `group`: G = S3 × Z/2, action, orbits
//! - `canon`: κ (Theorem 3 of *Orbits as Multisets*)
//! - `cells`: PP/Pp/pP/Ap (Section 3 of *Stacking as Words*)
//! - `children`: prepend/append/fiber (Theorem 1 of *Growing as Pruning*)

pub mod group;
pub mod types;

#[cfg(feature = "canon")]
pub mod canon;

#[cfg(feature = "cells")]
pub mod cells;

#[cfg(feature = "children")]
pub mod children;

pub use group::{Group, GroupElement, Perm3};
pub use types::{Cell, JamChain, JamOption, ParseError, Stabilizer, Token};

#[cfg(feature = "canon")]
pub use canon::{kappa_fast, kappa_literal};

#[cfg(feature = "cells")]
pub use cells::{cell, is_palindromic, is_periodic, period};

#[cfg(feature = "children")]
pub use children::{all_rows, append, children, fiber, prepend};
