//! κ: canonical representative (Theorem 1, *Orbits as Multisets*).
//!
//! `s` (serialization) is column-major (Definition 6): the chain is
//! serialized as three columns concatenated. `kappa_literal` and
//! `kappa_fast` both compare with `JamChain::cols()`.

use crate::group::{Group, GroupElement, Perm3};
use crate::types::JamChain;

/// The literal canonicalization: min over all 12 elements of G,
/// comparing by column-major serialization.
///
/// Reference implementation. O(|G| · k) = O(k) since |G| = 12 is constant.
pub fn kappa_literal(chain: &JamChain) -> JamChain {
    Group::elements()
        .iter()
        .map(|g| Group::apply(*g, chain))
        .min_by(|a, b| a.cols().cmp(&b.cols()))
        .expect("G is non-empty")
}

/// Order the three columns of `chain` lexicographically.
///
/// Returns the chain whose columns are sorted. This is the element
/// of the S3-orbit of `chain` with the minimal column-major
/// serialization (Theorem 3 of *Orbits as Multisets*).
fn sort_cols(chain: &JamChain) -> JamChain {
    let k = chain.k();
    let mut cols = chain.cols();
    cols.sort();
    let rows: Vec<[crate::types::Token; 3]> = (0..k)
        .map(|i| {
            [
                cols[0][i].clone(),
                cols[1][i].clone(),
                cols[2][i].clone(),
            ]
        })
        .collect();
    JamChain::from_rows(rows, chain.option())
}

/// The fast canonicalization: O(k) with constant 2 (Theorem 3).
///
/// κ(a) = min( sort_cols(a), sort_cols(v(a)) )
///
/// Two evaluations of `sort_cols` (one of them on `v(a)`), one
/// comparison. No enumeration of the 12 group elements.
pub fn kappa_fast(chain: &JamChain) -> JamChain {
    let sorted_a = sort_cols(chain);
    let v_a = Group::apply(
        GroupElement {
            sigma: Perm3::IDENTITY,
            v: true,
        },
        chain,
    );
    let sorted_v = sort_cols(&v_a);
    let ca = sorted_a.cols();
    let cv = sorted_v.cols();
    match ca.cmp(&cv) {
        std::cmp::Ordering::Less | std::cmp::Ordering::Equal => sorted_a,
        std::cmp::Ordering::Greater => sorted_v,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{JamOption, Token};
    use rand::{Rng, SeedableRng};
    use rand::rngs::StdRng;

    #[test]
    fn literal_and_fast_agree_on_mos2() {
        let chain = JamChain::parse("+j+0(+k-k)/+j+0(+k-k)", JamOption::OneH).unwrap();
        assert_eq!(kappa_literal(&chain).raw(), kappa_fast(&chain).raw());
    }

    #[test]
    fn kappa_is_idempotent() {
        let chain = JamChain::parse("+j+0(+k-k)/+j+0(+k-k)", JamOption::OneH).unwrap();
        let k1 = kappa_literal(&chain);
        let k2 = kappa_literal(&k1);
        assert_eq!(k1.raw(), k2.raw());
    }

    #[test]
    fn kappa_is_g_invariant() {
        let chain = JamChain::parse("+j+0(+k-k)/+j+0(+k-k)", JamOption::OneH).unwrap();
        let k = kappa_literal(&chain);
        for g in Group::elements() {
            let c = Group::apply(g, &chain);
            assert_eq!(kappa_literal(&c).raw(), k.raw());
        }
    }

    #[test]
    fn literal_and_fast_agree_on_mos2_bilayer_full() {
        // 2-layer OneH: all distinct permutations of [+0, +j, (+k-k)]
        // (6 per row) → 6² = 36 chains.
        let row_tokens = [
            ("+0+j(+k-k)", 0usize),
            ("+0(+k-k)+j", 1),
            ("+j+0(+k-k)", 2),
            ("+j(+k-k)+0", 3),
            ("(+k-k)+0+j", 4),
            ("(+k-k)+j+0", 5),
        ];
        for (s0, _) in &row_tokens {
            for (s1, _) in &row_tokens {
                let s = format!("{}/{}", s0, s1);
                let chain = JamChain::parse(&s, JamOption::OneH).unwrap();
                assert_eq!(
                    kappa_literal(&chain).raw(),
                    kappa_fast(&chain).raw(),
                    "mismatch for {}",
                    s
                );
            }
        }
    }

    #[test]
    fn literal_and_fast_agree_pl_k3() {
        // PL: each row is a permutation of [+0, +j, +j] (3 per row) → 3³ = 27.
        let rows = ["+0+j+j", "+j+0+j", "+j+j+0"];
        for r0 in &rows {
            for r1 in &rows {
                for r2 in &rows {
                    let s = format!("{}/{}/{}", r0, r1, r2);
                    let chain = JamChain::parse(&s, JamOption::PL).unwrap();
                    assert_eq!(
                        kappa_literal(&chain).raw(),
                        kappa_fast(&chain).raw(),
                        "PL k=3 mismatch for {}",
                        s
                    );
                }
            }
        }
    }

    #[test]
    fn literal_and_fast_agree_random() {
        // Random chains for PL, BU, OneH at k ∈ {6, 8, 10, 12}.
        // N = 100 per (option, k).
        let mut rng = StdRng::seed_from_u64(0xC0FFEE);

        for option in [JamOption::PL, JamOption::BU, JamOption::OneH] {
            for k in [6usize, 8, 10, 12] {
                for _ in 0..100 {
                    let rows: Vec<[Token; 3]> = (0..k)
                        .map(|_| random_row(option, &mut rng))
                        .collect();
                    let chain = JamChain::from_rows(rows, option);
                    assert_eq!(
                        kappa_literal(&chain).raw(),
                        kappa_fast(&chain).raw(),
                        "random mismatch: {:?} k={} chain={}",
                        option,
                        k,
                        chain.raw()
                    );
                }
            }
        }
    }

    // --------------- helpers for random generation ---------------

    fn random_row<R: Rng>(option: JamOption, rng: &mut R) -> [Token; 3] {
        match option {
            JamOption::PL => {
                let letters = ["j"];
                let letter = letters[rng.gen_range(0..letters.len())];
                let mut row = [
                    Token::Empty,
                    Token::Up(letter.into()),
                    Token::Up(letter.into()),
                ];
                shuffle(&mut row, rng);
                row
            }
            JamOption::BU => {
                let letters = ["j"];
                let letter = letters[rng.gen_range(0..letters.len())];
                let mut row = [
                    Token::Empty,
                    Token::Up(letter.into()),
                    Token::Down(letter.into()),
                ];
                shuffle(&mut row, rng);
                row
            }
            JamOption::OneH => {
                let letters = ["k"];
                let letter = letters[rng.gen_range(0..letters.len())];
                let mut row = [
                    Token::Empty,
                    Token::Up("j".into()),
                    Token::Buckled {
                        up: letter.into(),
                        down: letter.into(),
                    },
                ];
                shuffle(&mut row, rng);
                row
            }
            _ => panic!("random_row not implemented for {:?}", option),
        }
    }

    fn shuffle<T, R: Rng>(arr: &mut [T; 3], rng: &mut R) {
        for i in (1..3).rev() {
            let j = rng.gen_range(0..=i);
            arr.swap(i, j);
        }
    }
}
