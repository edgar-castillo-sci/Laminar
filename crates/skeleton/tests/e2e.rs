//! Fase 0.5 end-to-end test.

use laminar_core::{canon::kappa_fast, canon::kappa_literal, JamChain, JamOption};
use laminar_io::cif::to_cif;
use laminar_io::ReferenceCell;
use std::collections::HashMap;

#[test]
fn skeleton_pipeline() {
    // OneH uses *j (Fixed) per Tabla 7.
    let chain = JamChain::parse("*j+0(+k-k)/*j+0(+k-k)", JamOption::OneH)
        .expect("parse failed");
    assert_eq!(chain.k(), 2);
    assert_eq!(kappa_literal(&chain).raw(), kappa_fast(&chain).raw());

    let mut species_map = HashMap::new();
    species_map.insert("j".into(), "Mo".into());
    species_map.insert("k".into(), "S".into());
    let cell = ReferenceCell {
        a: 3.16,
        c: 12.30,
        z_up: 1.56,
        z_down: -1.56,
        species_map,
    };

    let cif = to_cif(&chain, &cell, "P 1").expect("to_cif failed");
    assert!(cif.contains("Mo"));
    assert!(cif.contains("S"));
    assert_eq!(cif.lines().filter(|l| l.contains(" Mo ")).count(), 2);
    assert_eq!(cif.lines().filter(|l| l.contains(" S ")).count(), 4);
}

#[test]
fn kappa_parity_for_k2_to_k4() {
    let cases = [
        ("*j+0(+k-k)/*j+0(+k-k)", JamOption::OneH),
        ("*j+0(+k-k)/*j+0(+k-k)/*j+0(+k-k)", JamOption::OneH),
        ("*j+0(+k-k)/*j+0(+k-k)/*j+0(+k-k)/*j+0(+k-k)", JamOption::OneH),
    ];
    for (s, opt) in cases {
        let chain = JamChain::parse(s, opt).expect("parse failed");
        assert_eq!(
            kappa_literal(&chain).raw(),
            kappa_fast(&chain).raw(),
            "mismatch for {}",
            s
        );
    }
}
