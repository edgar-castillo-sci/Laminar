//! G = S3 × Z/2, order 12.

use crate::types::{JamChain, Stabilizer, Token};

#[cfg(test)]
use crate::types::JamOption;

/// A permutation of {0, 1, 2}, representing a column permutation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Perm3(pub [usize; 3]);

impl Perm3 {
    pub const IDENTITY: Perm3 = Perm3([0, 1, 2]);

    /// All 6 elements of S3.
    pub fn all() -> [Perm3; 6] {
        [
            Perm3([0, 1, 2]), // e
            Perm3([0, 2, 1]), // (1 2)
            Perm3([1, 0, 2]), // (0 1)
            Perm3([1, 2, 0]), // (0 1 2)
            Perm3([2, 0, 1]), // (0 2 1)
            Perm3([2, 1, 0]), // (0 2)
        ]
    }
}

/// An element of G = S3 × Z/2.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GroupElement {
    pub sigma: Perm3,
    pub v: bool,
}

impl GroupElement {
    pub const IDENTITY: GroupElement = GroupElement {
        sigma: Perm3::IDENTITY,
        v: false,
    };
}

/// The group G, with 12 elements.
pub struct Group;

impl Group {
    /// All 12 elements of G.
    pub fn elements() -> [GroupElement; 12] {
        let mut out = [GroupElement::IDENTITY; 12];
        let mut i = 0;
        for sigma in Perm3::all() {
            for v in [false, true] {
                out[i] = GroupElement { sigma, v };
                i += 1;
            }
        }
        out
    }

    /// Apply a group element to a chain.
    pub fn apply(g: GroupElement, chain: &JamChain) -> JamChain {
        let mut rows: Vec<[Token; 3]> = chain
            .rows()
            .iter()
            .map(|row| {
                [
                    row[g.sigma.0[0]].clone(),
                    row[g.sigma.0[1]].clone(),
                    row[g.sigma.0[2]].clone(),
                ]
            })
            .collect();

        if g.v {
            if chain.option().v_is_planar() {
                rows.reverse();
            } else {
                rows.reverse();
                for row in &mut rows {
                    for token in row {
                        *token = flip_token(token.clone());
                    }
                }
            }
        }

        JamChain::from_rows(rows, chain.option())
    }

    /// The orbit of a chain under G (12 elements, with duplicates removed).
    pub fn orbit(chain: &JamChain) -> Vec<JamChain> {
        let mut seen = std::collections::HashSet::new();
        let mut out = Vec::new();
        for g in Self::elements() {
            let c = Self::apply(g, chain);
            if seen.insert(c.raw()) {
                out.push(c);
            }
        }
        out
    }

    /// The stabilizer of a chain.
    pub fn stabilizer(chain: &JamChain) -> Stabilizer {
        let mut gens = smallvec::SmallVec::new();
        for g in Self::elements() {
            if g == GroupElement::IDENTITY {
                continue;
            }
            let c = Self::apply(g, chain);
            if c.rows() == chain.rows() {
                gens.push(g);
            }
        }
        match gens.len() {
            0 => Stabilizer::Trivial,
            1 => Stabilizer::Z2,
            2 => Stabilizer::Z3,
            _ => Stabilizer::Other(gens),
        }
    }
}

/// Flip the sign of a token under v = sign∘flip (buckled-type options).
fn flip_token(token: Token) -> Token {
    match token {
        Token::Up(x) => Token::Down(x),
        Token::Down(x) => Token::Up(x),
        Token::Fixed(x) => Token::Fixed(x),
        Token::Buckled { up, down } => Token::Buckled { up: down, down: up },
        Token::Empty => Token::Empty,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn group_has_12_elements() {
        assert_eq!(Group::elements().len(), 12);
    }

    #[test]
    fn identity_fixes_chain() {
        let chain = JamChain::parse("+j+0(+k-k)/+j+0(+k-k)", JamOption::OneH).unwrap();
        let c2 = Group::apply(GroupElement::IDENTITY, &chain);
        assert_eq!(chain.raw(), c2.raw());
    }

    #[test]
    fn flip_reverses_layers() {
        // Both rows valid for OneH (1×Empty, 1×Up, 1×Buckled).
        let chain = JamChain::parse("+j+0(+k-k)/+0+j(+k-k)", JamOption::OneH).unwrap();
        let flipped = Group::apply(
            GroupElement {
                sigma: Perm3::IDENTITY,
                v: true,
            },
            &chain,
        );
        assert_eq!(flipped.raw(), "+0+j(+k-k)/+j+0(+k-k)");
    }
}
