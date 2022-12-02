use pyo3::{exceptions::PyException, prelude::*};

use crate::{
    prelude::*,
    observation::{
        Crinex,
        record::*,
    },
    header::{
        MarkerType,
    },
    hardware::*,
    navigation::*,
};

impl std::convert::From<Error> for PyErr {
    fn from(err: Error) -> PyErr {
        PyException::new_err(err.to_string())
    }
}

#[pymodule]
fn rinex(_py: Python, m: &PyModule) -> PyResult<()> {
    /*
     * TODO: prelude module
     */
    m.add_class::<Epoch>()?;
    m.add_class::<EpochFlag>()?;
    m.add_class::<Rcvr>()?;
    m.add_class::<Antenna>()?;
    m.add_class::<Augmentation>()?;
    // header
    m.add_class::<MarkerType>()?;
    /*
     * TODO: Observation module
     */
    m.add_class::<Crinex>()?;
    m.add_class::<Ssi>()?;
    m.add_class::<LliFlags>()?;
    m.add_class::<ObservationData>()?;
    /*
     * TODO: Navigation module
     */
    m.add_class::<MsgType>()?;
    m.add_class::<FrameClass>()?;
    m.add_class::<Ephemeris>()?;
    Ok(())
}
