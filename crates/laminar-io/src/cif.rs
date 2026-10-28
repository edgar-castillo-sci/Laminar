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
