//! Cell classification (Section 3 of *Stacking as Words*).
//!
//! Two Boolean properties:
//! - `pa`: S3-palindromic (∃σ ∈ S3 with σ(w) = w^rev). NOT strict palindrome.
//! - `per`: periodic (w = u^m, m ≥ 2).
//!
//! The word projection (see `word`) uses the distinguished token per
//! *Stacking as Words* p.4: `*x` (Fixed) for options 5-8, `+0` (Empty)
//! for options 1-4.

use crate::types::{Cell, JamChain, JamOption, Token};

const S3_PERMS: [[u8; 3]; 6] = [
    [0, 1, 2], // id
    [1, 0, 2], // (0 1)
    [2, 1, 0], // (0 2)
    [0, 2, 1], // (1 2)
    [1, 2, 0], // (0 1 2)
    [2, 0, 1], // (0 2 1)
];

// ---------- word-level primitives ----------

/// The word of a chain: the column index of the **distinguished token**
/// in each row.
///
/// Per *Stacking as Words* (p.4): "the fixed-site token `*x` if the
/// row contains one (options 5-8); otherwise, the empty-site token
/// `+0` (all options)."
///
/// # Panics
///
/// Panics if a row lacks the expected distinguished token.
pub fn word(chain: &JamChain) -> Vec<u8> {
    let use_fixed = matches!(
        chain.option(),
        JamOption::OneH | JamOption::OneT | JamOption::Janus1H | JamOption::Janus1T
    );
    chain
        .rows()
        .iter()
        .enumerate()
        .map(|(i, row)| {
            let target = if use_fixed {
                row.iter().position(|t| matches!(t, Token::Fixed(_)))
            } else {
                row.iter().position(|t| matches!(t, Token::Empty))
            };
            target.unwrap_or_else(|| {
                panic!(
                    "row {} of chain (option {:?}) has no `{}` token; \
                     word() expects {} per *Stacking as Words* p.4",
                    i,
                    chain.option(),
                    if use_fixed { "*x" } else { "+0" },
                    if use_fixed { "*x (Fixed)" } else { "+0 (Empty)" }
                )
            }) as u8
        })
        .collect()
}

/// Whether the word is a strict palindrome (w == w^rev).
pub fn is_palindromic_word(w: &[u8]) -> bool {
    w.iter().zip(w.iter().rev()).all(|(a, b)| a == b)
}

/// Whether the word is S3-palindromic: ∃σ ∈ S3 with σ(w) == w^rev.
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

    // ---------- word() token selection ----------

    #[test]
    fn word_of_mos2_bilayer() {
        // Option 5 uses *j (Fixed). Mo at col 0, empty at col 1, S at col 2.
        // word = column of *j = [0, 0].
        let chain = JamChain::parse("*j+0(+k-k)/*j+0(+k-k)", JamOption::OneH).unwrap();
        assert_eq!(word(&chain), vec![0, 0]);
    }

    #[test]
    fn word_uses_fixed_for_options_5_through_8() {
        // Option 5: *j at column 1 → word [1].
        let chain = JamChain::parse("+0*j(+k-k)", JamOption::OneH).unwrap();
        assert_eq!(word(&chain), vec![1]);

        // Option 6: *j at column 0 → word [0].
        let chain = JamChain::parse("*j+k-k", JamOption::OneT).unwrap();
        assert_eq!(word(&chain), vec![0]);

        // Option 7: *j at column 2 → word [2].
        let chain = JamChain::parse("+0(+k-l)*j", JamOption::Janus1H).unwrap();
        assert_eq!(word(&chain), vec![2]);

        // Option 8: *j at column 1 → word [1].
        let chain = JamChain::parse("+k*j-l", JamOption::Janus1T).unwrap();
        assert_eq!(word(&chain), vec![1]);
    }

    #[test]
    fn word_uses_empty_for_options_1_through_4() {
        // Option 1 (PL): +0 at column 0 → word [0].
        let chain = JamChain::parse("+0+j+j", JamOption::PL).unwrap();
        assert_eq!(word(&chain), vec![0]);

        // Option 3 (BU): +0 at column 1 → word [1].
        let chain = JamChain::parse("+j+0-j", JamOption::BU).unwrap();
        assert_eq!(word(&chain), vec![1]);
    }

    // ---------- word-level primitives ----------

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
        let cases: &[&[u8]] = &[
            &[0u8], &[1], &[2],
            &[0, 0], &[0, 1], &[1, 2], &[2, 0],
        ];
        for w in cases {
            assert!(is_s3_palindromic_word(w), "should be S3-pal: {:?}", w);
        }
        assert!(is_s3_palindromic_word(&[0, 1, 0, 1]));
        assert!(!is_palindromic_word(&[0, 1, 0, 1]));
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
        assert_eq!(cell_word(&[0, 0, 0, 0]), Cell::PP);
        assert_eq!(cell_word(&[0, 1, 0, 1]), Cell::PP);
        assert_eq!(cell_word(&[0, 1, 0]), Cell::pP);
        assert_eq!(cell_word(&[0, 1, 1, 0]), Cell::pP);
        assert_eq!(cell_word(&[0, 0, 1, 0, 0, 1]), Cell::Pp);
        assert_eq!(cell_word(&[0, 0, 1]), Cell::Ap);
        assert_eq!(cell_word(&[0, 0, 1, 0, 0, 2]), Cell::Ap);
    }

    // ---------- chain-level ----------

    #[test]
    fn mos2_bilayer_is_pp() {
        let chain = JamChain::parse("*j+0(+k-k)/*j+0(+k-k)", JamOption::OneH).unwrap();
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
            assert!(has[0], "PP empty at k={}", k);
            let expected_pp = is_composite(k) && k != 4;
            assert_eq!(has[1], expected_pp, "Pp nonemptiness at k={}", k);
            assert!(has[2], "pP empty at k={}", k);
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
