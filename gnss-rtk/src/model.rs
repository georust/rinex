use crate::SolverType;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

fn default_sv_clock() -> bool {
    true
}

fn default_sv_tgd() -> bool {
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
    #[cfg_attr(feature = "serde", serde(default = "default_sv_tgd"))]
    pub sv_total_group_delay: bool,
    #[cfg_attr(feature = "serde", serde(default = "default_earth_rot"))]
    pub earth_rotation: bool,
    #[cfg_attr(feature = "serde", serde(default = "default_rel_clock_corr"))]
    pub relativistic_clock_corr: bool,
}

impl Default for Modeling {
    fn default() -> Self {
        Self {
            sv_clock_bias: default_sv_clock(),
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
                s.relativistic_clock_corr = false;
            },
            _ => {},
        }
        s
    }
}
