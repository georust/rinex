mod obs;
// mod nav;
// mod meteo;
use itertools::Itertools;
use rinex::prelude::Constellation;

mod clock;
use clock::ClkReport;

mod meteo;
use meteo::MeteoReport;

mod ionex;
use ionex::IonexReport;

// mod ionex;
use crate::report::Error;

use maud::{html, Markup, Render};

use obs::Report as ObsReport;

use rinex::prelude::{Rinex, RinexType};
use std::collections::HashMap;

// TODO
pub struct NavPage {}
// TODO
pub struct NavReport {
    pages: HashMap<Constellation, NavPage>,
}

impl NavReport {
    pub fn new(rnx: &Rinex) -> Self {
        Self {
            pages: Default::default(),
        }
    }
    pub fn html_inline_menu_bar(&self) -> Markup {
        html! {
            a id="menu:brdc" {
                span class="icon" {
                    i class="fa-solid fa-satellite-dish" {}
                }
                "Broadcast Navigation (BRDC)"
            }
            ul class="menu-list" id="menu:tabs:brdc" style="display:none" {
                @for page in self.pages.keys().sorted() {
                    li {
                        a id=(format!("menu:brdc:{}", page)) style="margin-left:29px" {
                            span class="icon" {
                                i class="fa-solid fa-satellite" {}
                            }
                            (page.to_string())
                        }
                    }
                }
            }
        }
    }
}

impl Render for NavReport {
    fn render(&self) -> Markup {
        html! {}
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
    pub fn html_inline_menu_bar(&self) -> Markup {
        html! {
            a id="menu:doris" {
                span class="icon" {
                    i class="fa-solid fa-tower-cell" {}
                }
                "DORIS Observatoins"
            }
        }
    }
}

impl Render for DorisReport {
    fn render(&self) -> Markup {
        html! {}
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
