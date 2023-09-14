use horrorshow::{box_html, helper::doctype, html, RenderBox};
use rinex::prelude::{GroundPosition, Rinex};
use rinex::Error;
use rinex_qc_traits::HtmlReport;
use std::path::{Path, PathBuf};

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

impl QcPrimaryData {
    /// Parses Self from local file
    pub fn from_file(path: &str) -> Result<Self, Error> {
        Ok(Self {
            path: Path::new(path).to_path_buf(),
            data: Rinex::from_file(path)?,
        })
    }
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
