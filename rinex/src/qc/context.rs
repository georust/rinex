use horrorshow::{helper::doctype, RenderBox};
use std::path::{Path, PathBuf};

use crate::prelude::GroundPosition;
use crate::quality::HtmlReport;
use crate::{Error, Rinex};

use sp3::prelude::SP3;

#[derive(Default, Debug, Clone)]
pub struct QcInputData {
    /// File source path
    pub path: PathBuf,
    /// File Data
    pub rinex: Rinex,
}

impl QcInputData {
    pub fn new(path: &str) -> Result<Self, Error> {
        Ok(Self {
            path: Path::new(path).to_path_buf(),
            rinex: Rinex::from_file(path)?,
        })
    }
}

#[derive(Default, Debug, Clone)]
pub struct QcInputSp3Data {
    /// File source path
    pub path: PathBuf,
    /// File Data
    pub sp3: SP3,
}

impl QcInputSp3Data {
    pub fn new(path: &str) -> Result<Self, sp3::Errors> {
        Ok(Self {
            path: Path::new(path).to_path_buf(),
            sp3: SP3::from_file(path)?,
        })
    }
}

#[derive(Default, Debug, Clone)]
pub struct QcContext {
    /// Primary RINEX file
    pub primary: QcInputData,
    /// Optionnal NAV augmentation
    pub nav: Option<QcInputData>,
    /// Optionnal ATX data
    pub atx: Option<QcInputData>,
    /// Optionnal SP3 data
    pub sp3: Option<QcInputSp3Data>,
}

impl QcContext {
    /// Returns reference to primary data
    pub fn primary_data(&self) -> &Rinex {
        &self.primary.rinex
    }
    /// Returns reference to primary data
    pub fn primary_path(&self) -> &PathBuf {
        &self.primary.path
    }
    /// Returns true if provided context contains
    /// navigation data, either as primary or subsidary data set.
    pub fn has_navigation_data(&self) -> bool {
        self.primary.rinex.is_navigation_rinex() || self.nav.is_some()
    }
    /// Returns reference to navigation data specifically
    pub fn navigation_data(&self) -> Option<&Rinex> {
        if let Some(ref nav) = self.nav {
            Some(&nav.rinex)
        } else {
            None
        }
    }
    /// Returns possible Reference position defined in this context.
    /// Usually the Receiver location in the laboratory.
    pub fn ground_position(&self) -> Option<GroundPosition> {
        if let Some(pos) = self.primary.rinex.header.ground_position {
            return Some(pos);
        }
        if let Some(nav) = &self.nav {
            if let Some(pos) = nav.rinex.header.ground_position {
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
                    : format!("Primary ({})", self.primary.rinex.header.rinex_type)
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
                    @ if let Some(nav) = &self.nav {
                        : nav.path.file_name()
                            .unwrap()
                            .to_string_lossy()
                            .to_string()
                    } else {
                        : "None"
                    }
                }
            }
            tr {
                td {
                    : "ATX data"
                }
                td {
                    @ if let Some(atx) = &self.atx {
                        : atx.path.file_name()
                            .unwrap()
                            .to_string_lossy()
                            .to_string()
                    } else {
                        : "None"
                    }
                }
            }
        }
    }
}
