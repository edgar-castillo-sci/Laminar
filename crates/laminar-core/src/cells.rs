//! Cell classification (Section 3 of *Stacking as Words*).
//!
//! The four cells PP, Pp, pP, Ap are defined on the **word**
//! (the projection of a chain to {A,B,C}^k), not on the chain.
//!
//! Two Boolean properties:
//! - `pa`: S3-palindromic (∃σ ∈ S3 with σ(w) = w^rev). NOT strict palindrome.
//! - `per`: periodic (w = u^m, m ≥ 2).
//!
//! For options 1-5 and 7, the distinguished token is `+0` (one per row),
//! so the word is well-defined. Options 6 and 8 (1T and Janus1T) have
//! no `+0` in their rows; their word projection is deferred to Fase 1B.

use crate::types::{Cell, JamChain, Token};

const S3_PERMS: [[u8; 3]; 6] = [
    [0, 1, 2], // id
    [1, 0, 2], // (0 1)
    [2, 1, 0], // (0 2)
    [0, 2, 1], // (1 2)
    [1, 2, 0], // (0 1 2)
    [2, 0, 1], // (0 2 1)
];

// ---------- word-level primitives ----------

/// The word of a chain: column index of `+0` in each row.
pub fn word(chain: &JamChain) -> Vec<u8> {
    chain
        .rows()
        .iter()
        .enumerate()
        .map(|(i, row)| {
            row.iter()
                .position(|t| matches!(t, Token::Empty))
                .unwrap_or_else(|| {
                    panic!(
                        "row {} of chain (option {:?}) has no `+0` token; \
                         word() is only defined for options with a distinguished `+0` \
                         (1-5, 7). Options 6 and 8 are Fase 1B.",
                        i,
                        chain.option()
                    )
                }) as u8
        })
        .collect()
}

/// Whether the word is a strict palindrome (w == w^rev).
///
/// This is the `H` class (Ramsdell), not the cell predicate.
pub fn is_palindromic_word(w: &[u8]) -> bool {
    w.iter().zip(w.iter().rev()).all(|(a, b)| a == b)
}

/// Whether the word is S3-palindromic: ∃σ ∈ S3 with σ(w) == w^rev.
///
/// This is the `pa` predicate of the cell classification
/// (Definition 3 of *Stacking as Words*). Broader than `is_palindromic_word`.
pub fn is_s3_palindromic_word(w: &[u8]) -> bool {
    let n = w.len();
    if n == 0 {
        return true;
    }
    for sigma in S3_PERMS.iter() {
        if (0..n).all(|i| sigma[w[i] as usize] == w[n - 1 - i]) {
            return true;
        }
    }
    false
}

/// Minimal period of the word, if periodic (w = u^m, m ≥ 2).
pub fn period_word(w: &[u8]) -> Option<usize> {
    let n = w.len();
    if n < 2 {
        return None;
    }
    for d in 1..n {
        if n % d != 0 {
            continue;
        }
        if (d..n).all(|i| w[i] == w[i - d]) {
            return Some(d);
        }
    }
    None
}

pub fn is_periodic_word(w: &[u8]) -> bool {
    period_word(w).is_some()
}

/// The cell of a word (S3-palindromic + periodic).
pub fn cell_word(w: &[u8]) -> Cell {
    let pal = is_s3_palindromic_word(w);
    let per = is_periodic_word(w);
    match (pal, per) {
        (true, true) => Cell::PP,
        (true, false) => Cell::pP,
        (false, true) => Cell::Pp,
        (false, false) => Cell::Ap,
    }
}

// ---------- chain-level wrappers ----------

pub fn is_palindromic(chain: &JamChain) -> bool {
    is_palindromic_word(&word(chain))
}

pub fn is_s3_palindromic(chain: &JamChain) -> bool {
    is_s3_palindromic_word(&word(chain))
}

pub fn period(chain: &JamChain) -> Option<usize> {
    period_word(&word(chain))
}

pub fn is_periodic(chain: &JamChain) -> bool {
    is_periodic_word(&word(chain))
}

pub fn cell(chain: &JamChain) -> Cell {
    cell_word(&word(chain))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{JamOption, Token};

    #[test]
    fn word_of_mos2_bilayer() {
        let chain = JamChain::parse("+j+0(+k-k)/+j+0(+k-k)", JamOption::OneH).unwrap();
        assert_eq!(word(&chain), vec![1, 1]);
    }

    #[test]
    fn is_palindromic_word_basic() {
        assert!(is_palindromic_word(&[0, 1, 0]));
        assert!(is_palindromic_word(&[0, 0]));
        assert!(is_palindromic_word(&[1]));
        assert!(is_palindromic_word(&[]));
        assert!(!is_palindromic_word(&[0, 1, 2]));
        assert!(!is_palindromic_word(&[0, 1]));
    }

    #[test]
    fn is_s3_palindromic_word_basic() {
        // Every word of length ≤ 2 is S3-palindromic.
        // Use &[u8] slices: arrays of different sizes can't be iterated together.
        let cases: &[&[u8]] = &[
            &[0u8], &[1], &[2],
            &[0, 0], &[0, 1], &[1, 2], &[2, 0],
        ];
        for w in cases {
            assert!(is_s3_palindromic_word(w), "should be S3-pal: {:?}", w);
        }
        // [0,1,0,1] is S3-pal via σ=(0 1), though not strict-pal.
        assert!(is_s3_palindromic_word(&[0, 1, 0, 1]));
        assert!(!is_palindromic_word(&[0, 1, 0, 1]));
        // [0,0,1] is not S3-pal (no σ works).
        assert!(!is_s3_palindromic_word(&[0, 0, 1]));
        assert!(!is_palindromic_word(&[0, 0, 1]));
    }

    #[test]
    fn period_word_basic() {
        assert_eq!(period_word(&[0, 0, 1, 0, 0, 1]), Some(3));
        assert_eq!(period_word(&[0, 0, 0, 0]), Some(1));
        assert_eq!(period_word(&[0, 1, 0, 1]), Some(2));
        assert_eq!(period_word(&[0, 1, 2, 0, 1, 2]), Some(3));
        assert_eq!(period_word(&[0, 1, 2, 0, 1, 3]), None);
        assert_eq!(period_word(&[0, 1]), None);
        assert_eq!(period_word(&[0, 1, 2]), None);
        assert_eq!(period_word(&[0]), None);
    }

    #[test]
    fn cell_word_basic() {
        // PP: S3-pal + per.
        assert_eq!(cell_word(&[0, 0, 0, 0]), Cell::PP);
        assert_eq!(cell_word(&[0, 1, 0, 1]), Cell::PP); // S3-pal via (0 1)
        // pP: S3-pal + non-per.
        assert_eq!(cell_word(&[0, 1, 0]), Cell::pP);
        assert_eq!(cell_word(&[0, 1, 1, 0]), Cell::pP);
        // Pp: ¬S3-pal + per.
        assert_eq!(cell_word(&[0, 0, 1, 0, 0, 1]), Cell::Pp);
        // Ap: ¬S3-pal + non-per.
        assert_eq!(cell_word(&[0, 0, 1]), Cell::Ap);
        assert_eq!(cell_word(&[0, 0, 1, 0, 0, 2]), Cell::Ap);
    }

    #[test]
    fn mos2_bilayer_is_pp() {
        let chain = JamChain::parse("+j+0(+k-k)/+j+0(+k-k)", JamOption::OneH).unwrap();
        assert_eq!(cell(&chain), Cell::PP);
        assert_eq!(period(&chain), Some(1));
        assert!(is_palindromic(&chain));
        assert!(is_s3_palindromic(&chain));
    }

    #[test]
    fn chain_and_word_agree() {
        let rows = vec![
            [Token::Empty, Token::Up("j".into()), Token::Up("j".into())],
            [Token::Up("j".into()), Token::Empty, Token::Up("j".into())],
            [Token::Empty, Token::Up("j".into()), Token::Up("j".into())],
        ];
        let chain = JamChain::from_rows(rows, JamOption::PL);
        assert_eq!(word(&chain), vec![0, 1, 0]);
        assert_eq!(cell(&chain), Cell::pP);
    }

    #[test]
    fn cell_nonemptiness_pattern() {
        for k in [2usize, 3, 4, 5, 6, 8] {
            let mut has = [false; 4];
            let total = 3usize.pow(k as u32);
            for n in 0..total {
                let mut w = vec![0u8; k];
                let mut m = n;
                for i in 0..k {
                    w[i] = (m % 3) as u8;
                    m /= 3;
                }
                match cell_word(&w) {
                    Cell::PP => has[0] = true,
                    Cell::Pp => has[1] = true,
                    Cell::pP => has[2] = true,
                    Cell::Ap => has[3] = true,
                }
            }
            // PP: always nonempty (k >= 2, word [0]^k).
            assert!(has[0], "PP empty at k={}", k);
            // Pp: nonempty iff k composite, k >= 6, k != 4 (Paper 2, Thm 2).
            let expected_pp = is_composite(k) && k != 4;
            assert_eq!(has[1], expected_pp, "Pp nonemptiness at k={}", k);
            // pP: nonempty for all k >= 2 ([0,1] at k=2).
            assert!(has[2], "pP empty at k={}", k);
            // Ap: nonempty for k >= 3 (per paper); empty at k=2.
            if k >= 3 {
                assert!(has[3], "Ap empty at k={}", k);
            } else {
                assert!(!has[3], "Ap should be empty at k=2");
            }
        }
    }

    fn is_composite(k: usize) -> bool {
        if k < 4 {
            return false;
        }
        (2..k).any(|d| k % d == 0)
    }
}
