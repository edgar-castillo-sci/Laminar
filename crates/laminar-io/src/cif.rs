//! CIF export.
//!
//! **v0.1.0:** always writes `P 1` (explicit atoms, no symmetry).
//! The `space_group` argument is reserved for v0.2.0, which will add
//! `to_cif_symmetric` (asymmetric unit + non-trivial space group).
//! Writing a non-P1 space group with explicit atoms causes ASE to
//! expand the atoms (Mo4S8 instead of Mo2S4), so we forbid it here.

use laminar_core::{JamChain, Token};

use crate::reference_cell::{ReferenceCell, ReferenceCellError};

/// Export a JAM chain to CIF format.
///
/// `space_group` is reserved for v0.2.0; it is currently ignored and
/// the output always uses `P 1` (explicit atoms, no symmetry).
pub fn to_cif(
    chain: &JamChain,
    cell: &ReferenceCell,
    _space_group: &str,
) -> Result<String, ReferenceCellError> {
    cell.validate(chain)?;

    let mut s = String::new();
    s.push_str("data_jam_export\n");
    s.push_str(&format!("_cell_length_a     {:.4}\n", cell.a));
    s.push_str(&format!("_cell_length_b     {:.4}\n", cell.a));
    s.push_str(&format!("_cell_length_c     {:.4}\n", cell.c));
    s.push_str("_cell_angle_alpha  90.0000\n");
    s.push_str("_cell_angle_beta   90.0000\n");
    s.push_str("_cell_angle_gamma 120.0000\n");
    s.push_str("_symmetry_space_group_name_H-M 'P 1'\n\n");

    s.push_str("loop_\n");
    s.push_str("_atom_site_label\n");
    s.push_str("_atom_site_type_symbol\n");
    s.push_str("_atom_site_fract_x\n");
    s.push_str("_atom_site_fract_y\n");
    s.push_str("_atom_site_fract_z\n");

    let k = chain.k();
    let mut atom_idx = 0;
    for (i, row) in chain.rows().iter().enumerate() {
        let z_layer_frac = (2.0 * i as f64 + 1.0) / (2.0 * k as f64);
        for (j, token) in row.iter().enumerate() {
            let (wx, wy) = wyckoff_position(j);
            match token {
                Token::Empty => {}
                Token::Up(letter) => {
                    let species = cell.species(letter)?;
                    atom_idx += 1;
                    let z = z_layer_frac + cell.z_up / cell.c;
                    s.push_str(&format!(
                        "{}{} {} {:.6} {:.6} {:.6}\n",
                        species, atom_idx, species, wx, wy, z
                    ));
                }
                Token::Down(letter) => {
                    let species = cell.species(letter)?;
                    atom_idx += 1;
                    let z = z_layer_frac + cell.z_down / cell.c;
                    s.push_str(&format!(
                        "{}{} {} {:.6} {:.6} {:.6}\n",
                        species, atom_idx, species, wx, wy, z
                    ));
                }
                Token::Fixed(letter) => {
                    let species = cell.species(letter)?;
                    atom_idx += 1;
                    let z = z_layer_frac;
                    s.push_str(&format!(
                        "{}{} {} {:.6} {:.6} {:.6}\n",
                        species, atom_idx, species, wx, wy, z
                    ));
                }
                Token::Buckled { up, down } => {
                    let species_up = cell.species(up)?;
                    let species_down = cell.species(down)?;
                    let z_up = z_layer_frac + cell.z_up / cell.c;
                    let z_down = z_layer_frac + cell.z_down / cell.c;
                    atom_idx += 1;
                    s.push_str(&format!(
                        "{}{} {} {:.6} {:.6} {:.6}\n",
                        species_up, atom_idx, species_up, wx, wy, z_up
                    ));
                    atom_idx += 1;
                    s.push_str(&format!(
                        "{}{} {} {:.6} {:.6} {:.6}\n",
                        species_down, atom_idx, species_down, wx, wy, z_down
                    ));
                }
            }
        }
    }
    Ok(s)
}

/// Wyckoff position (x, y) for column index j ∈ {0, 1, 2} (1a, 1b, 1c).
fn wyckoff_position(j: usize) -> (f64, f64) {
    match j {
        0 => (0.0, 0.0),
        1 => (1.0 / 3.0, 2.0 / 3.0),
        2 => (2.0 / 3.0, 1.0 / 3.0),
        _ => unreachable!("column index out of range"),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use laminar_core::{JamChain, JamOption};
    use std::collections::HashMap;

    fn mos2_cell() -> ReferenceCell {
        let mut species_map = HashMap::new();
        species_map.insert("j".into(), "Mo".into());
        species_map.insert("k".into(), "S".into());
        ReferenceCell { a: 3.16, c: 12.30, z_up: 1.56, z_down: -1.56, species_map }
    }

    fn pl_cell() -> ReferenceCell {
        // PL rows have `Up` tokens (+j) that are physically in-plane (z = 0).
        // V4 of the spec requires z_up != 0 when the chain has `Up` and no
        // `Down`/`Buckled`. We use a tiny z_up (0.01 Å) to satisfy V4 while
        // keeping the physical intent (nearly in-plane). See D8 in CHANGELOG.
        let mut species_map = HashMap::new();
        species_map.insert("j".into(), "C".into());
        ReferenceCell { a: 2.46, c: 6.70, z_up: 0.01, z_down: 0.0, species_map }
    }

    fn bu_cell() -> ReferenceCell {
        let mut species_map = HashMap::new();
        species_map.insert("j".into(), "P".into());
        ReferenceCell { a: 3.30, c: 10.50, z_up: 0.50, z_down: -0.50, species_map }
    }

    fn count_atoms(cif: &str, species: &str) -> usize {
        cif.lines()
            .filter(|l| l.split_whitespace().nth(1) == Some(species))
            .count()
    }

    #[test]
    fn cif_oneh_mos2_2h() {
        // MoS2 1H, 2 layers: 2 Mo (1 per layer) + 4 S (2 per layer, buckled).
        let chain = JamChain::parse("+j+0(+k-k)/+j+0(+k-k)", JamOption::OneH).unwrap();
        let cif = to_cif(&chain, &mos2_cell(), "P 1").unwrap();
        assert!(cif.contains("_cell_length_a"));
        assert!(cif.contains("_symmetry_space_group_name_H-M 'P 1'"));
        assert_eq!(count_atoms(&cif, "Mo"), 2);
        assert_eq!(count_atoms(&cif, "S"), 4);
        assert_eq!(cif.lines().filter(|l| l.contains(" Mo ")).count(), 2);
    }

    #[test]
    fn cif_pl_monoelemental() {
        // PL: 2 layers, 2 C per layer = 4 C.
        let chain = JamChain::parse("+j+0+j/+j+0+j", JamOption::PL).unwrap();
        let cif = to_cif(&chain, &pl_cell(), "P 1").unwrap();
        assert_eq!(count_atoms(&cif, "C"), 4);
        assert_eq!(count_atoms(&cif, "Mo"), 0);
    }

    #[test]
    fn cif_bu_phosphorene_like() {
        // BU: 2 layers, 1 P up + 1 P down per layer = 4 P.
        let chain = JamChain::parse("+0+j-j/+0+j-j", JamOption::BU).unwrap();
        let cif = to_cif(&chain, &bu_cell(), "P 1").unwrap();
        assert_eq!(count_atoms(&cif, "P"), 4);
        // 2 P at z = z_layer + z_up/c, 2 at z = z_layer + z_down/c.
        // Verify that z values are distinct (buckled).
        let zs: Vec<f64> = cif
            .lines()
            .filter(|l| l.split_whitespace().nth(1) == Some("P"))
            .filter_map(|l| l.split_whitespace().nth(4))
            .filter_map(|s| s.parse::<f64>().ok())
            .collect();
        let unique: std::collections::HashSet<_> = zs.iter().map(|z| (z * 1000.0).round() as i64).collect();
        assert!(unique.len() >= 2, "expected buckled z values, got {:?}", zs);
    }

    #[test]
    fn cif_rejects_invalid_cell() {
        let chain = JamChain::parse("+j+0(+k-k)/+j+0(+k-k)", JamOption::OneH).unwrap();
        let mut bad_cell = mos2_cell();
        bad_cell.z_down = 0.0; // V2 should trigger
        assert!(to_cif(&chain, &bad_cell, "P 1").is_err());
    }
}
