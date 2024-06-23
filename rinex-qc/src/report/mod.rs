//! Generic analysis report
use crate::prelude::{ProductType, QcConfig, QcContext};
use itertools::Itertools;
use maud::{html, Markup, PreEscaped, Render, DOCTYPE};
use std::collections::HashMap;
use thiserror::Error;

// shared analysis, that may apply to several products
mod shared;

mod summary;
use summary::QcSummary;

mod rinex;
use rinex::RINEXReport;

mod navi;
use navi::NAVIReport;

#[cfg(feature = "sp3")]
mod sp3;

#[cfg(feature = "sp3")]
use sp3::SP3Report;

#[derive(Debug, Error)]
pub enum Error {
    #[error("non supported RINEX format")]
    NonSupportedRINEX,
    #[error("sampling analysis failed to return")]
    SamplingAnalysis,
    #[error("missing Clock RINEX header")]
    MissingClockHeader,
    #[error("missing Meteo RINEX header")]
    MissingMeteoHeader,
    #[error("missing IONEX header")]
    MissingIonexHeader,
}

enum ProductReport {
    /// RINEX products report
    RINEX(RINEXReport),
    #[cfg(feature = "sp3")]
    /// SP3 product report
    SP3(SP3Report),
}

impl ProductReport {
    pub fn html_inline_menu_bar(&self) -> Markup {
        match self {
            #[cfg(feature = "sp3")]
            Self::SP3(report) => report.html_inline_menu_bar(),
            Self::RINEX(report) => report.html_inline_menu_bar(),
        }
    }
}

fn html_id(product: &ProductType) -> &str {
    match product {
        ProductType::IONEX => "ionex",
        ProductType::DORIS => "doris",
        ProductType::ANTEX => "antex",
        ProductType::Observation => "obs",
        ProductType::BroadcastNavigation => "brdc",
        ProductType::HighPrecisionClock => "clk",
        ProductType::MeteoObservation => "meteo",
        #[cfg(feature = "sp3")]
        ProductType::HighPrecisionOrbit => "sp3",
    }
}

impl Render for ProductReport {
    fn render(&self) -> Markup {
        match self {
            Self::RINEX(report) => match report {
                RINEXReport::Obs(report) => {
                    html! {
                        div class="section" {
                            (report.render())
                        }
                    }
                },
                RINEXReport::Doris(report) => {
                    html! {
                        div class="section" {
                            (report.render())
                        }
                    }
                },
                RINEXReport::Ionex(report) => {
                    html! {
                        div class="section" {
                            (report.render())
                        }
                    }
                },
                RINEXReport::Nav(report) => {
                    html! {
                        div class="section" {
                            (report.render())
                        }
                    }
                },
                RINEXReport::Clk(report) => {
                    html! {
                        div class="section" {
                            (report.render())
                        }
                    }
                },
                RINEXReport::Meteo(report) => {
                    html! {
                        div class="section" {
                            (report.render())
                        }
                    }
                },
            },
            #[cfg(feature = "sp3")]
            Self::SP3(report) => {
                html! {
                    div class="section" {
                        (report.render())
                    }
                }
            },
        }
    }
}

impl ProductReport {
    pub fn as_rinex(&self) -> Option<&RINEXReport> {
        match self {
            Self::RINEX(report) => Some(report),
            _ => None,
        }
    }
    #[cfg(feature = "sp3")]
    pub fn as_sp3(&self) -> Option<&SP3Report> {
        match self {
            Self::SP3(report) => Some(report),
            _ => None,
        }
    }
}

/// [QcReport] is a generic structure to report complex analysis results
pub struct QcReport {
    /// Name of this report.
    /// Currently, the report is named after the primary input product.
    name: String,
    /// Report Summary (always present)
    summary: QcSummary,
    /// NAVI QC only available on full + compatible contexts
    navi: Option<NAVIReport>,
    /// In depth analysis per input product.
    /// In summary mode, these do not exist (empty).
    products: HashMap<ProductType, ProductReport>,
    /// Custom chapters
    custom_parts: HashMap<String, Box<dyn Render>>,
}

impl QcReport {
    /// Builds a new GNSS report, ready to be rendered
    pub fn new(context: &QcContext, cfg: QcConfig) -> Self {
        Self {
            navi: None,
            name: context.name(),
            custom_parts: HashMap::new(),
            summary: QcSummary::new(&context, &cfg),
            // Build the report, which comprises
            //   1. one general (high level) context tab
            //   2. one tab per product type (which can have sub tabs itself)
            //   3. one complex tab for "shared" analysis
            products: {
                let mut items = HashMap::<ProductType, ProductReport>::new();
                // one tab per RINEX product
                for product in [
                    ProductType::Observation,
                    ProductType::DORIS,
                    ProductType::MeteoObservation,
                    ProductType::BroadcastNavigation,
                    ProductType::HighPrecisionClock,
                    ProductType::IONEX,
                    ProductType::ANTEX,
                ] {
                    if let Some(rinex) = context.rinex(product) {
                        if let Ok(report) = RINEXReport::new(rinex) {
                            items.insert(product, ProductReport::RINEX(report));
                        }
                    }
                }
                // one tab for SP3 when supported
                #[cfg(feature = "sp3")]
                if let Some(sp3) = context.sp3() {
                    items.insert(
                        ProductType::HighPrecisionOrbit,
                        ProductReport::SP3(SP3Report::new(sp3)),
                    );
                }
                items
            },
        }
    }
    /// Add a custom chapter to the report
    pub fn add_custom(&mut self, section: &str, chapter: Box<dyn Render>) {
        self.custom_parts.insert(section.to_string(), chapter);
    }
}

/// HTML menu bar
struct HtmlMenuBar {}

impl HtmlMenuBar {
    pub fn render(report: &QcReport) -> Markup {
        html! {
            aside class="menu" {
                p class="menu-label" {
                    (format!("RINEX-QC v{}", env!("CARGO_PKG_VERSION")))
                }
                ul class="menu-list" {
                    li {
                        a id="menu:summary" {
                            span class="icon" {
                                i class="fa fa-home" {}
                            }
                            "Summary"
                        }
                    }
                    @for product in report.products.keys().sorted() {
                        @if let Some(report) = report.products.get(&product) {
                            li {
                                (report.html_inline_menu_bar())
                            }
                        }
                    }
                    p class="menu-label" {
                        a href="https://github.com/georust/rinex/wiki" style="margin-left:29px" {
                            "Documentation"
                        }
                    }
                    p class="menu-label" {
                        a href="https://github.com/georust/rinex/issues" style="margin-left:29px" {
                            "Bug Report"
                        }
                    }
                    p class="menu-label" {
                        a href="https://github.com/georust/rinex" {
                            span class="icon" {
                                i class="fa-brands fa-github" {}
                            }
                            "Sources"
                        }
                    }
                } // menu-list
            }//menu
        }
    }
}

impl Render for QcReport {
    fn render(&self) -> Markup {
        html! {
            (DOCTYPE)
            html {
                head {
                    meta charset="utf-8";
                    meta http-equip="X-UA-Compatible" content="IE-edge";
                    meta name="viewport" content="width=device-width, initial-scale=1";
                    link rel="icon" type="image/x-icon" href="https://raw.githubusercontent.com/georust/meta/master/logo/logo.png";
                    script src="https://cdn.plot.ly/plotly-2.12.1.min.js" {};
                    script defer="true" src="https://use.fontawesome.com/releases/v5.3.1/js/all.js" {};
                    script src="https://cdn.jsdelivr.net/npm/mathjax@3.2.2/es5/tex-svg.js" {};
                    link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/bulma@1.0.0/css/bulma.min.css";
                    link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.5.2/css/all.min.css";
                }//head
                body {
                        //div id="plotly-html-element" class="plotly-graph-div" style="height:100%; width:100;" {
                        //}
                        div id="title" {
                            title {
                                "RINEX QC"
                            }
                        }
                        div id="body" {
                            div class="columns is-fullheight" {
                                div id="menubar" class="column is-3 is-sidebar-menu is-hidden-mobile" {
                                    (HtmlMenuBar::render(&self))
                                } // id=menubar
                                div class="hero is-fullheight" {
                                    div id="summary" class="container is-main" style="display:block" {
                                        div class="section" {
                                            (self.summary.render())
                                        }
                                    }//id=summary
                                    @for product in self.products.keys().sorted() {
                                        @if let Some(report) = self.products.get(product) {
                                            div id=(html_id(product)) class="container is-main" style="display:none" {
                                                (report.render())
                                            }
                                        }
                                    }
                                }//class=hero
                            } // class=columns
                        }
                        script {
                          (PreEscaped(
"
  var sidebar_menu = document.getElementById('menubar');
  var main_pages = document.getElementsByClassName('is-main');
  var sub_pages = document.getElementsByClassName('is-page');

  sidebar_menu.onclick = function(evt) {
    var clicked_id = evt.originalTarget.id;
    var category = clicked_id.substring(5).split(':')[0];
    var tab = clicked_id.substring(5).split(':')[1];
    var is_tab = clicked_id.split(':').length == 3;
    console.log('clicked id: ' + clicked_id + ' category: ' + category + ' tab: ' +is_tab);

    if (is_tab == true ) {
      var targetted_tab = category+':'+tab;
      for (var i=0; i<main_pages.length; i++) {
        if (main_pages[i].id == category) {
          console.log('Matched: '+main_pages[i].id);
          main_pages[i].style = 'display:block';
        } else {
          main_pages[i].style = 'display:none';
        }
      }
    } else {
      var targetted_tab = category+':'+tab;
      for (var i=0; i<main_pages.length; i++) {
        if (main_pages[i].id == category) {
          console.log('Matched: '+main_pages[i].id);
          main_pages[i].style = 'display:block';
        } else {
          main_pages[i].style = 'display:none';
        }
      }
    }
  }
"
                          ))
                        } //JS
                }//body
            }
        }
    }
}
