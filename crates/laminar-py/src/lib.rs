use pyo3::prelude::*;

#[pymodule]
fn laminar(_py: Python<'_>, _m: &Bound<'_, PyModule>) -> PyResult<()> {
    todo!("PyO3 bindings: canonical filter, augment, count");
}
