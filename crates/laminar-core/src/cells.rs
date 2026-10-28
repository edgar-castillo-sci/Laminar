//! Cell classification (Section 3 of *Stacking as Words*).

use crate::types::{Cell, JamChain};

/// Compute the cell of a chain.
///
/// **Skeleton implementation:** only distinguishes palindrome vs not,
/// and period-1 vs not. The full period computation (any d | k) is Fase 1A.3.
pub fn cell(chain: &JamChain) -> Cell {
    let pal = is_palindromic(chain);
    let per = is_periodic(chain);
    match (pal, per) {
        (true, true) => Cell::PP,
        (true, false) => Cell::pP,
        (false, true) => Cell::Pp,
        (false, false) => Cell::Ap,
    }
}

/// Whether the chain is a strict palindrome (w == w^rev).
pub fn is_palindromic(chain: &JamChain) -> bool {
    let rows = chain.rows();
    rows.iter().zip(rows.iter().rev()).all(|(a, b)| a == b)
}

/// Whether the chain is periodic (w = u^m for some m >= 2).
///
/// **Skeleton implementation:** only checks period 1 (all rows identical).
/// Full period computation is Fase 1A.3.
pub fn is_periodic(chain: &JamChain) -> bool {
    let rows = chain.rows();
    if rows.len() < 2 {
        return false;
    }
    rows.iter().all(|row| row == &rows[0])
}

/// The minimal period of the chain, if periodic.
pub fn period(chain: &JamChain) -> Option<usize> {
    if is_periodic(chain) {
        Some(1)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::JamOption;

    #[test]
    fn mos2_bilayer_is_pp() {
        let chain = JamChain::parse("+j+0(+k-k)/+j+0(+k-k)", JamOption::OneH).unwrap();
        assert!(is_palindromic(&chain));
        assert!(is_periodic(&chain));
        assert_eq!(cell(&chain), Cell::PP);
    }
}
