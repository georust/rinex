use crate::RTKConfig;
use hifitime::Epoch;
use log::debug;
use map_3d::{deg2rad, ecef2geodetic, Ellipsoid};
use rinex::prelude::{Observable, RnxContext, SV};
use std::collections::HashMap;

use crate::SolverType;

mod tropo;

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
pub struct Modeling {
    #[cfg_attr(feature = "serde", serde(default = "default_sv_clock"))]
    pub sv_clock_bias: bool,
    #[cfg_attr(feature = "serde", serde(default = "default_tropo"))]
    pub tropo_delay: bool,
    #[cfg_attr(feature = "serde", serde(default = "default_iono"))]
    pub iono_delay: bool,
    #[cfg_attr(feature = "serde", serde(default = "default_sv_tgd"))]
    pub sv_total_group_delay: bool,
    #[cfg_attr(feature = "serde", serde(default = "default_earth_rot"))]
    pub earth_rotation: bool,
    #[cfg_attr(feature = "serde", serde(default = "default_rel_clock_corr"))]
    pub relativistic_clock_corr: bool,
}

pub trait Modelization {
    fn sum_up(&self, sv: SV) -> f64;
    fn modelize(
        &mut self,
        t: Epoch,
        sv: Vec<(SV, f64)>,
        lat_ddeg: f64,
        ctx: &RnxContext,
        cfg: &RTKConfig,
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

impl From<SolverType> for Modeling {
    fn from(solver: SolverType) -> Self {
        let mut s = Self::default();
        match solver {
            SolverType::PPP => {
                s.earth_rotation = true;
                s.relativistic_clock_corr = true;
            },
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
     * ctx: global context
     * cfg: global configuration
     */
    fn modelize(
        &mut self,
        t: Epoch,
        sv: Vec<(SV, f64)>,
        lat_ddeg: f64,
        ctx: &RnxContext,
        cfg: &RTKConfig,
    ) {
        self.clear();
        for (sv, elev) in sv {
            if cfg.modeling.tropo_delay {
                let tropo = tropo::tropo_delay(t, lat_ddeg, elev, ctx);
                debug!("{:?}: {} tropo delay {}", t, sv, tropo);
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
