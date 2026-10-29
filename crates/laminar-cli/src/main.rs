//! Laminar CLI.

mod common;
mod compare;
mod config;
mod modules;
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
    /// Ignore config files (`./.laminar.conf` and `~/.config/laminar/laminar.conf`).
    #[arg(long, global = true)]
    pub no_config: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate and export JAM stacking configurations.
    Sandwich(sandwich::SandwichArgs),
    /// Compare chains for equivalence.
    Compare(compare::CompareArgs),
    /// List available modules (symmetry, notation, export formats).
    Modules(modules::ModulesArgs),
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    let cfg = if cli.no_config {
        config::Config::default()
    } else {
        config::Config::load()
    };

    let result = match cli.command {
        Commands::Sandwich(args) => sandwich::run(args, &cfg),
        Commands::Compare(args) => compare::run(args, &cfg),
        Commands::Modules(args) => modules::run(args),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("[laminar] error: {}", e);
            ExitCode::FAILURE
        }
    }
}
