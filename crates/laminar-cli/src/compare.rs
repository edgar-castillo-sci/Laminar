//! `compare`: check equivalence between JAM chains.

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use clap::Args;
use serde::Serialize;

use laminar_core::canon::kappa_fast;
use laminar_core::types::JamChain;
use laminar_core::JamOption;

use crate::common::{metadata, Metadata};
use crate::config::Config;

#[derive(Args, Debug)]
pub struct CompareArgs {
    /// Chains to compare (positional).
    #[arg(value_name = "CHAIN")]
    pub chains: Vec<String>,

    /// Read chains from a file (one per line; `#` comments ignored).
    /// Can be repeated.
    #[arg(long = "file", value_name = "PATH")]
    pub files: Vec<PathBuf>,

    /// JAM option: PL|BPL|BU|BBU|OneH|OneT|Janus1H|Janus1T.
    /// If not given, uses config default, then infers from the chains.
    #[arg(long, value_name = "OPT")]
    pub option: Option<String>,

    /// Emit JSON to stdout.
    #[arg(long)]
    pub json: bool,

    /// Print only `true` or `false` (one line). Exits 1 if not equivalent.
    /// Useful for shell scripting: `if laminar compare A B -q; then ...`
    #[arg(short = 'q', long)]
    pub quiet: bool,

    /// Emit extra diagnostics to stderr.
    #[arg(short = 'v', long)]
    pub verbose: bool,
}

#[derive(Serialize)]
struct CompareOutput {
    command: &'static str,
    option: String,
    equivalent: bool,
    canonical_keys: BTreeMap<String, String>,
    orbits: BTreeMap<String, Vec<String>>,
    metadata: Metadata,
}

pub fn run(
    args: CompareArgs,
    cfg: &Config,
) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Collect all raw chains (positional + files).
    let mut raw_chains: Vec<String> = args.chains.clone();
    for path in &args.files {
        let content = fs::read_to_string(path).map_err(|e| {
            format!("failed to read chains from '{}': {}", path.display(), e)
        })?;
        for line in content.lines() {
            let line = line.trim();
            if !line.is_empty() && !line.starts_with('#') {
                raw_chains.push(line.to_string());
            }
        }
    }

    if raw_chains.is_empty() {
        return Err("no chains to compare (pass positional chains or --file)".into());
    }

    // 2. Determine option: CLI > config > infer.
    let option = match args
        .option
        .as_deref()
        .or(cfg.defaults.option.as_deref())
    {
        Some(s) => parse_option(s)?,
        None => infer_option(&raw_chains)?,
    };
    let opt_str = format!("{:?}", option);

    // 3. Parse all chains.
    let mut parsed: Vec<JamChain> = Vec::with_capacity(raw_chains.len());
    for (i, raw) in raw_chains.iter().enumerate() {
        match JamChain::parse(raw, option) {
            Ok(c) => parsed.push(c),
            Err(e) => {
                return Err(format!("chain {} ({}): {}", i, raw, e).into());
            }
        }
    }

    // 4. Canonicalize.
    let canonical_keys: BTreeMap<String, String> = raw_chains
        .iter()
        .zip(&parsed)
        .map(|(raw, chain)| (raw.clone(), kappa_fast(chain).raw()))
        .collect();

    let mut orbits: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (raw, canon) in &canonical_keys {
        orbits.entry(canon.clone()).or_default().push(raw.clone());
    }
    let equivalent = orbits.len() == 1;

    if args.verbose {
        eprintln!(
            "[laminar] symmetry=honeycomb notation=jam version={}",
            env!("CARGO_PKG_VERSION")
        );
        eprintln!(
            "[laminar] option={} chains={} orbits={}",
            opt_str,
            raw_chains.len(),
            orbits.len()
        );
    }

    // 5. Report.
    if args.quiet {
        println!("{}", equivalent);
        if !equivalent {
            std::process::exit(1);
        }
    } else if args.json {
        let out = CompareOutput {
            command: "compare",
            option: opt_str,
            equivalent,
            canonical_keys,
            orbits,
            metadata: metadata(),
        };
        println!("{}", serde_json::to_string(&out)?);
    } else {
        println!("[compare] option={}", opt_str);
        println!("equivalent: {}", equivalent);
        println!("canonical_keys:");
        for (raw, canon) in &canonical_keys {
            println!("  {}  →  {}", raw, canon);
        }
        println!("orbits:");
        for (canon, raws) in &orbits {
            println!("  {}: {} chain(s)", canon, raws.len());
            for r in raws {
                println!("    - {}", r);
            }
        }
    }
    Ok(())
}

/// Infer the JAM option from the chains, trying all 8 in order.
fn infer_option(raw_chains: &[String]) -> Result<JamOption, Box<dyn std::error::Error>> {
    let all_options = [
        JamOption::PL,
        JamOption::BPL,
        JamOption::BU,
        JamOption::BBU,
        JamOption::OneH,
        JamOption::OneT,
        JamOption::Janus1H,
        JamOption::Janus1T,
    ];
    for opt in all_options {
        if raw_chains.iter().all(|raw| JamChain::parse(raw, opt).is_ok()) {
            return Ok(opt);
        }
    }
    Err("could not infer option from the given chains (use --option)".into())
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
