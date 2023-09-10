use horrorshow::{box_html, helper::doctype, html, RenderBox};
use rinex_qc_traits::HtmlReport;
use std::collections::HashMap;
use std::path::PathBuf;

use rinex::prelude::{Epoch, GroundPosition, Rinex, Sv};
use sp3::prelude::SP3;

#[derive(Default, Debug, Clone)]
pub struct QcPrimaryData {
    /// Source path
    pub path: PathBuf,
    /// Data
    pub data: Rinex,
}

#[derive(Default, Debug, Clone)]
pub struct QcExtraData<T> {
    /// Source paths
    pub paths: Vec<PathBuf>,
    /// Data
    pub data: T,
}

impl<T> QcExtraData<T> {
    /// Returns reference to Inner Data
    pub fn data(&self) -> &T {
        &self.data
    }
    /// Returns mutable reference to Inner Data
    pub fn data_mut(&mut self) -> &mut T {
        &mut self.data
    }
    /// Returns list of files that created this context
    pub fn paths(&self) -> &[PathBuf] {
        &self.paths
    }
}

#[derive(Default, Debug, Clone)]
pub struct QcContext {
    /// Primary RINEX Data
    pub primary: QcPrimaryData,
    /// Optionnal NAV RINEX Data
    pub nav: Option<QcExtraData<Rinex>>,
    /// Optionnal ATX RINEX Data
    pub atx: Option<QcExtraData<Rinex>>,
    /// Optionnal SP3 Orbit Data
    pub sp3: Option<QcExtraData<SP3>>,
    // Interpolated orbits
    pub orbits: HashMap<(Sv, Epoch), (f64, f64, f64)>,
    /// true if orbits have been interpolated
    pub interpolated: bool,
}

impl QcContext {
    /// Returns reference to primary data
    pub fn primary_path(&self) -> &PathBuf {
        &self.primary.path
    }
    /// Returns reference to primary data
    pub fn primary_data(&self) -> &Rinex {
        &self.primary.data
    }
    /// Returns mutable reference to primary data
    pub fn primary_data_mut(&mut self) -> &mut Rinex {
        &mut self.primary.data
    }
    /// Returns true if provided context contains
    /// navigation data, either as primary or subsidary data set.
    pub fn has_navigation_data(&self) -> bool {
        self.primary.data.is_navigation_rinex() || self.nav.is_some()
    }
    /// Returns NAV files source path
    pub fn nav_paths(&self) -> Option<&[PathBuf]> {
        if let Some(ref nav) = self.nav {
            Some(nav.paths())
        } else {
            None
        }
    }
    /// Returns reference to navigation data specifically
    pub fn navigation_data(&self) -> Option<&Rinex> {
        if let Some(ref nav) = self.nav {
            Some(&nav.data)
        } else {
            None
        }
    }
    /// Returns mutable reference to navigation data specifically
    pub fn navigation_data_mut(&mut self) -> Option<&mut Rinex> {
        if let Some(ref mut nav) = self.nav {
            Some(&mut nav.data)
        } else {
            None
        }
    }
    /// Returns true if provided context contains SP3 high precision
    /// orbits data
    pub fn has_sp3(&self) -> bool {
        self.sp3.is_some()
    }
    /// Returns reference to SP3 data specifically
    pub fn sp3_data(&self) -> Option<&SP3> {
        if let Some(ref sp3) = self.sp3 {
            Some(&sp3.data())
        } else {
            None
        }
    }
    /// Returns SP3 files source path
    pub fn sp3_paths(&self) -> Option<&[PathBuf]> {
        if let Some(ref sp3) = self.sp3 {
            Some(sp3.paths())
        } else {
            None
        }
    }
    /// Returns true if provided context contains ATX RINEX Data
    pub fn has_atx(&self) -> bool {
        self.atx.is_some()
    }
    /// Returns reference to ATX data specifically
    pub fn atx_data(&self) -> Option<&Rinex> {
        if let Some(ref atx) = self.atx {
            Some(atx.data())
        } else {
            None
        }
    }
    /// Returns ATX files source path
    pub fn atx_paths(&self) -> Option<&[PathBuf]> {
        if let Some(ref atx) = self.atx {
            Some(atx.paths())
        } else {
            None
        }
    }
    /// Returns possible Reference position defined in this context.
    /// Usually the Receiver location in the laboratory.
    pub fn ground_position(&self) -> Option<GroundPosition> {
        if let Some(pos) = self.primary_data().header.ground_position {
            return Some(pos);
        }
        if let Some(data) = self.navigation_data() {
            if let Some(pos) = data.header.ground_position {
                return Some(pos);
            }
        }
        None
    }
    /// Request SV Orbit interpolation
    pub fn sv_orbit_interpolation(&mut self) {
        /* TODO: only interpolate on "complete" OBS Epochs */
        for ((e, _flag), sv, observ, data) in self.primary_data().carrier_phase() {
            // make it smart :
            // if orbit already exit do not interpolate
            // this will make things much quicker for high quality productions (sync'ed NAV + OBS)
            let found = self
                .sv_position()
                .into_iter()
                .find(|(sv_e, svnn, _)| *sv_e == e && *svnn == sv);
            if found.is_none() {
                if let Some(sp3) = self.sp3_data() {
                } else if let Some(nav) = self.navigation_data() {
                }
            }
        }
        self.interpolated = true;
    }
    /// Returns (unique) Iterator over SV orbit (3D positions)
    /// to be used in this context
    pub fn sv_position(&self) -> Vec<(Epoch, Sv, (f64, f64, f64))> {
        match self.sp3_data() {
            Some(sp3) => sp3.sv_position().collect(),
            _ => self
                .navigation_data()
                .unwrap()
                .sv_position()
                .map(|(e, sv, (x, y, z))| {
                    (e, sv, (x / 1000.0, y / 1000.0, z / 1000.0)) // match SP3 format
                })
                .collect(),
        }
    }
}

impl HtmlReport for QcContext {
    fn to_html(&self) -> String {
        format!(
            "{}",
            html! {
                : doctype::HTML;
                html {
                    head {
                        meta(charset="UTF-8");
                        meta(name="viewport", content="width=device-width, initial-scale=1");
                        link(rel="stylesheet", href="https:////cdn.jsdelivr.net/npm/bulma@0.9.4/css/bulma.min.css");
                        script(defer="true", src="https://use.fontawesome.com/releases/v5.3.1/js/all.js");
                        title:
                            if let Some(name) = self.primary.path.file_name() {
                                name.to_str()
                                    .unwrap_or("Unknown")
                            } else {
                                "Unknown"
                            }
                    }
                    body {
                        : self.to_inline_html()
                    }
                }
            }
        )
    }
    fn to_inline_html(&self) -> Box<dyn RenderBox + '_> {
        box_html! {
            tr {
                th {
                    : "File"
                }
                th {
                    : "Name"
                }
            }
            tr {
                td {
                    : format!("Primary ({})", self.primary_data().header.rinex_type)
                }
                td {
                    : self.primary.path.file_name()
                        .unwrap()
                        .to_string_lossy()
                        .to_string()
                }
            }
            tr {
                td {
                    : "NAV Augmentation"
                }
                td {
                    @ if self.nav_paths().is_none() {
                        : "None"
                    } else {
                        @ for path in self.nav_paths().unwrap() {
                            br {
                                : format!("{}", path.file_name().unwrap().to_string_lossy())
                            }
                        }
                    }
                }
            }
            tr {
                td {
                    : "ATX data"
                }
                td {
                    @ if self.atx_paths().is_none() {
                        : "None"
                    } else {
                        @ for path in self.atx_paths().unwrap() {
                            br {
                                : format!("{}", path.file_name().unwrap().to_string_lossy())
                            }
                        }
                    }
                }
            }
            tr {
                td {
                    : "SP3"
                }
                td {
                    @ if self.sp3_paths().is_none() {
                        : "None"
                    } else {
                        @ for path in self.sp3_paths().unwrap() {
                            br {
                                : format!("{}", path.file_name().unwrap().to_string_lossy())
                            }
                        }
                    }
                }
            }
        }
    }
}
