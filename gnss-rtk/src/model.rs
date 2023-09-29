use crate::SolverType;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Modeling {
    pub sv_clock_bias: bool,
    pub sv_total_group_delay: bool,
    pub earth_rotation: bool,
    pub relativistic_clock_corr: bool,
}

impl Default for Modeling {
    fn default() -> Self {
        Self {
            sv_clock_bias: true,
            sv_total_group_delay: true,
            earth_rotation: false,
            relativistic_clock_corr: false,
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
