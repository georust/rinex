use pyo3::{
    exceptions::{PyException, PyTypeError},
    prelude::*,
    types::PyDict,
};

use std::collections::{BTreeMap, HashMap};
use std::hash::Hash;

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

/*
 * Type used when casting HashMaps to Python compatible
 * dictionnary
 */
pub struct PyMap<K, V>(HashMap<K, V>);

impl<K, V> PyMap<K, V> {
    pub(crate) fn new() -> Self {
        Self(HashMap::<K, V>::new())
    }
}

impl<K, V> From<HashMap<K, V>> for PyMap<K, V> {
    fn from(map: HashMap<K, V>) -> Self {
        Self(map)
    }
}

impl<'a, K, V> FromPyObject<'a> for PyMap<K, V>
where
    K: FromPyObject<'a> + Hash + Eq,
    V: FromPyObject<'a> + Hash + Eq,
{
    fn extract(ob: &'a PyAny) -> PyResult<Self> {
        if let Ok(dict) = ob.downcast::<PyDict>() {
            Ok(PyMap(
                dict.items()
                    .extract::<Vec<(K, V)>>()?
                    .into_iter()
                    .collect::<HashMap<_, _>>(),
            ))
        } else {
            Err(PyTypeError::new_err("dictionnary like type expected"))
        }
    }
}

pub struct PyBMap<K, V>(BTreeMap<K, V>);

impl<K, V> PyBMap<K, V> {
    pub(crate) fn new() -> Self {
        Self(BTreeMap::<K, V>::new())
    }
}

impl<K, V> From<BTreeMap<K, V>> for PyBMap<K, V> {
    fn from(bmap: BTreeMap<K, V>) -> Self {
        Self(bmap)
    }
}

impl<'a, K, V> FromPyObject<'a> for PyBMap<K, V>
where
    K: FromPyObject<'a> + Hash + Ord + Eq,
    V: FromPyObject<'a> + Hash + Ord + Eq,
{
    fn extract(ob: &'a PyAny) -> PyResult<Self> {
        if let Ok(dict) = ob.downcast::<PyDict>() {
            Ok(PyBMap(
                dict.items()
                    .extract::<Vec<(K, V)>>()?
                    .into_iter()
                    .collect::<BTreeMap<_, _>>(),
            ))
        } else {
            Err(PyTypeError::new_err("dictionnary like type expected"))
        }
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
