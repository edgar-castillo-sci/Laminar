//! `compare`: check equivalence between JAM chains.
//!
//! Usage:
//!   laminar compare <chain1> <chain2> ... [--option OPT]
//!   laminar compare --file A.txt --file B.txt [--option OPT]
//!
//! Output: `equivalent`, `canonical_keys`, `orbits`.
//! If `--option` is not given, the option is inferred from the first
//! chain that parses successfully.

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use clap::Args;

use laminar_core::canon::kappa_fast;
use laminar_core::types::{JamChain, JamOption};

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
    /// If not given, inferred from the chains.
    #[arg(long, value_name = "OPT")]
    pub option: Option<String>,
}

pub fn run(args: CompareArgs) -> Result<(), Box<dyn std::error::Error>> {
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

    // 2. Determine the option (explicit or inferred).
    let option = match args.option.as_deref() {
        Some(s) => parse_option(s)?,
        None => infer_option(&raw_chains)?,
    };

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

    // 4. Canonicalize and group by canonical key.
    let mut canon_to_chains: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (raw, chain) in raw_chains.iter().zip(&parsed) {
        let canon = kappa_fast(chain);
        canon_to_chains
            .entry(canon.raw())
            .or_default()
            .push(raw.clone());
    }

    // 5. Report.
    let all_equivalent = canon_to_chains.len() == 1;
    println!("[compare] option={:?}", option);
    println!("equivalent: {}", all_equivalent);
    println!("canonical_keys:");
    for (raw, chain) in raw_chains.iter().zip(&parsed) {
        let canon = kappa_fast(chain);
        println!("  {}  →  {}", raw, canon.raw());
    }
    println!("orbits:");
    for (canon, raws) in &canon_to_chains {
        println!("  {}: {} chain(s)", canon, raws.len());
        for r in raws {
            println!("    - {}", r);
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
