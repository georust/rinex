use pyo3::{exceptions::PyException, prelude::*};

use crate::{
    hardware::*,
    header::MarkerType,
    navigation::*,
    observation::{record::*, Crinex, Snr},
    prelude::*,
};

impl std::convert::From<Error> for PyErr {
    fn from(err: Error) -> PyErr {
        PyException::new_err(err.to_string())
    }
}

#[pymodule]
fn rinex(_py: Python, m: &PyModule) -> PyResult<()> {
    /*
     * TODO: follow the crate module names
     * this should be the wrapped in the "prelude" module
     */
    m.add_class::<Epoch>()?;
    m.add_class::<EpochFlag>()?;
    m.add_class::<Rcvr>()?;
    m.add_class::<Antenna>()?;
    //m.add_class::<GroundPosition>()?;
    m.add_class::<Augmentation>()?;
    m.add_class::<Sv>()?;
    // header
    m.add_class::<MarkerType>()?;
    // rinex
    m.add_class::<Header>()?;
    m.add_class::<Rinex>()?;
    /*
     * TODO: Observation module
     */
    m.add_class::<Crinex>()?;
    m.add_class::<Snr>()?;
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
