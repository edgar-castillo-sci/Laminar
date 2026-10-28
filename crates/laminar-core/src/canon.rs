//! κ: canonical representative (Theorem 1, *Orbits as Multisets*).

use crate::group::Group;
use crate::types::JamChain;

/// The literal canonicalization: min over all 12 elements of G, using
/// the row-major serialization for comparison.
///
/// This is the reference implementation. It is O(|G| · k) = O(k) since
/// |G| = 12 is constant, but it evaluates all 12 elements.
pub fn kappa_literal(chain: &JamChain) -> JamChain {
    Group::elements()
        .iter()
        .map(|g| Group::apply(*g, chain))
        // Compare by Token::Ord (rank), not by raw() string.
        // raw() is variable-length, so char-order != token-order.
        .min_by(|a, b| a.rows().cmp(b.rows()))
        .expect("G is non-empty")
}

/// The fast canonicalization: O(k) with constant 2.
///
/// **Skeleton implementation:** delegates to `kappa_literal`.
/// The real O(k) algorithm (Theorem 3 of *Orbits as Multisets*)
/// will be implemented in Fase 1 using column-major serialization
/// and `sort_cols`.
pub fn kappa_fast(chain: &JamChain) -> JamChain {
    kappa_literal(chain)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::JamOption;

    #[test]
    fn literal_and_fast_agree_on_mos2() {
        let chain = JamChain::parse("+j+0(+k-k)/+j+0(+k-k)", Option::OneH).unwrap();
        assert_eq!(kappa_literal(&chain).raw(), kappa_fast(&chain).raw());
    }

    #[test]
    fn kappa_is_idempotent() {
        let chain = JamChain::parse("+j+0(+k-k)/+j+0(+k-k)", Option::OneH).unwrap();
        let k1 = kappa_literal(&chain);
        let k2 = kappa_literal(&k1);
        assert_eq!(k1.raw(), k2.raw());
    }

    #[test]
    fn kappa_is_g_invariant() {
        let chain = JamChain::parse("+j+0(+k-k)/+j+0(+k-k)", Option::OneH).unwrap();
        let k = kappa_literal(&chain);
        for g in Group::elements() {
            let c = Group::apply(g, &chain);
            assert_eq!(kappa_literal(&c).raw(), k.raw());
        }
    }
}
