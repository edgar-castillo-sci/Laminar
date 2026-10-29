//! Laminar CLI.
//!
//! Commands:
//! - `sandwich`: generate, export, and count JAM stacking configurations.
//! - `compare`: check equivalence between chains.
//! - `sample`: (Fase 7) MCMC sampling.
//! - `modules`: (Fase 3) list available modules.

mod compare;
mod sandwich;

use clap::{Parser, Subcommand};
use std::process::ExitCode;

/// Infrastructure for laminar stacking.
///
/// Defaults: honeycomb symmetry (G = S3 × Z/2), JAM notation.
#[derive(Parser)]
#[command(name = "laminar")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Generate, canonicalize, and export laminar stacking configurations")]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate and export JAM stacking configurations.
    Sandwich(sandwich::SandwichArgs),
    /// Compare chains for equivalence.
    Compare(compare::CompareArgs),
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = match cli.command {
        Commands::Sandwich(args) => sandwich::run(args),
        Commands::Compare(args) => compare::run(args),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("[laminar] error: {}", e);
            ExitCode::FAILURE
        }
    }
}
