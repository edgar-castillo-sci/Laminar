//! `sandwich`: generate, export, and count JAM stacking configurations.

use std::fs;
use std::path::PathBuf;

use clap::Args;
use serde::Serialize;

use laminar_core::children::augment_to;
use laminar_core::JamOption;
use laminar_io::cif::to_cif;
use laminar_io::ReferenceCell;

use crate::common::{metadata, Metadata};
use crate::config::Config;

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

    /// Emit JSON to stdout instead of human-readable text.
    #[arg(long)]
    pub json: bool,

    /// Emit extra diagnostics to stderr.
    #[arg(short = 'v', long)]
    pub verbose: bool,
}

#[derive(Serialize)]
struct SandwichOutput {
    command: &'static str,
    option: String,
    n: usize,
    mode: &'static str,
    count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    chains: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    files: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    output_dir: Option<String>,
    metadata: Metadata,
}

pub fn run(
    args: SandwichArgs,
    cfg: &Config,
) -> Result<(), Box<dyn std::error::Error>> {
    if args.list_options {
        print_options();
        return Ok(());
    }

    // Resolve option: CLI > config > error.
    let option_str_owned = args
        .option
        .as_deref()
        .or(cfg.defaults.option.as_deref())
        .map(|s| s.to_string());
    let option_str = option_str_owned
        .as_deref()
        .ok_or("--option is required (use --list-options to see available options)")?;
    let option = parse_option(option_str)?;

    // Resolve layers: CLI > config > error.
    let n = args
        .layers
        .or(cfg.defaults.layers)
        .ok_or("--layers is required")?;
    if n == 0 {
        return Err("--layers must be >= 1".into());
    }
    let opt_str = format!("{:?}", option);

    // Count-only mode.
    if args.count {
        let count = augment_to(n, option).len();
        if args.json {
            let out = SandwichOutput {
                command: "sandwich",
                option: opt_str,
                n,
                mode: "count",
                count,
                chains: None,
                files: None,
                output_dir: None,
                metadata: metadata(),
            };
            println!("{}", serde_json::to_string(&out)?);
        } else {
            println!("{}", count);
        }
        return Ok(());
    }

    // Generation mode: resolve export format (CLI > config > error).
    let export_owned = args
        .export
        .as_deref()
        .or(cfg.defaults.export_format.as_deref())
        .map(|s| s.to_string());
    let export = export_owned
        .as_deref()
        .ok_or("--export is required (use -e stdout to print, or -q to count)")?;

    let chains = augment_to(n, option);
    log_generation(&args, &opt_str, n, chains.len());

    match export {
        "stdout" => {
            if args.json {
                let out = SandwichOutput {
                    command: "sandwich",
                    option: opt_str,
                    n,
                    mode: "stdout",
                    count: chains.len(),
                    chains: Some(chains.iter().map(|c| c.raw()).collect()),
                    files: None,
                    output_dir: None,
                    metadata: metadata(),
                };
                println!("{}", serde_json::to_string(&out)?);
            } else {
                for c in &chains {
                    println!("{}", c.raw());
                }
            }
        }
        "jam" => {
            let output = args
                .output
                .as_ref()
                .or(cfg.defaults.output.as_ref())
                .ok_or("--output is required for -e jam")?
                .clone();
            fs::create_dir_all(&output)?;
            let path = output.join(format!("n{}_{}.jam", n, opt_str));
            let content: Vec<String> = chains.iter().map(|c| c.raw()).collect();
            fs::write(&path, content.join("\n"))?;
            if args.verbose || !args.json {
                eprintln!("[laminar] wrote {}", path.display());
            }
            if args.json {
                let out = SandwichOutput {
                    command: "sandwich",
                    option: opt_str,
                    n,
                    mode: "jam",
                    count: chains.len(),
                    chains: None,
                    files: Some(vec![path.display().to_string()]),
                    output_dir: Some(output.display().to_string()),
                    metadata: metadata(),
                };
                println!("{}", serde_json::to_string(&out)?);
            }
        }
        "cif" => {
            let output = args
                .output
                .as_ref()
                .or(cfg.defaults.output.as_ref())
                .ok_or("--output is required for -e cif")?
                .clone();
            let cell_path = args
                .reference_cell
                .as_ref()
                .or(cfg.defaults.reference_cell.as_ref())
                .ok_or("--reference-cell is required for -e cif")?
                .clone();
            let cell = ReferenceCell::from_toml(&cell_path)?;
            fs::create_dir_all(&output)?;
            let mut written = Vec::new();
            for (i, c) in chains.iter().enumerate() {
                let cif = to_cif(c, &cell, "P 1")?;
                let path = output.join(format!("class_{:06}.cif", i));
                fs::write(&path, cif)?;
                written.push(path.display().to_string());
            }
            if args.verbose || !args.json {
                eprintln!(
                    "[laminar] wrote {} CIF files to {}",
                    written.len(),
                    output.display()
                );
            }
            if args.json {
                let out = SandwichOutput {
                    command: "sandwich",
                    option: opt_str,
                    n,
                    mode: "cif",
                    count: chains.len(),
                    chains: None,
                    files: Some(written),
                    output_dir: Some(output.display().to_string()),
                    metadata: metadata(),
                };
                println!("{}", serde_json::to_string(&out)?);
            }
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

fn log_generation(args: &SandwichArgs, opt_str: &str, n: usize, classes: usize) {
    if args.verbose {
        eprintln!(
            "[laminar] symmetry=honeycomb notation=jam version={}",
            env!("CARGO_PKG_VERSION")
        );
        eprintln!("[laminar] option={} n={} classes={}", opt_str, n, classes);
    } else if !args.json {
        eprintln!("[laminar] option={} n={} classes={}", opt_str, n, classes);
    }
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
