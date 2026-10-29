//! Shared types for CLI output (JSON metadata).

use serde::Serialize;

#[derive(Serialize)]
pub struct Metadata {
    pub laminar_version: &'static str,
    pub symmetry: &'static str,
    pub notation: &'static str,
}

pub fn metadata() -> Metadata {
    Metadata {
        laminar_version: env!("CARGO_PKG_VERSION"),
        symmetry: "honeycomb",
        notation: "jam",
    }
}
