//! ReferenceCell + SiteOccupancy: the bridge between JAM tokens and
//! atomic coordinates.

use std::collections::HashMap;
use std::fmt;

/// A reference cell for exporting JAM chains to external formats.
///
/// **This is a user choice, not inferred by Laminar.** The user must
/// provide `a`, `c`, `z_up`, `z_down`, and the species map.
#[derive(Clone, Debug)]
pub struct ReferenceCell {
    /// In-plane lattice parameter (Å).
    pub a: f64,
    /// Interlayer distance (Å).
    pub c: f64,
    /// z of `Up` atoms relative to the layer plane (Å).
    pub z_up: f64,
    /// z of `Down` atoms relative to the layer plane (Å).
    pub z_down: f64,
    /// JAM letter → chemical species (e.g., "j" → "Mo").
    pub species_map: HashMap<String, String>,
}

impl ReferenceCell {
    /// Validate the cell against a chain (rules V1-V5 of the spec).
    pub fn validate(&self, chain: &laminar_core::JamChain) -> Result<(), ReferenceCellError> {
        use laminar_core::Token;
        let has_down = chain
            .rows()
            .iter()
            .flatten()
            .any(|t| matches!(t, Token::Down(_)));
        let has_buckled = chain
            .rows()
            .iter()
            .flatten()
            .any(|t| matches!(t, Token::Buckled { .. }));
        let has_fixed = chain
            .rows()
            .iter()
            .flatten()
            .any(|t| matches!(t, Token::Fixed(_)));
        let has_up = chain
            .rows()
            .iter()
            .flatten()
            .any(|t| matches!(t, Token::Up(_)));

        // V1
        if has_down && self.z_down == 0.0 {
            return Err(ReferenceCellError::V1 { z_down: self.z_down });
        }
        // V2
        if has_buckled && (self.z_up == self.z_down || !(self.z_down < 0.0 && 0.0 < self.z_up)) {
            return Err(ReferenceCellError::V2 {
                z_up: self.z_up,
                z_down: self.z_down,
            });
        }
        // V3: Fixed — no requirement.
        let _ = has_fixed;
        // V4
        if has_up && !has_down && !has_buckled && self.z_up == 0.0 {
            return Err(ReferenceCellError::V4 { z_up: self.z_up });
        }
        // V5
        let has_any_atom = has_up || has_down || has_buckled || has_fixed;
        if !has_any_atom {
            eprintln!("[laminar-io] warning: chain has no atoms; V5 applies");
        }
        Ok(())
    }

    /// Look up the chemical species for a JAM letter.
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
