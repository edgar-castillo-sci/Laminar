//! Config loading for Laminar CLI.
//!
//! Hierarchy (highest to lowest priority):
//! 1. CLI flags
//! 2. Environment variables (`LAMINAR_*`)
//! 3. Local `./.laminar.conf`
//! 4. Global `$XDG_CONFIG_HOME/laminar/laminar.conf` or `~/.config/laminar/laminar.conf`
//! 5. Compiled defaults

use std::env;
use std::path::{Path, PathBuf};

use serde::Deserialize;

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct Config {
    pub modules: ModulesConfig,
    pub defaults: DefaultsConfig,
    pub behavior: BehaviorConfig,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct ModulesConfig {
    pub symmetry: Option<String>,
    pub notation: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct DefaultsConfig {
    pub option: Option<String>,
    pub output: Option<PathBuf>,
    pub export_format: Option<String>,
    pub layers: Option<usize>,
    pub reference_cell: Option<PathBuf>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct BehaviorConfig {
    pub verbosity: Option<String>, // "quiet" | "normal" | "verbose"
}

impl Config {
    /// Load the config from the default search paths.
    /// Never fails: missing files are ignored, invalid files are reported
    /// to stderr and skipped.
    pub fn load() -> Self {
        let mut cfg = Config::default();
        if let Some(path) = global_config_path() {
            cfg.merge_from_file(&path);
        }
        let local = PathBuf::from(".laminar.conf");
        if local.exists() {
            cfg.merge_from_file(&local);
        }
        cfg.merge_from_env();
        cfg
    }

    /// Load the config from a single file. Used by tests.
    #[cfg(test)]
    pub fn load_from(path: &Path) -> Self {
        let mut cfg = Config::default();
        cfg.merge_from_file(path);
        cfg
    }

    fn merge_from_file(&mut self, path: &Path) {
        let content = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                if path != Path::new(".laminar.conf") && path.exists() {
                    eprintln!("[laminar] warning: cannot read {}: {}", path.display(), e);
                }
                return;
            }
        };
        match toml::from_str::<Config>(&content) {
            Ok(partial) => self.merge_from(partial),
            Err(e) => eprintln!(
                "[laminar] warning: invalid config at {}: {}",
                path.display(),
                e
            ),
        }
    }

    fn merge_from(&mut self, other: Config) {
        if other.modules.symmetry.is_some() {
            self.modules.symmetry = other.modules.symmetry;
        }
        if other.modules.notation.is_some() {
            self.modules.notation = other.modules.notation;
        }
        if other.defaults.option.is_some() {
            self.defaults.option = other.defaults.option;
        }
        if other.defaults.output.is_some() {
            self.defaults.output = other.defaults.output;
        }
        if other.defaults.export_format.is_some() {
            self.defaults.export_format = other.defaults.export_format;
        }
        if other.defaults.layers.is_some() {
            self.defaults.layers = other.defaults.layers;
        }
        if other.defaults.reference_cell.is_some() {
            self.defaults.reference_cell = other.defaults.reference_cell;
        }
        if other.behavior.verbosity.is_some() {
            self.behavior.verbosity = other.behavior.verbosity;
        }
    }

    fn merge_from_env(&mut self) {
        if let Ok(v) = env::var("LAMINAR_SYMMETRY") {
            self.modules.symmetry = Some(v);
        }
        if let Ok(v) = env::var("LAMINAR_NOTATION") {
            self.modules.notation = Some(v);
        }
        if let Ok(v) = env::var("LAMINAR_OPTION") {
            self.defaults.option = Some(v);
        }
        if let Ok(v) = env::var("LAMINAR_OUTPUT") {
            self.defaults.output = Some(PathBuf::from(v));
        }
        if let Ok(v) = env::var("LAMINAR_EXPORT") {
            self.defaults.export_format = Some(v);
        }
        if let Ok(v) = env::var("LAMINAR_LAYERS") {
            if let Ok(n) = v.parse() {
                self.defaults.layers = Some(n);
            }
        }
        if let Ok(v) = env::var("LAMINAR_REFERENCE_CELL") {
            self.defaults.reference_cell = Some(PathBuf::from(v));
        }
        if let Ok(v) = env::var("LAMINAR_VERBOSITY") {
            self.behavior.verbosity = Some(v);
        }
    }
}

fn global_config_path() -> Option<PathBuf> {
    if let Ok(xdg) = env::var("XDG_CONFIG_HOME") {
        return Some(PathBuf::from(xdg).join("laminar").join("laminar.conf"));
    }
    if let Ok(home) = env::var("HOME") {
        return Some(PathBuf::from(home).join(".config/laminar/laminar.conf"));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn load_from_file_parses_modules() {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        writeln!(f, "[modules]").unwrap();
        writeln!(f, "symmetry = \"honeycomb\"").unwrap();
        writeln!(f, "notation = \"jam\"").unwrap();
        let cfg = Config::load_from(f.path());
        assert_eq!(cfg.modules.symmetry.as_deref(), Some("honeycomb"));
        assert_eq!(cfg.modules.notation.as_deref(), Some("jam"));
    }

    #[test]
    fn load_from_file_parses_defaults() {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        writeln!(f, "[defaults]").unwrap();
        writeln!(f, "option = \"OneH\"").unwrap();
        writeln!(f, "layers = 4").unwrap();
        let cfg = Config::load_from(f.path());
        assert_eq!(cfg.defaults.option.as_deref(), Some("OneH"));
        assert_eq!(cfg.defaults.layers, Some(4));
    }

    #[test]
    fn missing_file_returns_default() {
        let cfg = Config::load_from(Path::new("/nonexistent/laminar.conf"));
        assert!(cfg.modules.symmetry.is_none());
        assert!(cfg.defaults.option.is_none());
    }

    #[test]
    fn invalid_file_returns_default_with_warning() {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        writeln!(f, "this is not valid = = toml").unwrap();
        let cfg = Config::load_from(f.path());
        assert!(cfg.modules.symmetry.is_none());
    }
}
