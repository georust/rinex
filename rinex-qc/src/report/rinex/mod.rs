mod obs;
// mod nav;
// mod meteo;
use rinex::prelude::Constellation;

mod clock;
use clock::ClkReport;

mod meteo;
use meteo::MeteoReport;

// mod ionex;
use crate::report::Error;

use qc_traits::html::*;
use qc_traits::processing::{FilterItem, MaskOperand};

use obs::Report as ObsReport;

use rinex::prelude::{Observable, Rinex, RinexType};
use std::collections::HashMap;

// TODO
pub struct NavPage {}
// TODO
pub struct NavReport {
    pub pages: HashMap<Constellation, NavPage>,
}

impl RenderHtml for NavReport {
    fn to_inline_html(&self) -> Box<dyn RenderBox + '_> {
        box_html! {}
    }
}

pub struct DorisPage {}
// TODO
pub struct DorisReport {
    pub pages: HashMap<Constellation, DorisPage>,
}
impl DorisReport {
    pub fn new(rnx: &Rinex) -> Self {
        Self {
            pages: Default::default(),
        }
    }
}

impl RenderHtml for DorisReport {
    fn to_inline_html(&self) -> Box<dyn RenderBox + '_> {
        box_html! {}
    }
}

//TODO
pub struct IonexReport {}

impl IonexReport {
    pub fn new(rnx: &Rinex) -> Self {
        Self {}
    }
}
impl RenderHtml for IonexReport {
    fn to_inline_html(&self) -> Box<dyn RenderBox + '_> {
        box_html! {}
    }
}

impl NavReport {
    pub fn new(rnx: &Rinex) -> Self {
        Self {
            pages: Default::default(),
        }
    }
}

/// RINEX type dependent report
pub enum RINEXReport {
    Obs(ObsReport),
    Nav(NavReport),
    Clk(ClkReport),
    Meteo(MeteoReport),
    Doris(DorisReport),
    Ionex(IonexReport),
}

impl RINEXReport {
    pub fn new(rnx: &Rinex) -> Result<Self, Error> {
        match rnx.header.rinex_type {
            RinexType::DORIS => Ok(Self::Doris(DorisReport::new(rnx))),
            RinexType::ClockData => Ok(Self::Clk(ClkReport::new(rnx)?)),
            RinexType::MeteoData => Ok(Self::Meteo(MeteoReport::new(rnx)?)),
            RinexType::NavigationData => Ok(Self::Nav(NavReport::new(rnx))),
            RinexType::ObservationData => Ok(Self::Obs(ObsReport::new(rnx))),
            RinexType::IonosphereMaps => Ok(Self::Ionex(IonexReport::new(rnx))),
            _ => Err(Error::NonSupportedRINEX),
        }
    }
    pub fn as_obs(&self) -> Option<&ObsReport> {
        match self {
            Self::Obs(report) => Some(report),
            _ => None,
        }
    }
    pub fn as_nav(&self) -> Option<&NavReport> {
        match self {
            Self::Nav(report) => Some(report),
            _ => None,
        }
    }
    pub fn as_meteo(&self) -> Option<&MeteoReport> {
        match self {
            Self::Meteo(report) => Some(report),
            _ => None,
        }
    }
    pub fn as_clk(&self) -> Option<&ClkReport> {
        match self {
            Self::Clk(report) => Some(report),
            _ => None,
        }
    }
}
