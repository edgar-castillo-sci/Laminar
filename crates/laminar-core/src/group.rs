//! G = S3 × Z/2, order 12.

use crate::types::{JamChain, JamOption, Stabilizer, Token};

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
                // v = flip: reverse layer order, no content change.
                rows.reverse();
            } else {
                // v = sign∘flip: reverse + flip sign / exchange buckled pair.
                rows.reverse();
                for row in &mut rows {
                    for token in row {
                        *token = flip_token(token.clone(), chain.option());
                    }
                }
            }
        }

        JamChain::parse(&rows_to_string(&rows), chain.option())
            .expect("Group::apply produced an invalid chain")
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
            if c.raw() == chain.raw() {
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

fn flip_token(token: Token, option: JamOption) -> Token {
    match token {
        Token::Up(x) => Token::Down(x),
        Token::Down(x) => Token::Up(x),
        Token::Fixed(x) => Token::Fixed(x),
        Token::Buckled { up, down } => Token::Buckled { up: down, down: up },
        Token::Empty => Token::Empty,
    }
    .tap(|_| {
        // For buckled options, (+x -y) maps to (-x +y); the swap above
        // implements the exchange. For planar options this branch is
        // unreachable because v_is_planar() is checked before calling.
        let _ = option;
    })
}

fn rows_to_string(rows: &[[Token; 3]]) -> String {
    rows.iter()
        .map(|row| row.iter().map(|t| t.to_jam()).collect::<String>())
        .collect::<Vec<_>>()
        .join("/")
}

trait Tap: Sized {
    fn tap<F: FnOnce(&Self)>(self, f: F) -> Self {
        f(&self);
        self
    }
}
impl<T> Tap for T {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn group_has_12_elements() {
        assert_eq!(Group::elements().len(), 12);
    }

    #[test]
    fn identity_fixes_chain() {
        let chain = JamChain::parse("+j+0(+k-k)/+j+0(+k-k)", Option::OneH).unwrap();
        let c2 = Group::apply(GroupElement::IDENTITY, &chain);
        assert_eq!(chain.raw(), c2.raw());
    }

    #[test]
    fn flip_reverses_layers() {
        let chain = JamChain::parse("+j+0(+k-k)/+k+j(+k-k)", Option::OneH).unwrap();
        let flipped = Group::apply(
            GroupElement {
                sigma: Perm3::IDENTITY,
                v: true,
            },
            &chain,
        );
        assert_eq!(flipped.raw(), "+k+j(+k-k)/+j+0(+k-k)");
    }
}
