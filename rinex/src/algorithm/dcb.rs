//! GNSS code biases estimator 
use crate::prelude::*;
use std::collections::{HashMap, BTreeMap};

/// GNSS code bias estimation trait
/// cf. phase data model <https://github.com/gwbres/rinex/blob/main/rinex-cli/doc/gnss-combination.md>.
/// Cf. page 12
/// <http://navigation-office.esa.int/attachments_12649498_1_Reichel_5thGalSciCol_2015.pdf>.
pub trait Dcb {
	/// ```
	/// use rinex::prelude::*;
	/// use rinex::observation::*;
	/// use rinex::processing::{Combination, Combine};
	/// let rinex = Rinex::from_file("../test_resources/OBS/V3/DUTH0630.22O")
	///		.unwrap();
	/// let gf = rinex.combine<Combination::GeometryFree>;
	/// ```
	fn dcb(&self) -> HashMap<String, HashMap<Sv, BTreeMap<(Epoch, EpochFlag), f64>>>;
}
