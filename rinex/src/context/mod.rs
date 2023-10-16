//! RINEX post processing context
use std::path::{Path, PathBuf};
use thiserror::Error;
use walkdir::WalkDir;

use crate::{merge, merge::Merge};

use sp3::Merge as SP3Merge;

// use crate::observation::Snr;
// use crate::prelude::Epoch;
use crate::prelude::{GroundPosition, Rinex};
// use gnss::prelude::SV;

use sp3::prelude::SP3;

use log::{error, trace};

#[cfg(feature = "qc")]
use horrorshow::{box_html, helper::doctype, html, RenderBox};

#[cfg(feature = "qc")]
use rinex_qc_traits::HtmlReport;

#[cfg(feature = "rtk")]
mod rtk;

#[cfg(feature = "rtk")]
pub use rtk::RTKContext;

#[derive(Debug, Error)]
pub enum Error {
    #[error("can only form a RINEX context from a directory, not a single file")]
    NotADirectory,
    #[error("parsing error")]
    RinexError(#[from] crate::Error),
    #[error("invalid file type")]
    NonSupportedType,
    #[error("failed to extend rinex context")]
    RinexMergeError(#[from] merge::Error),
    #[error("failed to extend sp3 context")]
    SP3MergeError(#[from] sp3::MergeError),
}

#[derive(Default, Debug, Clone)]
pub struct ProvidedData<T> {
    /// Source paths
    pub paths: Vec<PathBuf>,
    /// Data
    pub data: T,
}

impl<T> ProvidedData<T> {
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

/// RnxContext is a structure dedicated to typical
/// RINEX post processing workflows.
#[derive(Default, Debug, Clone)]
pub struct RnxContext {
    /// Optional Observation data, to provide
    /// sampled GNSS signals. Allows precise point
    /// positioning.
    pub obs: Option<ProvidedData<Rinex>>,
    /// Optional NAV RINEX Data, allows precise point
    /// positioning.
    pub nav: Option<ProvidedData<Rinex>>,
    /// Optional ATX RINEX Data
    pub atx: Option<ProvidedData<Rinex>>,
    /// Optional SP3 Orbit Data. Allows precise point
    /// positioning.
    pub sp3: Option<ProvidedData<SP3>>,
    /// Optional Meteo file, can serve
    /// for either detailed Meteorological survey,
    /// or high accuracy troposphere modeling
    pub meteo: Option<ProvidedData<Rinex>>,
    /// Optional IONEX file for accurate ionospheric
    /// delay modeling
    pub ionex: Option<ProvidedData<Rinex>>,
}

impl RnxContext {
    /// Form a Rinex Context, by loading a directory recursively
    pub fn new(path: PathBuf) -> Result<Self, Error> {
        if !path.is_dir() {
            Err(Error::NotADirectory)
        } else {
            /* recursive builder */
            Self::from_directory(path)
        }
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
    /// Unwraps inner RINEX data, by preference order:
    /// 1. Observation Data if provided
    /// 2. Navigation Data if provided
    /// 3. Meteo Data if provided
    /// 4. Ionex Data if provided
    /// 5. ATX Data if provided
    fn provided_rinex(&self) -> Option<&ProvidedData<Rinex>> {
        if let Some(data) = &self.obs {
            Some(&data)
        } else if let Some(data) = &self.nav {
            Some(&data)
        } else if let Some(data) = &self.meteo {
            Some(&data)
        } else if let Some(data) = &self.ionex {
            Some(&data)
        } else if let Some(data) = &self.atx {
            Some(&data)
        } else {
            None
        }
    }
    /// Unwraps most major RINEX data provided, by preference order:
    /// 1. Observation Data if provided
    /// 2. Navigation Data if provided
    /// 3. Meteo Data if provided
    /// 4. Ionex Data if provided
    /// 5. ATX Data if provided
    pub fn rinex_data(&self) -> Option<&Rinex> {
        let rinex_source = self.provided_rinex()?;
        Some(&rinex_source.data)
    }
    /// Unwraps most major file path provided, by preference order:
    /// 1. Observation Data if provided
    /// 2. Navigation Data if provided
    /// 3. Meteo Data if provided
    /// 4. Ionex Data if provided
    /// 5. ATX Data if provided
    pub fn rinex_path(&self) -> Option<&PathBuf> {
        let rinex_source = self.provided_rinex()?;
        let paths = &rinex_source.paths;
        if !paths.is_empty() {
            Some(&paths[0])
        } else {
            None
        }
    }
    /// Name this context, from most major file name that has been loaded,
    /// by preference order :
    /// 1. Observation Data if provided
    /// 2. Navigation Data if provided
    /// 3. Meteo Data if provided
    /// 4. Ionex Data if provided
    /// 5. ATX Data if provided
    pub fn rinex_name(&self) -> Option<String> {
        let path = self.rinex_path()?;
        Some(path.file_name().unwrap().to_string_lossy().to_string())
    }
    /// Loads given file into Context
    pub fn load(&mut self, filename: &str) -> Result<(), Error> {
        if let Ok(rnx) = Rinex::from_file(filename) {
            let path = Path::new(filename);
            if rnx.is_observation_rinex() {
                self.load_obs(path, &rnx)?;
                trace!("loaded observations \"{}\"", filename);
            } else if rnx.is_navigation_rinex() {
                self.load_nav(path, &rnx)?;
                trace!("loaded brdc nav \"{}\"", filename);
            } else if rnx.is_meteo_rinex() {
                self.load_meteo(path, &rnx)?;
                trace!("loaded meteo observations \"{}\"", filename);
            } else if rnx.is_ionex() {
                self.load_ionex(path, &rnx)?;
                trace!("loaded ionex \"{}\"", filename);
            } else if rnx.is_antex() {
                self.load_antex(path, &rnx)?;
                trace!("loaded antex dataset \"{}\"", filename);
            } else {
                return Err(Error::NonSupportedType);
            }
        } else if let Ok(sp3) = SP3::from_file(filename) {
            let path = Path::new(filename);
            self.load_sp3(path, &sp3)?;
            trace!("loaded sp3 \"{}\"", filename);
        }

        Ok(())
    }
    /// Returns possible Observation Data paths
    pub fn obs_paths(&self) -> Option<&[PathBuf]> {
        if let Some(ref obs) = self.obs {
            Some(obs.paths())
        } else {
            None
        }
    }
    /// Returns true if observations are present
    pub fn has_observation_data(&self) -> bool {
        self.obs.is_some()
    }
    /// Returns reference to Observation Data
    pub fn obs_data(&self) -> Option<&Rinex> {
        if let Some(ref obs) = self.obs {
            Some(&obs.data)
        } else {
            None
        }
    }
    /// Returns mutable reference to Observation Data
    pub fn obs_data_mut(&mut self) -> Option<&mut Rinex> {
        if let Some(ref mut obs) = self.obs {
            Some(&mut obs.data)
        } else {
            None
        }
    }
    /// Returns true if provided context contains navigation data
    pub fn has_navigation_data(&self) -> bool {
        self.nav.is_some()
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
    pub fn nav_data(&self) -> Option<&Rinex> {
        if let Some(ref nav) = self.nav {
            Some(&nav.data)
        } else {
            None
        }
    }
    /// Returns mutable reference to navigation data specifically
    pub fn nav_data_mut(&mut self) -> Option<&mut Rinex> {
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
    /// Returns true if self contains meteo data
    pub fn has_meteo_data(&self) -> bool {
        self.meteo.is_some()
    }
    /// Returns reference to Meteo Data
    pub fn meteo_data(&self) -> Option<&Rinex> {
        if let Some(ref data) = self.meteo {
            Some(&data.data)
        } else {
            None
        }
    }
    /// Returns Meteo files source paths
    pub fn meteo_paths(&self) -> Option<&[PathBuf]> {
        if let Some(ref meteo) = self.meteo {
            Some(meteo.paths())
        } else {
            None
        }
    }
    /// Returns mutable reference to meteo data specifically
    pub fn meteo_data_mut(&mut self) -> Option<&mut Rinex> {
        if let Some(ref mut meteo) = self.meteo {
            Some(&mut meteo.data)
        } else {
            None
        }
    }
    /// Returns IONEX files source paths
    pub fn ionex_paths(&self) -> Option<&[PathBuf]> {
        if let Some(ref ionex) = self.ionex {
            Some(ionex.paths())
        } else {
            None
        }
    }
    /// Returns reference to IONEX Data
    pub fn ionex_data(&self) -> Option<&Rinex> {
        if let Some(ref data) = self.ionex {
            Some(&data.data)
        } else {
            None
        }
    }
    /// Returns mutable reference to ionex data specifically
    pub fn ionex_data_mut(&mut self) -> Option<&mut Rinex> {
        if let Some(ref mut ionex) = self.ionex {
            Some(&mut ionex.data)
        } else {
            None
        }
    }
    /// Returns possible Reference position defined in this context.
    /// Usually the Receiver location in the laboratory.
    pub fn ground_position(&self) -> Option<GroundPosition> {
        if let Some(data) = self.obs_data() {
            if let Some(pos) = data.header.ground_position {
                return Some(pos);
            }
        }
        if let Some(data) = self.nav_data() {
            if let Some(pos) = data.header.ground_position {
                return Some(pos);
            }
        }
        None
    }
    fn load_obs(&mut self, path: &Path, rnx: &Rinex) -> Result<(), Error> {
        if let Some(obs) = &mut self.obs {
            obs.data.merge_mut(&rnx)?;
            obs.paths.push(path.to_path_buf());
        } else {
            self.obs = Some(ProvidedData {
                data: rnx.clone(),
                paths: vec![path.to_path_buf()],
            });
        }
        Ok(())
    }
    fn load_nav(&mut self, path: &Path, rnx: &Rinex) -> Result<(), Error> {
        if let Some(nav) = &mut self.nav {
            nav.data.merge_mut(&rnx)?;
            nav.paths.push(path.to_path_buf());
        } else {
            self.nav = Some(ProvidedData {
                data: rnx.clone(),
                paths: vec![path.to_path_buf()],
            });
        }
        Ok(())
    }
    fn load_meteo(&mut self, path: &Path, rnx: &Rinex) -> Result<(), Error> {
        if let Some(meteo) = &mut self.meteo {
            meteo.data.merge_mut(&rnx)?;
            meteo.paths.push(path.to_path_buf());
        } else {
            self.meteo = Some(ProvidedData {
                data: rnx.clone(),
                paths: vec![path.to_path_buf()],
            });
        }
        Ok(())
    }
    fn load_ionex(&mut self, path: &Path, rnx: &Rinex) -> Result<(), Error> {
        if let Some(ionex) = &mut self.ionex {
            ionex.data.merge_mut(&rnx)?;
            ionex.paths.push(path.to_path_buf());
        } else {
            self.ionex = Some(ProvidedData {
                data: rnx.clone(),
                paths: vec![path.to_path_buf()],
            });
        }
        Ok(())
    }
    fn load_antex(&mut self, path: &Path, rnx: &Rinex) -> Result<(), Error> {
        if let Some(atx) = &mut self.atx {
            atx.data.merge_mut(&rnx)?;
            atx.paths.push(path.to_path_buf());
        } else {
            self.atx = Some(ProvidedData {
                data: rnx.clone(),
                paths: vec![path.to_path_buf()],
            });
        }
        Ok(())
    }
    fn load_sp3(&mut self, path: &Path, sp3: &SP3) -> Result<(), Error> {
        if let Some(data) = &mut self.sp3 {
            /* extend existing context */
            data.data.merge_mut(&sp3)?;
            data.paths.push(path.to_path_buf());
        } else {
            self.sp3 = Some(ProvidedData {
                data: sp3.clone(),
                paths: vec![path.to_path_buf()],
            });
        }
        Ok(())
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
    // /// Performs SV Orbit interpolation
    // pub fn orbit_interpolation(&mut self, _order: usize, _min_snr: Option<Snr>) {
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
    //            } else if let Some(nav) = self.nav_data() {
    //                if let Some((x_m, y_m, z_m)) = nav.sv_position_interpolate(sv, e, order) {
    //                    self.orbits
    //                        .insert((e, sv), (x_m * 1.0E-3, y_m * 1.0E-3, z_m * 1.0E-3));
    //                }
    //            }
    //        }
    //    }
    //}
    //}
    // /// Returns (unique) Iterator over SV orbit (3D positions)
    // /// to be used in this context
    // #[cfg(feature = "nav")]
    // pub fn sv_position(&self) -> Vec<(Epoch, SV, (f64, f64, f64))> {
    //     if self.interpolated {
    //         todo!("CONCLUDE THIS PLEASE");
    //     } else {
    //         match self.sp3_data() {
    //             Some(sp3) => sp3.sv_position().collect(),
    //             _ => self
    //                 .nav_data()
    //                 .unwrap()
    //                 .sv_position()
    //                 .map(|(e, sv, (x, y, z))| {
    //                     (e, sv, (x / 1000.0, y / 1000.0, z / 1000.0)) // match SP3 format
    //                 })
    //                 .collect(),
    //         }
    //     }
    // }
}

#[cfg(feature = "qc")]
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
                        title: self.rinex_name().unwrap_or(String::from("Undefined"))
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
