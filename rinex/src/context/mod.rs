//! RINEX post processing context

use thiserror::Error;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::{
    merge::{Error as RinexMergeError, Merge as RinexMerge},
    prelude::{GroundPosition, Rinex},
    types::Type as RinexType,
    Error as RinexError,
};

use sp3::{prelude::SP3, Merge as SP3Merge, MergeError as SP3MergeError};

#[cfg(feature = "qc")]
use horrorshow::{box_html, helper::doctype, html, RenderBox};

#[cfg(feature = "qc")]
use rinex_qc_traits::HtmlReport;

#[derive(Debug, Error)]
pub enum Error {
    #[error("non supported file format")]
    NonSupportedFileFormat,
    #[error("failed to determine filename")]
    FileNameDetermination,
    #[error("invalid rinex format")]
    RinexError(#[from] RinexError),
    #[error("failed to extend rinex context")]
    RinexMergeError(#[from] RinexMergeError),
    #[error("failed to extend sp3 context")]
    SP3MergeError(#[from] SP3MergeError),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Hash)]
pub enum ProductType {
    /// GNSS carrier signal observation in the form
    /// of Observation RINEX data.
    Observation,
    /// Meteo sensors data wrapped as Meteo RINEX files.
    MeteoObservation,
    /// Broadcast Navigation message as contained in
    /// Navigation RINEX files.
    BroadcastNavigation,
    /// High precision clock data wrapped in Clock RINEX files.
    HighPrecisionOrbit,
    /// High precision orbital attitudes wrapped in Clock RINEX files.
    HighPrecisionClock,
    /// Antenna calibration information wrapped in ANTEX special RINEX files.
    Antex,
    /// Precise Ionosphere state wrapped in IONEX special RINEX files.
    Ionex,
}

impl From<RinexType> for ProductType {
    fn from(rt: RinexType) -> Self {
        match rt {
            RinexType::ObservationData => Self::Observation,
            RinexType::NavigationData => Self::BroadcastNavigation,
            RinexType::MeteoData => Self::MeteoObservation,
            RinexType::ClockData => Self::HighPrecisionClock,
            RinexType::IonosphereMaps => Self::Ionex,
            RinexType::AntennaData => Self::Antex,
        }
    }
}

#[derive(Clone)]
enum BlobData<'a> {
    /// RINEX content
    Rinex(&'a Rinex),
    /// SP3 content
    Sp3(&'a SP3),
}

impl<'a> BlobData<'a> {
    /// Returns reference to inner RINEX data.
    pub fn as_rinex(&self) -> Option<&'a Rinex> {
        match self {
            Self::Rinex(r) => Some(r),
            _ => None,
        }
    }
    /// Returns mutable reference to inner RINEX data.
    pub fn as_mut_rinex(&mut self) -> Option<&'a mut Rinex> {
        match self {
            Self::Rinex(r) => Some(r),
            _ => None,
        }
    }
    /// Returns reference to inner SP3 data.
    pub fn as_sp3(&self) -> Option<&'a SP3> {
        match self {
            Self::Sp3(s) => Some(s),
            _ => None,
        }
    }
    /// Returns mutable reference to inner SP3 data.
    pub fn as_mut_sp3(&mut self) -> Option<&'a mut SP3> {
        match self {
            Self::Sp3(s) => Some(s),
            _ => None,
        }
    }
}

/// RnxContext is a structure dedicated to RINEX post processing workflows,
/// like precise timing, positioning or atmosphere analysis.
#[derive(Clone, Default)]
pub struct RnxContext<'a> {
    /// Context is named after "primary" file.
    name: String,
    /// Files merged into self
    files: HashMap<ProductType, Vec<PathBuf>>,
    /// Context blob created by merging each members of each category
    blob: HashMap<ProductType, BlobData<'a>>,
}

impl<'a> RnxContext<'a> {
    /// Returns name of this context.
    /// Context is named after "primary" file where
    /// Observation RINEX is always prefered.
    pub fn name(&self) -> String {
        self.name.clone()
    }
    /// Returns reference to files loaded in given category
    pub fn files(&self, product: ProductType) -> Option<&Vec<PathBuf>> {
        self.files
            .iter()
            .filter_map(|(prod_type, paths)| {
                if *prod_type == product {
                    Some(paths)
                } else {
                    None
                }
            })
            .reduce(|k, _| k)
    }
    /// Returns reference to inner data of given category
    fn data(&'a self, product: ProductType) -> Option<&'a BlobData> {
        self.blob
            .iter()
            .filter_map(|(prod_type, data)| {
                if *prod_type == product {
                    Some(data)
                } else {
                    None
                }
            })
            .reduce(|k, _| k)
    }
    /// Returns mutable reference to inner data of given category
    fn data_mut(&'a mut self, product: ProductType) -> Option<&'a mut BlobData> {
        self.blob
            .iter_mut()
            .filter_map(|(prod_type, data)| {
                if *prod_type == product {
                    Some(data)
                } else {
                    None
                }
            })
            .reduce(|k, _| k)
    }
    /// Returns reference to inner RINEX data of given category
    pub fn rinex(&'a self, product: ProductType) -> Option<&'a Rinex> {
        self.data(product)?.as_rinex()
    }
    /// Returns reference to inner SP3 data
    pub fn sp3(&'a self) -> Option<&'a SP3> {
        self.data(ProductType::HighPrecisionOrbit)?.as_sp3()
    }
    // /// Returns mutable reference to inner RINEX data of given category
    // pub fn rinex_mut(&'a mut self, product: ProductType) -> Option<&'a mut Rinex> {
    //     self.data_mut(product)?.as_mut_rinex()
    // }
    // /// Returns mutable reference to inner SP3 data
    // pub fn sp3_mut(&'a mut self) -> Option<&'a mut SP3> {
    //     self.data_mut(ProductType::HighPrecisionOrbit)?
    //         .as_mut_sp3()
    // }
    /// Returns reference to inner [ProductType::Observation] data
    pub fn observation(&'a self) -> Option<&'a Rinex> {
        self.data(ProductType::BroadcastNavigation)?.as_rinex()
    }
    /// Returns reference to inner [ProductType::BroadcastNavigation] data
    pub fn broadcast_navigation(&'a self) -> Option<&'a Rinex> {
        self.data(ProductType::BroadcastNavigation)?.as_rinex()
    }
    // /// Returns mutal reference to inner [ProductType::Observation] data
    // pub fn observation_mut(&'a mut self) -> Option<&'a mut Rinex> {
    //     self.data_mut(ProductType::Observation)?.as_mut_rinex()
    // }
    /// Returns true if [ProductType::Observation] are present in Self
    pub fn has_observation(&self) -> bool {
        self.observation().is_some()
    }
    /// Returns true if High Precision Orbits also contains temporal information.
    pub fn sp3_has_clock(&self) -> bool {
        if let Some(sp3) = self.sp3() {
            sp3.sv_clock().count() > 0
        } else {
            false
        }
    }
    /// Load a single RINEX file into Self.
    /// File revision must be supported and must be correctly formatted
    /// for this operation to be effective.
    pub fn load_rinex(&'a mut self, path: &Path, rinex: &'a Rinex) -> Result<(), Error> {
        let prod_type = ProductType::from(rinex.header.rinex_type);
        // extend context blob
        //if let Some(inner) = self.rinex_mut(prod_type) {
        //    inner.merge_mut(rinex)?;
        //} else {
        self.blob.insert(prod_type, BlobData::Rinex(rinex));
        //}
        // extend file list
        if let Some(inner) = self
            .files
            .iter_mut()
            .filter_map(|(prod, files)| {
                if *prod == prod_type {
                    Some(files)
                } else {
                    None
                }
            })
            .reduce(|k, _| k)
        {
            inner.push(path.to_path_buf());
        } else {
            self.files.insert(prod_type, vec![path.to_path_buf()]);
        }
        Ok(())
    }
    /// Load a single SP3 file into Self.
    /// File revision must be supported and must be correctly formatted
    /// for this operation to be effective.
    pub fn load_sp3(&'a mut self, path: &Path, sp3: &'a SP3) -> Result<(), Error> {
        // extend context blob
        //if let Some(inner) = self.sp3_mut() {
        //    inner.merge_mut(sp3)?;
        //} else {
        self.blob
            .insert(ProductType::HighPrecisionOrbit, BlobData::Sp3(sp3));
        //}
        // extend file list
        if let Some(inner) = self
            .files
            .iter_mut()
            .filter_map(|(prod, files)| {
                if *prod == ProductType::HighPrecisionOrbit {
                    Some(files)
                } else {
                    None
                }
            })
            .reduce(|k, _| k)
        {
            inner.push(path.to_path_buf());
        } // else {
          //self.files.insert(ProductType::HighPrecisionOrbit, vec![path.to_path_buf()]);
          //}
        Ok(())
    }
    /// Returns possible Reference position defined in this context.
    /// Usually the Receiver location in the laboratory.
    pub fn ground_position(&self) -> Option<GroundPosition> {
        if let Some(data) = self.observation() {
            if let Some(pos) = data.header.ground_position {
                return Some(pos);
            }
        }
        if let Some(data) = self.broadcast_navigation() {
            if let Some(pos) = data.header.ground_position {
                return Some(pos);
            }
        }
        None
    }
}

#[cfg(feature = "qc")]
impl HtmlReport for RnxContext<'a> {
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
                        title: self.name(),
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
                    : "Observations"
                }
                td {
                    @ if self.obs_paths().is_none() {
                        : "None"
                    } else {
                        @ for path in self.obs_paths().unwrap() {
                            br {
                                : path.file_name()
                                    .unwrap()
                                    .to_string_lossy()
                                    .to_string()
                            }
                        }
                    }
                }
            }
            tr {
                td {
                    : "Broadcast Navigation"
                }
                td {
                    @ if self.nav_paths().is_none() {
                        : "None"
                    } else {
                        @ for path in self.nav_paths().unwrap() {
                            br {
                                : path.file_name()
                                    .unwrap()
                                    .to_string_lossy()
                                    .to_string()
                            }
                        }
                    }
                }
            }
            tr {
                td {
                    : "ANTEX"
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
