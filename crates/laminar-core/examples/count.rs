//! Count canonical classes at level n, per option.
//!
//! Usage:
//!   cargo run --example count -- <option> <n>
//!   cargo run --example count -- PL 8
//!
//! Options: PL, BU, OneH.

use laminar_core::children::augment_to;
use laminar_core::JamOption;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("usage: count <option> <n>");
        eprintln!("  option: PL | BU | OneH");
        eprintln!("  n:      number of layers (>= 1)");
        std::process::exit(1);
    }
    let option = match args[1].as_str() {
        "PL" => JamOption::PL,
        "BPL" => JamOption::BPL,
        "BU" => JamOption::BU,
        "BBU" => JamOption::BBU,
        "OneH" => JamOption::OneH,
        other => {
            eprintln!("unknown option: {}", other);
            std::process::exit(1);
        }
    };
    let n: usize = args[2].parse().expect("n must be an integer");
    let classes = augment_to(n, option);
    println!("{}", classes.len());
}
