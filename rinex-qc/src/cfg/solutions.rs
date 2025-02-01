use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct QcSolutions {
    /// Automatically attach PPP solutions
    #[serde(default)]
    pub ppp: bool,
    /// Automatically attach CGGTTS solutions
    #[serde(default)]
    pub cggtts: bool,
}

impl QcSolutions {
    /// True when no solutions should be integrated
    pub(crate) fn is_empty(&self) -> bool {
        self.ppp == false && self.cggtts == false
    }

    /// True when at least one type of solutions should be integrated
    pub(crate) fn is_some(&self) -> bool {
        !self.is_empty()
    }
}
