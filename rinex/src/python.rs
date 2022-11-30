use pyo3::{exceptions::PyException, prelude::*};

use crate::prelude::*;

/*impl std::convert::From<Error> for PyErr {
    fn from(err: Error) -> PyErr {
        PyException::new_err(err.to_string())
    }
}*/

#[pymodule]
fn rinex(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<EpochFlag>()?;
    Ok(())
}
