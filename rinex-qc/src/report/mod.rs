//! Generic analysis report
use crate::{ProductType, QcConfig, QcContext};
use itertools::Itertools;
use qc_traits::html::*;
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

use crate::cfg::QcReportType;

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
    pub fn html_inline_menu_bar(&self) -> Box<dyn RenderBox + '_> {
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

impl RenderHtml for ProductReport {
    fn to_inline_html(&self) -> Box<dyn RenderBox + '_> {
        match self {
            Self::RINEX(report) => match report {
                RINEXReport::Obs(report) => {
                    box_html! {
                        div(class="section") {
                            : report.to_inline_html()
                        }
                    }
                },
                RINEXReport::Doris(report) => {
                    box_html! {
                        div(class="section") {
                            : report.to_inline_html()
                        }
                    }
                },
                RINEXReport::Ionex(report) => {
                    box_html! {
                        div(class="section") {
                            : report.to_inline_html()
                        }
                    }
                },
                RINEXReport::Nav(report) => {
                    box_html! {
                        div(class="section") {
                            : report.to_inline_html()
                        }
                    }
                },
                RINEXReport::Clk(report) => {
                    box_html! {
                        div(class="section") {
                            : report.to_inline_html()
                        }
                    }
                },
                RINEXReport::Meteo(report) => {
                    box_html! {
                        div(class="section") {
                            : report.to_inline_html()
                        }
                    }
                },
            },
            #[cfg(feature = "sp3")]
            Self::SP3(report) => {
                box_html! {
                    div(class="section") {
                        : report.to_inline_html()
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
}

impl QcReport {
    /// Builds a new GNSS report, ready to be rendered
    pub fn new(context: &QcContext, cfg: QcConfig) -> Self {
        Self {
            navi: None,
            name: context.name(),
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
}

struct HtmlMenuBar {}

impl HtmlMenuBar {
    pub fn to_inline_html(report: &QcReport) -> Box<dyn RenderBox + '_> {
        box_html! {
            aside(class="menu") {
                p(class="menu-label") {
                    : &format!("RINEX-QC v{}", env!("CARGO_PKG_VERSION"))
                }
                ul(class="menu-list") {
                    li {
                        a(id="menu:summary") {
                            span(class="icon") {
                                i(class="fa fa-home");
                            }
                            : "Summary"
                        }
                    }
                    @ for product in report.products.keys().sorted() {
                        @ if let Some(report) = report.products.get(&product) {
                            li {
                                : report.html_inline_menu_bar()
                            }
                        }
                    }
                    p(class="menu-label") {
                        a(href="https://github.com/georust/rinex/wiki", style="margin-left:29px") {
                            : "Documentation"
                        }
                    }
                    p(class="menu-label") {
                        a(href="https://github.com/georust/rinex/issues", style="margin-left:29px") {
                            : "Bug Report"
                        }
                    }
                    p(class="menu-label") {
                        a(href="https://github.com/georust/rinex") {
                            span(class="icon") {
                                i(class="fa-brands fa-github");
                            }
                            : "Sources"
                        }
                    }
                } // menu-list
            }//menu
        }
    }
}

impl RenderHtml for QcReport {
    fn to_inline_html(&self) -> Box<dyn RenderBox + '_> {
        box_html! {
        div(class="columns is-fullheight") {
            div(id="menubar", class="column is-3 is-sidebar-menu is-hidden-mobile") {
                : HtmlMenuBar::to_inline_html(&self)
            } // id=menubar
            div(class="hero is-fullheight") {
                div(id="summary", class="container is-main", style="display:block") {
                    div(class="section") {
                        : self.summary.to_inline_html()
                    }
                }//id=summary
                @ for product in self.products.keys().sorted() {
                    @ if let Some(report) = self.products.get(product) {
                        div(id=&html_id(product), class="container is-main", style="display:none") {
                            : report.to_inline_html()
                        }
                    }
                }
            }//class=hero
        } // class=columns
        script {
            :
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
            var i=1;
            var targetted_tab = category+':'+tab;
            do {
                if (main_pages[i -1].id == category) {
                    main_pages[i-1].style = 'display:block';
                } else {
                    main_pages[i-1].style = 'display:none';
                }
                i += 1;
            } while (i != main_pages.length);

        } else {
            var i=1;
            do {
                if (main_pages[i -1].id == category) {
                    main_pages[i-1].style = 'display:block';
                } else {
                    main_pages[i-1].style = 'display:none';
                }
                i += 1;
            } while (i != main_pages.length);
        }
    }"
            }
        }
    }
}
