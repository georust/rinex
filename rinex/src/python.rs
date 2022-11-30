use pyo3::{exceptions::PyException, prelude::*};

use crate::{
    prelude::*,
    observation::{
        Crinex,
        record::*,
    },
};

#[pymodule]
fn rinex(_py: Python, m: &PyModule) -> PyResult<()> {
    /*
     * prelude module
     */
    m.add_class::<Epoch>()?;
    m.add_class::<EpochFlag>()?;
    /*
     * Observation module
     */
    m.add_class::<Crinex>()?;
    m.add_class::<Ssi>()?;
    m.add_class::<LliFlags>()?;
    m.add_class::<ObservationData>()?;
    Ok(())
}
