//! Fase 0.5: walking skeleton.
//!
//! Validates the full pipeline (types → group → canon → io → CIF)
//! on a hardcoded MoS2 1H bilayer. No CLI, no generation, no other options.
//!
//! Stop criterion:
//! 1. `kappa_literal == kappa_fast` for the hardcoded chain.
//! 2. CIF is written to `out/MoS2_1H.cif`.
//! 3. pymatgen and ASE can read the CIF.
//! 4. The CIF has the correct stoichiometry (2 Mo + 4 S).

use std::collections::HashMap;
use std::fs;

use laminar_core::{canon::kappa_fast, canon::kappa_literal, JamChain, JamOption};
use laminar_io::cif::to_cif;
use laminar_io::ReferenceCell;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Hardcoded MoS2 1H bilayer.
    //    Layer 0: Mo at 1a (+j), empty at 1b (+0), S buckled at 1c (+k-k).
    //    Layer 1: same (2H stacking).
    let chain = JamChain::parse("+j+0(+k-k)/+j+0(+k-k)", JamOption::OneH)?;
    println!("[skeleton] parsed chain: {}", chain.raw());
    println!("[skeleton] k = {}", chain.k());

    // 2. Parity: kappa_literal == kappa_fast.
    let k_lit = kappa_literal(&chain);
    let k_fast = kappa_fast(&chain);
    assert_eq!(
        k_lit.raw(),
        k_fast.raw(),
        "kappa_literal != kappa_fast"
    );
    println!("[skeleton] kappa parity OK: {}", k_fast.raw());

    // 3. ReferenceCell for MoS2 1H.
    let mut species_map = HashMap::new();
    species_map.insert("j".to_string(), "Mo".to_string());
    species_map.insert("k".to_string(), "S".to_string());
    let cell = ReferenceCell {
        a: 3.16,
        c: 12.30,
        z_up: 1.56,
        z_down: -1.56,
        species_map,
    };

    // 4. CIF.
    let cif = to_cif(&chain, &cell, "P 1")?;
    fs::create_dir_all("out")?;
    let path = "out/MoS2_1H.cif";
    fs::write(path, &cif)?;
    println!("[skeleton] CIF written to {}", path);

    // 5. Stoichiometry check (internal, without external deps).
    let n_mo = cif.lines().filter(|l| l.contains(" Mo ")).count();
    let n_s = cif.lines().filter(|l| l.contains(" S ")).count();
    println!("[skeleton] stoichiometry: {} Mo, {} S", n_mo, n_s);
    assert_eq!(n_mo, 2, "expected 2 Mo atoms");
    assert_eq!(n_s, 4, "expected 4 S atoms");

    // 6. External validation (print commands for the user).
    println!();
    println!("[skeleton] validate with:");
    println!(
        "  python3 -c 'from pymatgen.core import Structure; \
         s = Structure.from_file(\"{}\"); print(s.composition)'",
        path
    );
    println!(
        "  python3 -c 'from ase.io import read; \
         a = read(\"{}\"); print(a.get_chemical_formula())'",
        path
    );

    println!();
    println!("[skeleton] Fase 0.5 PASSED");
    Ok(())
}
