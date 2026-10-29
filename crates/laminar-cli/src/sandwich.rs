//! `sandwich`: generate, export, and count JAM stacking configurations.

use std::fs;
use std::path::PathBuf;

use clap::Args;

use laminar_core::children::augment_to;
use laminar_core::JamOption;
use laminar_io::cif::to_cif;
use laminar_io::ReferenceCell;

#[derive(Args, Debug)]
pub struct SandwichArgs {
    /// JAM option: PL|BPL|BU|BBU|OneH|OneT|Janus1H|Janus1T.
    #[arg(long, value_name = "OPT")]
    pub option: Option<String>,

    /// Number of layers (k).
    #[arg(short = 'n', long, value_name = "N")]
    pub layers: Option<usize>,

    /// Count only, don't generate (no `-e` required).
    #[arg(short = 'q', long)]
    pub count: bool,

    /// Export format: stdout|cif|jam.
    #[arg(short = 'e', long, value_name = "FORMAT")]
    pub export: Option<String>,

    /// Output directory (required for -e cif and -e jam).
    #[arg(long, value_name = "DIR")]
    pub output: Option<PathBuf>,

    /// ReferenceCell TOML file (required for -e cif).
    #[arg(long, value_name = "PATH")]
    pub reference_cell: Option<PathBuf>,

    /// List available options and exit.
    #[arg(long)]
    pub list_options: bool,
}

pub fn run(args: SandwichArgs) -> Result<(), Box<dyn std::error::Error>> {
    if args.list_options {
        print_options();
        return Ok(());
    }

    let option_str = args
        .option
        .as_deref()
        .ok_or("--option is required (use --list-options to see available options)")?;
    let option = parse_option(option_str)?;
    let n = args.layers.ok_or("--layers is required")?;
    if n == 0 {
        return Err("--layers must be >= 1".into());
    }

    // Count-only mode.
    if args.count {
        let count = augment_to(n, option).len();
        println!("{}", count);
        return Ok(());
    }

    // Generation mode: need an export format.
    let export = args
        .export
        .as_deref()
        .ok_or("--export is required (use -e stdout to print, or -q to count)")?;

    let chains = augment_to(n, option);
    eprintln!(
        "[laminar] option={:?}, n={}, classes={}",
        option,
        n,
        chains.len()
    );

    match export {
        "stdout" => {
            for c in &chains {
                println!("{}", c.raw());
            }
        }
        "jam" => {
            let output = args
                .output
                .as_ref()
                .ok_or("--output is required for -e jam")?;
            fs::create_dir_all(output)?;
            let path = output.join(format!("n{}_{:?}.jam", n, option));
            let content: Vec<String> = chains.iter().map(|c| c.raw()).collect();
            fs::write(&path, content.join("\n"))?;
            eprintln!("[laminar] wrote {}", path.display());
        }
        "cif" => {
            let output = args
                .output
                .as_ref()
                .ok_or("--output is required for -e cif")?;
            let cell_path = args
                .reference_cell
                .as_ref()
                .ok_or("--reference-cell is required for -e cif")?;
            let cell = ReferenceCell::from_toml(cell_path)?;
            fs::create_dir_all(output)?;
            let mut written = 0;
            for (i, c) in chains.iter().enumerate() {
                let cif = to_cif(c, &cell, "P 1")?;
                let path = output.join(format!("class_{:06}.cif", i));
                fs::write(&path, cif)?;
                written += 1;
            }
            eprintln!(
                "[laminar] wrote {} CIF files to {}",
                written,
                output.display()
            );
        }
        other => {
            return Err(format!(
                "unsupported export format: {} (available: stdout, cif, jam)",
                other
            )
            .into());
        }
    }
    Ok(())
}

fn parse_option(s: &str) -> Result<JamOption, Box<dyn std::error::Error>> {
    match s {
        "PL" => Ok(JamOption::PL),
        "BPL" => Ok(JamOption::BPL),
        "BU" => Ok(JamOption::BU),
        "BBU" => Ok(JamOption::BBU),
        "OneH" => Ok(JamOption::OneH),
        "OneT" => Ok(JamOption::OneT),
        "Janus1H" => Ok(JamOption::Janus1H),
        "Janus1T" => Ok(JamOption::Janus1T),
        _ => Err(format!(
            "unknown option: {} (available: PL, BPL, BU, BBU, OneH, OneT, Janus1H, Janus1T)",
            s
        )
        .into()),
    }
}

fn print_options() {
    println!("Available JAM options:");
    println!("  PL       (1) monoelemental planar");
    println!("  BPL      (2) bielemental planar");
    println!("  BU       (3) buckled monoelemental");
    println!("  BBU      (4) buckled bielemental");
    println!("  OneH     (5) 1H TMDC (e.g., MoS2)");
    println!("  OneT     (6) 1T TMDC");
    println!("  Janus1H  (7) Janus 1H TMDC (e.g., MoSSe)");
    println!("  Janus1T  (8) Janus 1T TMDC");
}
