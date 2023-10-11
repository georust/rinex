//! Post and differential processing contexts
use horrorshow::{box_html, helper::doctype, html, RenderBox};
use rinex_qc_traits::HtmlReport;
use std::path::{Path, PathBuf};
use thiserror::Error;
use walkdir::WalkDir;

use crate::{merge, merge::Merge};

use sp3::Merge as SP3Merge;

use gnss::prelude::SV;

use crate::observation::Snr;
use crate::prelude::{Epoch, GroundPosition, Rinex};

use sp3::prelude::SP3;

use log::{error, trace};

#[derive(Debug, Error)]
pub enum Error {
    #[error("parsing error")]
    RinexError(#[from] crate::Error),
    #[error("invalid file type")]
    InvalidType,
    #[error("non supported file type")]
    NonSupportedType,
    #[error("failed to extend rinex context")]
    RinexMergeError(#[from] merge::Error),
    #[error("failed to extend sp3 context")]
    SP3MergeError(#[from] sp3::MergeError),
}

#[derive(Default, Debug, Clone)]
pub struct RnxData<T> {
    /// Source paths
    pub paths: Vec<PathBuf>,
    /// Data
    pub data: T,
}

impl<T> RnxData<T> {
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
pub struct RnxContext {
    /// Primary RINEX Data
    pub primary: RnxData<Rinex>,
    /// Optionnal NAV RINEX Data
    pub nav: Option<RnxData<Rinex>>,
    /// Optionnal ATX RINEX Data
    pub atx: Option<RnxData<Rinex>>,
    /// Optionnal SP3 Orbit Data
    pub sp3: Option<RnxData<SP3>>,
    /// true if orbits have been interpolated
    pub interpolated: bool,
}

impl RnxContext {
    /// Form a Rinex Context, either from a base directory
    /// or a single file. Two loading scenarios are supported:
    /// Example 1: single file, must be Observation RINEX
    /// Example 2: recursive.
    pub fn new(path: PathBuf) -> Result<Self, Error> {
        if path.is_dir() {
            /* recursive builder */
            Self::from_directory(path)
        } else {
            Self::from_observation_file(path.to_string_lossy().as_ref())
        }
    }
    /// Builds Rinex Context from a single (Observation) File
    fn from_observation_file(path: &str) -> Result<Self, Error> {
        Ok(Self {
            primary: {
                let data = Rinex::from_file(path)?;
                if !data.is_observation_rinex() {
                    return Err(Error::InvalidType);
                }
                RnxData {
                    data,
                    paths: vec![Path::new(path).to_path_buf()],
                }
            },
            nav: None,
            atx: None,
            sp3: None,
            interpolated: false,
        })
    }
    /// Builds Self by recursive browsing
    fn from_directory(path: PathBuf) -> Result<Self, Error> {
        let mut ret = RnxContext::default();
        let walkdir = WalkDir::new(&path.to_string_lossy().to_string()).max_depth(5);
        for entry in walkdir.into_iter().filter_map(|e| e.ok()) {
            if !entry.path().is_dir() {
                let fullpath = entry.path().to_string_lossy().to_string();
                match ret.load(&fullpath) {
                    Ok(_) => trace!(
                        "loaded \"{}\"",
                        entry.path().file_name().unwrap().to_string_lossy()
                    ),
                    Err(e) => error!("failed to load \"{}\", {:?}", fullpath, e),
                }
            }
        }
        Ok(ret)
    }
    /// Loads given file into Context
    pub fn load(&mut self, path: &str) -> Result<(), Error> {
        if let Ok(rnx) = Rinex::from_file(path) {
            let path = Path::new(path);
            if rnx.is_observation_rinex() {
                self.primary.data.merge_mut(&rnx)?;
                self.primary.paths.push(path.to_path_buf());
            } else if rnx.is_navigation_rinex() {
                if let Some(nav) = &mut self.nav {
                    /* extend existing blob */
                    nav.data.merge_mut(&rnx)?;
                    nav.paths.push(path.to_path_buf());
                } else {
                    self.nav = Some(RnxData {
                        data: rnx.clone(),
                        paths: vec![path.to_path_buf()],
                    })
                }
            } else if rnx.is_antex() {
                if let Some(atx) = &mut self.atx {
                    /* extend existing blob */
                    atx.data.merge_mut(&rnx)?;
                    atx.paths.push(path.to_path_buf());
                } else {
                    self.atx = Some(RnxData {
                        data: rnx.clone(),
                        paths: vec![path.to_path_buf()],
                    })
                }
            } else {
                return Err(Error::NonSupportedType);
            }
        } else if let Ok(data) = SP3::from_file(path) {
            let path = Path::new(path);
            if let Some(sp3) = &mut self.sp3 {
                /* extend existing blob */
                sp3.data.merge_mut(&data)?;
                sp3.paths.push(path.to_path_buf());
            } else {
                self.sp3 = Some(RnxData {
                    data: data.clone(),
                    paths: vec![path.to_path_buf()],
                })
            }
        } else {
            return Err(Error::NonSupportedType);
        }
        Ok(())
    }
    /// Returns reference to primary data
    pub fn primary_paths(&self) -> &[PathBuf] {
        &self.primary.paths
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
            Some(sp3.data())
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
    // /// Removes "incomplete" Epochs from OBS Data
    // pub fn complete_epoch_filter(&mut self, min_snr: Option<Snr>) {
    //     let total = self.primary_data().epoch().count();
    //     let complete_epochs: Vec<_> = self.primary_data().complete_epoch(min_snr).collect();
    //     if let Some(rec) = self.primary_data_mut().record.as_mut_obs() {
    //         rec.retain(|(epoch, _), (_, sv)| {
    //             let epoch_is_complete = complete_epochs.iter().find(|(e, sv_carriers)| e == epoch);

    //             if epoch_is_complete.is_none() {
    //                 false
    //             } else {
    //                 let (_, sv_carriers) = epoch_is_complete.unwrap();
    //                 sv.retain(|sv, observables| {
    //                     let carriers: Vec<Carrier> = sv_carriers
    //                         .iter()
    //                         .filter_map(
    //                             |(svnn, carrier)| {
    //                                 if sv == svnn {
    //                                     Some(*carrier)
    //                                 } else {
    //                                     None
    //                                 }
    //                             },
    //                         )
    //                         .collect();
    //                     observables.retain(|obs, _| {
    //                         let carrier = Carrier::from_observable(sv.constellation, obs)
    //                             .unwrap_or(Carrier::default());
    //                         carriers.contains(&carrier)
    //                     });
    //                     !observables.is_empty()
    //                 });
    //                 !sv.is_empty()
    //             }
    //         });
    //     }
    // }
    /// Performs SV Orbit interpolation
    pub fn orbit_interpolation(&mut self, _order: usize, _min_snr: Option<Snr>) {
        // /* NB: interpolate Complete Epochs only */
        //let complete_epoch: Vec<_> = self.primary_data().complete_epoch(min_snr).collect();
        //for (e, sv_signals) in complete_epoch {
        //    for (sv, carrier) in sv_signals {
        //        // if orbit already exists: do not interpolate
        //        // this will make things much quicker for high quality data products
        //        let found = self
        //            .sv_position()
        //            .into_iter()
        //            .find(|(sv_e, svnn, _)| *sv_e == e && *svnn == sv);
        //        if let Some((_, _, (x, y, z))) = found {
        //            // store as is
        //            self.orbits.insert((e, sv), (x, y, z));
        //        } else {
        //            if let Some(sp3) = self.sp3_data() {
        //                if let Some((x_km, y_km, z_km)) = sp3.sv_position_interpolate(sv, e, order)
        //                {
        //                    self.orbits.insert((e, sv), (x_km, y_km, z_km));
        //                }
        //            } else if let Some(nav) = self.navigation_data() {
        //                if let Some((x_m, y_m, z_m)) = nav.sv_position_interpolate(sv, e, order) {
        //                    self.orbits
        //                        .insert((e, sv), (x_m * 1.0E-3, y_m * 1.0E-3, z_m * 1.0E-3));
        //                }
        //            }
        //        }
        //    }
        //}
        self.interpolated = true;
    }
    /// Returns (unique) Iterator over SV orbit (3D positions)
    /// to be used in this context
    pub fn sv_position(&self) -> Vec<(Epoch, SV, (f64, f64, f64))> {
        if self.interpolated {
            todo!("CONCLUDE THIS PLEASE");
        } else {
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
}

impl HtmlReport for RnxContext {
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
                        title: format!("{:?}",
                            self.primary.paths.iter().map(|p| p.file_name().unwrap().to_string_lossy().to_string()).collect::<Vec<String>>());
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
                    @ for path in &self.primary.paths {
                        br {
                            : path.file_name()
                                .unwrap()
                                .to_string_lossy()
                                .to_string()
                        }
                    }
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
