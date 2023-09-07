//! Hardware: receiver, antenna informations
use super::prelude::Sv;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// GNSS receiver description
#[derive(Default, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Rcvr {
    /// Receiver (hardware) model
    pub model: String,
    /// Receiver (hardware) identification info
    pub sn: String, // serial #
    /// Receiver embedded software info
    pub firmware: String, // firmware #
}

impl std::str::FromStr for Rcvr {
    type Err = std::io::Error;
    fn from_str(line: &str) -> Result<Self, Self::Err> {
        let (id, rem) = line.split_at(20);
        let (make, rem) = rem.split_at(20);
        let (version, _) = rem.split_at(20);
        Ok(Rcvr {
            sn: id.trim().to_string(),
            model: make.trim().to_string(),
            firmware: version.trim().to_string(),
        })
    }
}

/// Antenna description
#[derive(Debug, Clone, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Antenna {
    /// Hardware model / make descriptor
    pub model: String,
    /// Serial number / identification number
    pub sn: String,
    /// Base / reference point coordinates
    pub coords: Option<(f64, f64, f64)>,
    /// Optionnal `h` eccentricity (height component),
    /// referenced to base/reference point, in meter
    pub height: Option<f64>,
    /// Optionnal `eastern` eccentricity (eastern component),
    /// referenced to base/reference point, in meter
    pub eastern: Option<f64>,
    /// Optionnal `northern` eccentricity (northern component),
    /// referenced to base/reference point, in meter
    pub northern: Option<f64>,
}

impl Antenna {
    /// Sets desired model
    pub fn with_model(&self, m: &str) -> Self {
        let mut s = self.clone();
        s.model = m.to_string();
        s
    }
    /// Sets desired Serial Number
    pub fn with_serial_number(&self, sn: &str) -> Self {
        let mut s = self.clone();
        s.sn = sn.to_string();
        s
    }
    /// Sets reference/base coordinates (3D)
    pub fn with_base_coordinates(&self, coords: (f64, f64, f64)) -> Self {
        let mut s = self.clone();
        s.coords = Some(coords);
        s
    }
    /// Sets antenna `h` eccentricity component
    pub fn with_height(&self, h: f64) -> Self {
        let mut s = self.clone();
        s.height = Some(h);
        s
    }
    /// Sets antenna `eastern` coordinates component
    pub fn with_eastern_component(&self, e: f64) -> Self {
        let mut s = self.clone();
        s.eastern = Some(e);
        s
    }
    /// Sets antenna `northern` coordiantes component
    pub fn with_northern_component(&self, n: f64) -> Self {
        let mut s = self.clone();
        s.northern = Some(n);
        s
    }
}

#[cfg(feature = "qc")]
use horrorshow::RenderBox;

#[cfg(feature = "qc")]
use rinex_qc_traits::HtmlReport;

#[cfg(feature = "qc")]
impl HtmlReport for Antenna {
    fn to_html(&self) -> String {
        panic!("cannot render hardware::antenna on its own");
    }
    fn to_inline_html(&self) -> Box<dyn RenderBox + '_> {
        box_html! {
            table(class="table is-bordered") {
                tr {
                    th {
                        : "Model"
                    }
                    th {
                        : "SN#"
                    }
                    th {
                        : "Base Coordinates"
                    }
                    th {
                        : "Height"
                    }
                    th {
                        : "Eccentricity"
                    }
                }
                tr {
                    td {
                        : self.model.clone()
                    }
                    td {
                        : self.sn.clone()
                    }
                    td {
                        @ if let Some(coords) = self.coords {
                            : format!("({}m, {}m, {}m) (ECEF)",
                                coords.0, coords.1, coords.2)
                        } else {
                            : "Unknown"
                        }
                    }
                    td {
                        @ if let Some(h) = self.height {
                            : format!("{} m", h)
                        } else {
                            : "Unknown"
                        }
                    }
                    td {
                        @ if let Some(north) = self.northern {
                            @ if let Some(east) = self.eastern {
                                : format!("{}m N, {}m E", north, east)
                            } else {
                                : "Unknown"
                            }
                        } else {
                            : "Unknown"
                        }
                    }
                }
            }
        }
    }
}

#[cfg(feature = "qc")]
impl HtmlReport for Rcvr {
    fn to_html(&self) -> String {
        panic!("cannot render hardware::receiver on its own");
    }
    fn to_inline_html(&self) -> Box<dyn RenderBox + '_> {
        box_html! {
            table(class="table is-bordered; style=\"margin-bottom: 20px\"") {
                tr {
                    th {
                        : "Model"
                    }
                    th {
                        : "SN#"
                    }
                    th {
                        : "Firmware"
                    }
                }
                tr {
                    td {
                        : self.model.clone()
                    }
                    td {
                        : self.sn.clone()
                    }
                    td {
                        : self.firmware.clone()
                    }
                }
            }
        }
    }
}

/// Space vehicle antenna information,
/// only exists in ANTEX records
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SvAntenna {
    /// vehicle this antenna is attached to
    pub sv: Sv,
    /// antenna model description
    pub model: String,
    /// "YYYY-XXXA" year of vehicle launch
    /// XXX sequential launch vehicle
    /// A: alpha numeric sequence number within launch
    pub cospar: Option<String>,
}

impl SvAntenna {
    pub fn with_sv(&self, sv: Sv) -> Self {
        let mut s = self.clone();
        s.sv = sv;
        s
    }
    pub fn with_model(&self, m: &str) -> Self {
        let mut s = self.clone();
        s.model = m.to_string();
        s
    }
    pub fn with_cospar(&self, c: &str) -> Self {
        let mut s = self.clone();
        s.cospar = Some(c.to_string());
        s
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;
    #[test]
    fn rcvr_parser() {
        let content = "2090088             LEICA GR50          4.51                ";
        let rcvr = Rcvr::from_str(content);
        assert!(rcvr.is_ok());
        let rcvr = rcvr.unwrap();
        assert_eq!(rcvr.model, "LEICA GR50");
        assert_eq!(rcvr.sn, "2090088");
        assert_eq!(rcvr.firmware, "4.51");
    }
}
