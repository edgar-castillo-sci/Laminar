//! `modules`: list available symmetry, notation, and export modules.

use clap::Args;

#[derive(Args, Debug)]
pub struct ModulesArgs {
    /// Emit JSON instead of human-readable text.
    #[arg(long)]
    pub json: bool,
}

pub fn run(args: ModulesArgs) -> Result<(), Box<dyn std::error::Error>> {
    if args.json {
        print_json();
    } else {
        print_text();
    }
    Ok(())
}

fn print_text() {
    println!("Available symmetry modules:");
    println!("  honeycomb    (D6, |G|=12, honeycomb layered materials)");
    println!();
    println!("Available notation modules:");
    println!("  jam          (Joining and Arrangement of Multilayers, 8 canonical options)");
    println!();
    println!("Available export formats:");
    println!("  stdout       (print to terminal)");
    println!("  jam          (JAM string)");
    println!("  cif          (Crystallographic Information File)");
}

fn print_json() {
    let json = serde_json::json!({
        "symmetry_modules": [
            {"name": "honeycomb", "group": "D6", "order": 12,
             "description": "honeycomb layered materials"}
        ],
        "notation_modules": [
            {"name": "jam", "description": "Joining and Arrangement of Multilayers",
             "options": ["PL", "BPL", "BU", "BBU", "OneH", "OneT", "Janus1H", "Janus1T"]}
        ],
        "export_formats": [
            {"name": "stdout", "description": "print to terminal"},
            {"name": "jam", "description": "JAM string"},
            {"name": "cif", "description": "Crystallographic Information File"}
        ]
    });
    println!("{}", serde_json::to_string_pretty(&json).unwrap());
}
