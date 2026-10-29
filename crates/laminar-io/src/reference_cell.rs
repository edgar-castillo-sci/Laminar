//! ReferenceCell + SiteOccupancy: the bridge between JAM tokens and
//! atomic coordinates.
//!
//! Validation rules (spec §4.3):
//! - V1: chain has `Down` → z_down != 0.
//! - V2: chain has `Buckled` → z_up != z_down AND z_down < 0 < z_up.
//! - V2a: `Buckled` with up == down (symmetric pair) → |z_up| == |z_down|.
//! - V3: chain has `Fixed` → no z requirement (atom at z = 0).
//! - V4: chain has only `Up` (no Down/Buckled) → z_up != 0.
//! - V5: chain has no atoms → warning.

use std::collections::HashMap;
use std::fmt;

use laminar_core::{JamChain, Token};

/// A reference cell for exporting JAM chains.
///
/// **User choice, not inferred by Laminar.** The user must provide
/// `a`, `c`, `z_up`, `z_down`, and the species map.
#[derive(Clone, Debug)]
pub struct ReferenceCell {
    pub a: f64,
    pub c: f64,
    pub z_up: f64,
    pub z_down: f64,
    pub species_map: HashMap<String, String>,
}

impl ReferenceCell {
    /// Validate the cell against a chain (rules V1-V5).
    pub fn validate(&self, chain: &JamChain) -> Result<(), ReferenceCellError> {
        let has_down = chain.rows().iter().flatten().any(|t| matches!(t, Token::Down(_)));
        let has_buckled = chain.rows().iter().flatten().any(|t| matches!(t, Token::Buckled { .. }));
        let has_fixed = chain.rows().iter().flatten().any(|t| matches!(t, Token::Fixed(_)));
        let has_up = chain.rows().iter().flatten().any(|t| matches!(t, Token::Up(_)));

        // V1
        if has_down && self.z_down == 0.0 {
            return Err(ReferenceCellError::V1 { z_down: self.z_down });
        }
        // V2
        if has_buckled && (self.z_up == self.z_down || !(self.z_down < 0.0 && 0.0 < self.z_up)) {
            return Err(ReferenceCellError::V2 { z_up: self.z_up, z_down: self.z_down });
        }
        // V2a: symmetric buckled pairs must have |z_up| == |z_down|.
        if has_buckled {
            for row in chain.rows() {
                for token in row {
                    if let Token::Buckled { up, down } = token {
                        if up == down && (self.z_up.abs() - self.z_down.abs()).abs() > 1e-9 {
                            return Err(ReferenceCellError::V2a {
                                z_up: self.z_up,
                                z_down: self.z_down,
                                up: up.clone(),
                                down: down.clone(),
                            });
                        }
                    }
                }
            }
        }
        // V3: Fixed → no requirement.
        let _ = has_fixed;
        // V4
        if has_up && !has_down && !has_buckled && self.z_up == 0.0 {
            return Err(ReferenceCellError::V4 { z_up: self.z_up });
        }
        // V5: warning only.
        let has_any_atom = has_up || has_down || has_buckled || has_fixed;
        if !has_any_atom {
            eprintln!("[laminar-io] warning: chain has no atoms; V5 applies");
        }
        Ok(())
    }

    pub fn species(&self, letter: &str) -> Result<&str, ReferenceCellError> {
        self.species_map
            .get(letter)
            .map(|s| s.as_str())
            .ok_or_else(|| ReferenceCellError::MissingSpecies(letter.to_string()))
    }
}

#[derive(Debug)]
pub enum ReferenceCellError {
    V1 { z_down: f64 },
    V2 { z_up: f64, z_down: f64 },
    V2a { z_up: f64, z_down: f64, up: String, down: String },
    V4 { z_up: f64 },
    MissingSpecies(String),
}

impl fmt::Display for ReferenceCellError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReferenceCellError::V1 { z_down } => write!(
                f,
                "V1: chain contains Down token but z_down = {} (must be != 0)",
                z_down
            ),
            ReferenceCellError::V2 { z_up, z_down } => write!(
                f,
                "V2: chain contains Buckled token but z_up = {}, z_down = {} \
                 (must have z_down < 0 < z_up and z_up != z_down)",
                z_up, z_down
            ),
            ReferenceCellError::V2a { z_up, z_down, up, down } => write!(
                f,
                "V2a: buckled pair (+{}-{}) is symmetric (up == down) but \
                 |z_up| = {} != |z_down| = {} (must be equal)",
                up, down, z_up.abs(), z_down.abs()
            ),
            ReferenceCellError::V4 { z_up } => write!(
                f,
                "V4: chain contains Up token but z_up = {} (must be != 0)",
                z_up
            ),
            ReferenceCellError::MissingSpecies(s) => {
                write!(f, "species_map missing letter '{}'", s)
            }
        }
    }
}

impl std::error::Error for ReferenceCellError {}

#[cfg(test)]
mod tests {
    use super::*;
    use laminar_core::JamOption;

    fn cell(z_up: f64, z_down: f64) -> ReferenceCell {
        let mut species_map = HashMap::new();
        species_map.insert("j".into(), "X".into());
        species_map.insert("k".into(), "Y".into());
        ReferenceCell { a: 3.0, c: 10.0, z_up, z_down, species_map }
    }

    // ---------- V1: Down with z_down == 0 ----------

    #[test]
    fn v1_down_with_zero_z_down_fails() {
        let chain = JamChain::parse("+0+j-j", JamOption::BU).unwrap();
        let c = cell(1.5, 0.0);
        assert!(matches!(c.validate(&chain), Err(ReferenceCellError::V1 { .. })));
    }

    #[test]
    fn v1_down_with_nonzero_z_down_passes() {
        let chain = JamChain::parse("+0+j-j", JamOption::BU).unwrap();
        let c = cell(1.5, -1.5);
        assert!(c.validate(&chain).is_ok());
    }

    // ---------- V2: Buckled with z_up == z_down, or z_down >= 0 ----------

    #[test]
    fn v2_buckled_with_equal_z_fails() {
        let chain = JamChain::parse("+0*j(+k-k)", JamOption::OneH).unwrap();
        let c = cell(1.5, 1.5);
        assert!(matches!(c.validate(&chain), Err(ReferenceCellError::V2 { .. })));
    }

    #[test]
    fn v2_buckled_with_nonneg_z_down_fails() {
        let chain = JamChain::parse("+0*j(+k-k)", JamOption::OneH).unwrap();
        let c = cell(1.5, 0.5);
        assert!(matches!(c.validate(&chain), Err(ReferenceCellError::V2 { .. })));
    }

    #[test]
    fn v2_buckled_with_valid_z_passes() {
        let chain = JamChain::parse("+0*j(+k-k)", JamOption::OneH).unwrap();
        let c = cell(1.5, -1.5);
        assert!(c.validate(&chain).is_ok());
    }

    // ---------- V2a: symmetric buckled pair needs |z_up| == |z_down| ----------

    #[test]
    fn v2a_symmetric_pair_with_unequal_abs_fails() {
        let chain = JamChain::parse("+0*j(+k-k)", JamOption::OneH).unwrap();
        // up == down ("k" == "k"), but |z_up| != |z_down|.
        let c = cell(2.0, -1.0);
        assert!(matches!(c.validate(&chain), Err(ReferenceCellError::V2a { .. })));
    }

    #[test]
    fn v2a_symmetric_pair_with_equal_abs_passes() {
        let chain = JamChain::parse("+0*j(+k-k)", JamOption::OneH).unwrap();
        let c = cell(1.5, -1.5);
        assert!(c.validate(&chain).is_ok());
    }

    // ---------- V4: Up-only chain needs z_up != 0 ----------

    #[test]
    fn v4_up_only_with_zero_z_up_fails() {
        let chain = JamChain::parse("+0+j+j", JamOption::PL).unwrap();
        let c = cell(0.0, 0.0);
        assert!(matches!(c.validate(&chain), Err(ReferenceCellError::V4 { .. })));
    }

    #[test]
    fn v4_up_only_with_nonzero_z_up_passes() {
        let chain = JamChain::parse("+0+j+j", JamOption::PL).unwrap();
        let c = cell(1.5, 0.0); // z_down irrelevant for PL
        assert!(c.validate(&chain).is_ok());
    }

    // ---------- Missing species ----------

    #[test]
    fn missing_species_errors() {
        let c = cell(1.5, -1.5);
        assert!(matches!(
            c.species("z"),
            Err(ReferenceCellError::MissingSpecies(_))
        ));
    }
}
