//! Physical, Atmospherical and Environmental modelizations
// use log::debug;
use crate::Config;
use crate::Mode;
use hifitime::Epoch;

//use map_3d::{deg2rad, ecef2geodetic, Ellipsoid};
use std::collections::HashMap;

use gnss::prelude::SV;

use log::{debug, trace};

mod tropo;
pub use tropo::TropoComponents;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

fn default_sv_clock() -> bool {
    true
}

fn default_sv_tgd() -> bool {
    true
}

fn default_iono() -> bool {
    true
}

fn default_tropo() -> bool {
    true
}

fn default_earth_rot() -> bool {
    false
}

fn default_rel_clock_corr() -> bool {
    false
}

#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
/// Superceed Tropospheric delay modeling
pub struct SuperceededTropoModel {
    /// Provide a Zenith Dry Delay [s] value yourself
    pub zdd: f64,
    /// Provide a Zenith Wet Delay [s] value yourself
    pub zwd: f64,
}

/// When solving, we let the possibility to superceed
/// atmospherical models, in case the user is capable of providing
/// a better or more trusty model himself.
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SuperceededModels {
    pub tropo: Option<SuperceededTropoModel>,
}

/// Atmospherical, Physical and Environmental modeling
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Modeling {
    #[cfg_attr(feature = "serde", serde(default))]
    pub sv_clock_bias: bool,
    #[cfg_attr(feature = "serde", serde(default))]
    pub tropo_delay: bool,
    #[cfg_attr(feature = "serde", serde(default))]
    pub iono_delay: bool,
    #[cfg_attr(feature = "serde", serde(default))]
    pub sv_total_group_delay: bool,
    #[cfg_attr(feature = "serde", serde(default))]
    pub earth_rotation: bool,
    #[cfg_attr(feature = "serde", serde(default))]
    pub relativistic_clock_corr: bool,
}

pub(crate) trait Modelization {
    fn sum_up(&self, sv: SV) -> f64;
    fn modelize(
        &mut self,
        t: Epoch,
        sv: Vec<(SV, f64)>,
        lat_ddeg: f64,
        alt_above_sea_m: f64,
        cfg: &Config,
        tropo_components: fn(Epoch, f64, f64) -> Option<TropoComponents>,
    );
}

impl Default for Modeling {
    fn default() -> Self {
        Self {
            sv_clock_bias: default_sv_clock(),
            iono_delay: default_iono(),
            tropo_delay: default_tropo(),
            sv_total_group_delay: default_sv_tgd(),
            earth_rotation: default_earth_rot(),
            relativistic_clock_corr: default_rel_clock_corr(),
        }
    }
}

impl From<Mode> for Modeling {
    fn from(mode: Mode) -> Self {
        let mut s = Self::default();
        match mode {
            //TODO
            //Mode::PPP => {
            //    s.earth_rotation = true;
            //    s.relativistic_clock_corr = true;
            //},
            _ => {},
        }
        s
    }
}

pub type Models = HashMap<SV, f64>;

impl Modelization for Models {
    /*
     * Eval new models at Epoch "t" given
     * sv: list of SV at given elevation,
     * lat_ddeg: receiver location latitude
     * cfg: global configuration
     */
    fn modelize(
        &mut self,
        t: Epoch,
        sv: Vec<(SV, f64)>,
        lat_ddeg: f64,
        alt_above_sea_m: f64,
        cfg: &Config,
        tropo_components: fn(Epoch, f64, f64) -> Option<TropoComponents>,
    ) {
        self.clear();
        for (sv, elev) in sv {
            self.insert(sv, 0.0_f64);

            if cfg.modeling.tropo_delay {
                let components = match tropo_components(t, lat_ddeg, alt_above_sea_m) {
                    Some(components) => {
                        trace!(
                            "superceeded tropo delay: zwd: {}, zdd: {}",
                            components.zwd,
                            components.zdd
                        );
                        components
                    },
                    None => {
                        let (zdd, zwd) = tropo::unb3_delay_components(t, lat_ddeg, alt_above_sea_m);
                        trace!("unb3 model: zwd: {}, zdd: {}", zdd, zwd);
                        TropoComponents { zwd, zdd }
                    },
                };

                let tropo = tropo::tropo_delay(elev, components.zwd, components.zdd);
                debug!("{:?}: {}(e={:.3}) tropo delay {} [m]", t, sv, elev, tropo);
                self.insert(sv, tropo);
            }
        }
    }
    fn sum_up(&self, sv: SV) -> f64 {
        self.iter()
            .filter_map(|(k, v)| if *k == sv { Some(*v) } else { None })
            .reduce(|k, _| k)
            .unwrap() // unsed in infaillible manner, at main level
    }
}
