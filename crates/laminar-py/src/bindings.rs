//! PyO3 bindings: JamChain, ReferenceCell, sandwich, compare, modules.

use std::collections::{BTreeMap, HashMap};
use std::path::Path;

use pyo3::exceptions::{PyIOError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyDict;

use laminar_core::canon::kappa_fast;
use laminar_core::cells;
use laminar_core::children::augment_to;
use laminar_core::types::JamChain as RsJamChain;
use laminar_core::JamOption;
use laminar_io::cif::to_cif;
use laminar_io::ReferenceCell as RsReferenceCell;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn parse_option(s: &str) -> PyResult<JamOption> {
    match s {
        "PL" => Ok(JamOption::PL),
        "BPL" => Ok(JamOption::BPL),
        "BU" => Ok(JamOption::BU),
        "BBU" => Ok(JamOption::BBU),
        "OneH" => Ok(JamOption::OneH),
        "OneT" => Ok(JamOption::OneT),
        "Janus1H" => Ok(JamOption::Janus1H),
        "Janus1T" => Ok(JamOption::Janus1T),
        _ => Err(PyValueError::new_err(format!(
            "unknown option: {} (available: PL, BPL, BU, BBU, OneH, OneT, Janus1H, Janus1T)",
            s
        ))),
    }
}

// ---------------------------------------------------------------------------
// JamChain
// ---------------------------------------------------------------------------

#[pyclass(name = "JamChain")]
#[derive(Clone)]
pub struct PyJamChain {
    inner: RsJamChain,
}

#[pymethods]
impl PyJamChain {
    #[staticmethod]
    fn parse(s: &str, option: &str) -> PyResult<Self> {
        let opt = parse_option(option)?;
        let chain = RsJamChain::parse(s, opt)
            .map_err(|e| PyValueError::new_err(format!("{}", e)))?;
        Ok(Self { inner: chain })
    }

    #[getter]
    fn raw(&self) -> String {
        self.inner.raw()
    }

    #[getter]
    fn canonical(&self) -> String {
        kappa_fast(&self.inner).raw()
    }

    #[getter]
    fn option(&self) -> String {
        format!("{:?}", self.inner.option())
    }

    #[getter]
    fn k(&self) -> usize {
        self.inner.k()
    }

    #[getter]
    fn cell(&self) -> String {
        format!("{:?}", cells::cell(&self.inner))
    }

    #[getter]
    fn is_palindromic(&self) -> bool {
        cells::is_palindromic(&self.inner)
    }

    #[getter]
    fn is_periodic(&self) -> bool {
        cells::is_periodic(&self.inner)
    }

    #[getter]
    fn period(&self) -> Option<usize> {
        cells::period(&self.inner)
    }

    /// Export to CIF string. Requires a ReferenceCell.
    fn to_cif(&self, cell: PyRef<'_, PyReferenceCell>) -> PyResult<String> {
        to_cif(&self.inner, &cell.inner, "P 1")
            .map_err(|e| PyValueError::new_err(format!("{}", e)))
    }

    /// Export to `ase.Atoms`. Requires the `ase` Python package.
    fn to_ase(&self, py: Python<'_>, cell: PyRef<'_, PyReferenceCell>) -> PyResult<PyObject> {
        let cif = to_cif(&self.inner, &cell.inner, "P 1")
            .map_err(|e| PyValueError::new_err(format!("{}", e)))?;
        let io = py.import_bound("io")?;
        let string_io = io.getattr("StringIO")?.call1((cif,))?;
        let ase_io = py.import_bound("ase.io")?;
        let atoms = ase_io.getattr("read")?.call1((string_io,))?;
        Ok(atoms.unbind())
    }

    fn __repr__(&self) -> String {
        format!("JamChain({:?}, k={})", self.raw(), self.k())
    }
}

// ---------------------------------------------------------------------------
// ReferenceCell
// ---------------------------------------------------------------------------

#[pyclass(name = "ReferenceCell")]
#[derive(Clone)]
pub struct PyReferenceCell {
    inner: RsReferenceCell,
}

#[pymethods]
impl PyReferenceCell {
    #[new]
    #[pyo3(signature = (a, c, z_up, z_down, species_map))]
    fn new(
        a: f64,
        c: f64,
        z_up: f64,
        z_down: f64,
        species_map: HashMap<String, String>,
    ) -> Self {
        Self {
            inner: RsReferenceCell {
                a,
                c,
                z_up,
                z_down,
                species_map,
            },
        }
    }

    #[staticmethod]
    fn from_toml(path: &str) -> PyResult<Self> {
        let cell = RsReferenceCell::from_toml(Path::new(path))
            .map_err(|e| PyIOError::new_err(format!("{}", e)))?;
        Ok(Self { inner: cell })
    }

    #[getter]
    fn a(&self) -> f64 {
        self.inner.a
    }
    #[getter]
    fn c(&self) -> f64 {
        self.inner.c
    }
    #[getter]
    fn z_up(&self) -> f64 {
        self.inner.z_up
    }
    #[getter]
    fn z_down(&self) -> f64 {
        self.inner.z_down
    }
    #[getter]
    fn species_map(&self) -> HashMap<String, String> {
        self.inner.species_map.clone()
    }

    fn __repr__(&self) -> String {
        format!(
            "ReferenceCell(a={}, c={}, z_up={}, z_down={})",
            self.inner.a, self.inner.c, self.inner.z_up, self.inner.z_down
        )
    }
}

// ---------------------------------------------------------------------------
// Module functions
// ---------------------------------------------------------------------------

/// `sandwich(n, option, export="stdout", count=False)`
#[pyfunction]
#[pyo3(signature = (n, option, export="stdout", count=false))]
fn sandwich(
    py: Python<'_>,
    n: usize,
    option: &str,
    export: &str,
    count: bool,
) -> PyResult<PyObject> {
    let opt = parse_option(option)?;
    let chains = augment_to(n, opt);

    if count {
        return Ok(chains.len().into_py(py));
    }

    match export {
        "stdout" => {
            let v: Vec<String> = chains.iter().map(|c| c.raw()).collect();
            Ok(v.into_py(py))
        }
        other => Err(PyValueError::new_err(format!(
            "unsupported export format in Python bindings: {} (use 'stdout', or use the CLI for other formats)",
            other
        ))),
    }
}

/// `compare(chains, option)`
#[pyfunction]
fn compare(py: Python<'_>, chains: Vec<String>, option: &str) -> PyResult<PyObject> {
    let opt = parse_option(option)?;

    let mut parsed: Vec<RsJamChain> = Vec::with_capacity(chains.len());
    for (i, raw) in chains.iter().enumerate() {
        match RsJamChain::parse(raw, opt) {
            Ok(c) => parsed.push(c),
            Err(e) => {
                return Err(PyValueError::new_err(format!(
                    "chain {} ({}): {}",
                    i, raw, e
                )));
            }
        }
    }

    let mut canonical_keys: BTreeMap<String, String> = BTreeMap::new();
    let mut orbits: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (raw, chain) in chains.iter().zip(&parsed) {
        let canon = kappa_fast(chain).raw();
        canonical_keys.insert(raw.clone(), canon.clone());
        orbits.entry(canon).or_default().push(raw.clone());
    }
    let equivalent = orbits.len() == 1;

    let dict = PyDict::new_bound(py);
    dict.set_item("equivalent", equivalent)?;
    dict.set_item("canonical_keys", canonical_keys)?;
    dict.set_item("orbits", orbits)?;
    Ok(dict.into_any().unbind())
}

/// `modules()`: list available symmetry/notation/export modules.
#[pyfunction]
fn modules(py: Python<'_>) -> PyResult<PyObject> {
    let dict = PyDict::new_bound(py);
    dict.set_item("symmetry", vec!["honeycomb"])?;
    dict.set_item("notation", vec!["jam"])?;
    dict.set_item("export_formats", vec!["stdout", "jam", "cif"])?;
    Ok(dict.into_any().unbind())
}

/// `version()`: Laminar crate version.
#[pyfunction]
fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

// ---------------------------------------------------------------------------
// Module registration
// ---------------------------------------------------------------------------

#[pymodule]
fn laminar(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyJamChain>()?;
    m.add_class::<PyReferenceCell>()?;
    m.add_function(wrap_pyfunction!(sandwich, m)?)?;
    m.add_function(wrap_pyfunction!(compare, m)?)?;
    m.add_function(wrap_pyfunction!(modules, m)?)?;
    m.add_function(wrap_pyfunction!(version, m)?)?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
