mod obs;

use itertools::Itertools;
use rinex::prelude::Constellation;

mod clock;
use clock::ClkReport;

mod doris;
use doris::DorisReport;

mod meteo;
use meteo::MeteoReport;

mod ionex;
use ionex::IonexReport;

mod nav;
use nav::NavReport;

use crate::report::Error;

use maud::{html, Markup, Render};

use obs::Report as ObsReport;

use rinex::prelude::{Rinex, RinexType};
use std::collections::HashMap;

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
    pub fn new(rnx: &Rinex, sky_plot: bool) -> Result<Self, Error> {
        match rnx.header.rinex_type {
            RinexType::DORIS => Ok(Self::Doris(DorisReport::new(rnx))),
            RinexType::ClockData => Ok(Self::Clk(ClkReport::new(rnx)?)),
            RinexType::MeteoData => Ok(Self::Meteo(MeteoReport::new(rnx)?)),
            RinexType::NavigationData => Ok(Self::Nav(NavReport::new(rnx, sky_plot))),
            RinexType::ObservationData => Ok(Self::Obs(ObsReport::new(rnx))),
            RinexType::IonosphereMaps => Ok(Self::Ionex(IonexReport::new(rnx)?)),
            _ => Err(Error::NonSupportedRINEX),
        }
    }
    pub fn html_inline_menu_bar(&self) -> Markup {
        match self {
            Self::Obs(report) => report.html_inline_menu_bar(),
            Self::Nav(report) => report.html_inline_menu_bar(),
            Self::Clk(report) => report.html_inline_menu_bar(),
            Self::Meteo(report) => report.html_inline_menu_bar(),
            Self::Doris(report) => report.html_inline_menu_bar(),
            Self::Ionex(report) => report.html_inline_menu_bar(),
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
