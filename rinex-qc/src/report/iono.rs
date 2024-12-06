//! Ionosphere modeling and parameters tab
use crate::prelude::{Config, QcContext};

pub struct IonoReport {
    /// TEC plot per signal source, over time
    tec_plot: Option<Plot>,
    /// IPP projection over time and signal sources
    ipp_proj: Option<Plot>,
}

impl IonoReport {
    /// Builds new [IonoReport] from [QcContext] using custom [Config]
    pub fn new(context: &QcContext, cfg: &Config) -> Self {
        let tec_plot = None;
        let ipp_proj = None;
        None
    }

    pub(crate) fn exists(&self) -> bool {
        !self.is_empty()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.tec_plot.is_none() && self.ipp_proj.is_none()
    }
}
