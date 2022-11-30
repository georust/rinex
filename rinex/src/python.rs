use pyo3::{exceptions::PyException, prelude::*};

use crate::prelude::*;

#[pymodule]
fn rinex(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<EpochFlag>()?;
    Ok(())
}
