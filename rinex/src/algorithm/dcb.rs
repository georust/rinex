//! GNSS code biases estimator
use crate::prelude::*;
use std::collections::{BTreeMap, HashMap};

/// GNSS code bias estimation trait
/// cf. phase data model <https://github.com/gwbres/rinex/blob/main/rinex-cli/doc/gnss-combination.md>.
/// Cf. page 12
/// <http://navigation-office.esa.int/attachments_12649498_1_Reichel_5thGalSciCol_2015.pdf>.
pub trait Dcb {
    /// Returns Differential Code Bias estimates, sorted per (unique)
    /// signals combinations and for each individual Sv.
    /// ```
    /// use rinex::prelude::*;
    /// use rinex::processing::*; // .dcb()
    ///
    /// let rinex = Rinex::from_file("../test_resources/OBS/V3/DUTH0630.22O")
    ///		.unwrap();
    /// let dcb = rinex.dcb();
    /// ```
    fn dcb(&self) -> HashMap<String, BTreeMap<Sv, BTreeMap<(Epoch, EpochFlag), f64>>>;
}
