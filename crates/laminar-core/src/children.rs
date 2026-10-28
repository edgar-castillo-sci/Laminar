//! Canonical augmentation (Paper 3, *Growing as Pruning*).
//!
//! Implements:
//! - `all_rows(option)`: the single-layer rows of an option.
//! - `prepend(row, chain)`: insert `row` at position 1.
//! - `append(row, chain)`: insert `row` at position k+1.
//! - `children(chain)`: 2q chains (q prepends + q appends).
//! - `fiber(chain)`: canonical children, deduplicated.
//!
//! Theorem 1 (Paper 3): `fiber(g·a) == fiber(a)` for all `g ∈ G`.
//! Proof: Lemma 4 gives `Children(g·a) = g · Children(a)`, and κ is
//! G-invariant, so κ(Children(g·a)) = κ(Children(a)).

use crate::canon::kappa_fast;
use crate::types::{JamChain, JamOption, Token};

/// Enumerate the single-layer rows of a given option.
///
/// Only options PL (1), BU (3), and OneH (5) are implemented in Fase 1A.
pub fn all_rows(option: JamOption) -> Vec<[Token; 3]> {
    match option {
        // PL: all distinct permutations of [+0, +j, +j] → 3 rows.
        JamOption::PL => {
            let j = "j".to_string();
            vec![
                [Token::Empty, Token::Up(j.clone()), Token::Up(j.clone())],
                [Token::Up(j.clone()), Token::Empty, Token::Up(j.clone())],
                [Token::Up(j.clone()), Token::Up(j.clone()), Token::Empty],
            ]
        }
        // BU: all distinct permutations of [+0, +j, -j] → 6 rows.
        JamOption::BU => {
            let j = "j".to_string();
            permute_three(Token::Empty, Token::Up(j.clone()), Token::Down(j))
        }
        // OneH: all distinct permutations of [+0, +j, (+k-k)] → 6 rows.
        JamOption::OneH => {
            let k = "k".to_string();
            permute_three(
                Token::Empty,
                Token::Up("j".to_string()),
                Token::Buckled {
                    up: k.clone(),
                    down: k,
                },
            )
        }
        other => unimplemented!(
            "all_rows not implemented for {:?} (Fase 1B: BPL, BBU, OneT, Janus1H, Janus1T)",
            other
        ),
    }
}

/// All 6 permutations of 3 distinct tokens.
fn permute_three(a: Token, b: Token, c: Token) -> Vec<[Token; 3]> {
    vec![
        [a.clone(), b.clone(), c.clone()],
        [a.clone(), c.clone(), b.clone()],
        [b.clone(), a.clone(), c.clone()],
        [b.clone(), c.clone(), a.clone()],
        [c.clone(), a.clone(), b.clone()],
        [c.clone(), b.clone(), a],
    ]
}

/// Prepend a row to a chain: `row` is layer 1, the chain's rows are 2..k+1.
pub fn prepend(row: [Token; 3], chain: &JamChain) -> JamChain {
    let mut rows = Vec::with_capacity(chain.k() + 1);
    rows.push(row);
    rows.extend(chain.rows().iter().cloned());
    JamChain::from_rows(rows, chain.option())
}

/// Append a row to a chain: the chain's rows are 1..k, `row` is layer k+1.
pub fn append(row: [Token; 3], chain: &JamChain) -> JamChain {
    let mut rows: Vec<[Token; 3]> = chain.rows().iter().cloned().collect();
    rows.push(row);
    JamChain::from_rows(rows, chain.option())
}

/// The children of a chain: 2q chains (q prepends + q appends).
///
/// May contain duplicates (e.g., when a prepend and an append coincide).
/// Use `fiber` if you want the deduplicated canonical children.
pub fn children(chain: &JamChain) -> Vec<JamChain> {
    let rows = all_rows(chain.option());
    let mut out = Vec::with_capacity(2 * rows.len());
    for row in &rows {
        out.push(prepend(row.clone(), chain));
    }
    for row in &rows {
        out.push(append(row.clone(), chain));
    }
    out
}

/// The fiber of a chain: the set of canonical children (deduplicated).
///
/// Theorem 1 (Paper 3): `fiber(g·a) == fiber(a)` for all `g ∈ G`.
pub fn fiber(chain: &JamChain) -> Vec<JamChain> {
    let children = children(chain);
    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();
    for child in children {
        let canon = kappa_fast(&child);
        if seen.insert(canon.raw()) {
            out.push(canon);
        }
    }
    out
}

#[cfg(test)]
    mod tests {
        use super::*;
        use crate::group::Group;
    
        fn fiber_set(chain: &JamChain) -> std::collections::HashSet<String> {
            fiber(chain).into_iter().map(|c| c.raw()).collect()
        }

    #[test]
    fn all_rows_count() {
        assert_eq!(all_rows(JamOption::PL).len(), 3);
        assert_eq!(all_rows(JamOption::BU).len(), 6);
        assert_eq!(all_rows(JamOption::OneH).len(), 6);
    }

    #[test]
    fn prepend_and_append() {
        let chain = JamChain::parse("+0+j+j", JamOption::PL).unwrap();
        let f = [Token::Empty, Token::Up("j".into()), Token::Up("j".into())];
        let p = prepend(f.clone(), &chain);
        assert_eq!(p.raw(), "+0+j+j/+0+j+j");
        assert_eq!(p.k(), 2);
        let a = append(f, &chain);
        assert_eq!(a.raw(), "+0+j+j/+0+j+j");
        assert_eq!(a.k(), 2);
    }

    #[test]
    fn children_size() {
        let chain_pl = JamChain::parse("+0+j+j", JamOption::PL).unwrap();
        assert_eq!(children(&chain_pl).len(), 6); // 2 * 3
        let chain_bu = JamChain::parse("+0+j-j", JamOption::BU).unwrap();
        assert_eq!(children(&chain_bu).len(), 12); // 2 * 6
        let chain_1h = JamChain::parse("+0+j(+k-k)", JamOption::OneH).unwrap();
        assert_eq!(children(&chain_1h).len(), 12); // 2 * 6
    }

    #[test]
    fn fiber_theorem1_pl() {
        let chains = [
            JamChain::parse("+0+j+j", JamOption::PL).unwrap(),
            JamChain::parse("+0+j+j/+j+0+j", JamOption::PL).unwrap(),
        ];
        for chain in &chains {
            let f_a = fiber_set(chain);
            for g in Group::elements() {
                let ga = Group::apply(g, chain);
                assert_eq!(
                    f_a,
                    fiber_set(&ga),
                    "fiber mismatch for g={:?}, chain={}",
                    g,
                    chain.raw()
                );
            }
        }
    }

    #[test]
    fn fiber_theorem1_bu() {
        let chain = JamChain::parse("+0+j-j", JamOption::BU).unwrap();
        let f_a = fiber_set(&chain);
        for g in Group::elements() {
            let ga = Group::apply(g, &chain);
            assert_eq!(f_a, fiber_set(&ga), "BU fiber mismatch for g={:?}", g);
        }
    }

    #[test]
    fn fiber_theorem1_oneh() {
        let chain = JamChain::parse("+0+j(+k-k)/+j+0(+k-k)", JamOption::OneH).unwrap();
        let f_a = fiber_set(&chain);
        for g in Group::elements() {
            let ga = Group::apply(g, &chain);
            assert_eq!(f_a, fiber_set(&ga), "OneH fiber mismatch for g={:?}", g);
        }
    }

    #[test]
    fn fiber_theorem1_random() {
        use rand::rngs::StdRng;
        use rand::{Rng, SeedableRng};
        let mut rng = StdRng::seed_from_u64(0xBEEF);

        for option in [JamOption::PL, JamOption::BU, JamOption::OneH] {
            let rows = all_rows(option);
            for k in [2usize, 3, 4] {
                for _ in 0..20 {
                    let chain_rows: Vec<[Token; 3]> = (0..k)
                        .map(|_| rows[rng.gen_range(0..rows.len())].clone())
                        .collect();
                    let chain = JamChain::from_rows(chain_rows, option);
                    let f_a = fiber_set(&chain);
                    for g in Group::elements() {
                        let ga = Group::apply(g, &chain);
                        assert_eq!(
                            f_a,
                            fiber_set(&ga),
                            "fiber mismatch for option={:?}, k={}, chain={}, g={:?}",
                            option,
                            k,
                            chain.raw(),
                            g
                        );
                    }
                }
            }
        }
    }
}
