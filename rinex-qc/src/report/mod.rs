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
    #[error("missing Clock RINEX header")]
    MissingClockHeader,
    #[error("missing Meteo RINEX header")]
    MissingMeteoHeader,
}

enum ProductReport {
    /// RINEX products report
    RINEX(RINEXReport),
    #[cfg(feature = "sp3")]
    /// SP3 product report
    SP3(SP3Report),
}

fn html_id(product: &ProductType) -> &str {
    match product {
        ProductType::IONEX => "ionex",
        ProductType::DORIS => "doris",
        ProductType::ANTEX => "antex",
        ProductType::Observation => "observations",
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

impl RenderHtml for QcReport {
    fn to_inline_html(&self) -> Box<dyn RenderBox + '_> {
        box_html! {
        div(class="columns is-fullheight") {
            div(id="menubar", class="column is-3 is-sidebar-menu is-hidden-mobile") {
                aside(class="menu") {
                    p(class="menu-label") {
                        : &format!("RINEX-QC v{}", env!("CARGO_PKG_VERSION"))
                    }
                    ul(class="menu-list") {
                        li {
                            a(id="summary") {
                                span(class="icon") {
                                    i(class="fa fa-home");
                                }
                                : "Summary"
                            }
                        }
                        @ for product in self.products.keys().sorted() {
                            @ if let Some(report) = self.products.get(&product) {
                                @ if *product == ProductType::Observation {
                                    li {
                                        a(id="observations") {
                                            span(class="icon") {
                                                i(class="fa-solid fa-tower-cell");
                                            }
                                            : "Observations"
                                        }
                                    }
                                    ul(class="menu-list", id="nested:observations", style="display:none") {
                                        @ for constell in report
                                            .as_rinex()
                                            .unwrap()
                                            .as_obs()
                                            .unwrap()
                                            .pages
                                            .keys()
                                        {
                                            li {
                                                a(id=&format!("nested:obs:{}", constell), style="margin-left:29px") {
                                                    span(class="icon") {
                                                        i(class="fa-solid fa-satellite");
                                                    }
                                                    : constell.to_string()
                                                }
                                            }
                                        }
                                    }
                                } else if *product == ProductType::BroadcastNavigation {
                                    li {
                                        a(id="brdc") {
                                            span(class="icon") {
                                                i(class="fa-solid fa-satellite-dish");
                                            }
                                            : "Broadcast Navigation (BRDC)"
                                        }
                                    }
                                    ul(class="menu-list", id="nested:brdc", style="display:none") {
                                        @ for constell in report
                                            .as_rinex()
                                            .unwrap()
                                            .as_nav()
                                            .unwrap()
                                            .pages
                                            .keys()
                                        {
                                            li {
                                                a(id=&format!("nested:brdc:{}", constell)) {
                                                    span(class="icon") {
                                                        i(class="fa-solid fa-satellite");
                                                    }
                                                    : constell.to_string()
                                                }
                                            }
                                        }
                                    }
                                } else if *product == ProductType::HighPrecisionOrbit {
                                    li {
                                        a(id="sp3") {
                                            span(class="icon") {
                                                i(class="fa-solid fa-satellite");
                                            }
                                            : "High Precision Orbit (SP3)"
                                        }
                                    }
                                    ul(class="menu-list", id="nested:sp3", style="display:none") {
                                        @ for constell in report
                                            .as_sp3()
                                            .unwrap()
                                            .pages
                                            .keys()
                                        {
                                            li {
                                                a(id=&format!("nested:sp3:{}", constell)) {
                                                    span(class="icon") {
                                                        i(class="fa-solid fa-satellite");
                                                    }
                                                    : constell.to_string()
                                                }
                                            }
                                        }
                                    }
                                } else if *product == ProductType::HighPrecisionClock {
                                    li {
                                        a(id="clk") {
                                            span(class="icon") {
                                                i(class="fa-solid fa-clock");
                                            }
                                            : "High Precision Clock (RINEX)"
                                        }
                                    }
                                    ul(class="menu-list", id="nested:clk", style="display:none") {
                                        @ for constell in report
                                            .as_rinex()
                                            .unwrap()
                                            .as_clk()
                                            .unwrap()
                                            .pages
                                            .keys()
                                        {
                                            li {
                                                a(id=&format!("nested:clk:{}", constell)) {
                                                    span(class="icon") {
                                                        i(class="fa-solid fa-satellite");
                                                    }
                                                    : constell.to_string()
                                                }
                                            }
                                        }
                                    }
                                } else if *product == ProductType::MeteoObservation {
                                    li {
                                        a(id="meteo") {
                                            span(class="icon") {
                                                i(class="fa-solid fa-cloud-sun-rain");
                                            }
                                            : "Meteo Observations"
                                        }
                                    }
                                } else if *product == ProductType::IONEX {
                                    li {
                                        a(id="ionex") {
                                            span(class="icon") {
                                                i(class="fa-solid fa-earth-americas");
                                            }
                                            : "Ionosphere Maps (IONEX)"
                                        }
                                    }
                                }
                            }
                        } // for products..
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
                    } //class=menu-list
                } //class=menu
            } // id=menubar
            div(class="hero is-fullheight") {
                div(id="hero:summary", class="container", style="display:block") {
                    div(class="section") {
                        : self.summary.to_inline_html()
                    }
                }//id=summary
                @ for (product, report) in self.products.iter() {
                    div(id=&format!("hero:{}", html_id(product)), class="container", style="display:none") {
                        div(class="section") {
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
    var sidebar_about = document.getElementById('hero:about');
    var summary_report = document.getElementById('hero:summary');
    var obs_report = document.getElementById('hero:observations');
    var brdc_report = document.getElementById('hero:brdc');
    var meteo_report = document.getElementById('hero:meteo');
    var sp3_report = document.getElementById('hero:sp3');
    var clk_report = document.getElementById('hero:clk');
    var ionex_report = document.getElementById('hero:ionex');
    var nested_sp3 = document.getElementById('nested:sp3');
    var nested_clk = document.getElementById('nested:clk');
    var nested_brdc = document.getElementById('nested:brdc');
    var nested_obs = document.getElementById('nested:observations');

    sidebar_menu.onclick = function(evt) {{
        var target_id = evt.originalTarget.id;
        console.log('clicked id: ' + target_id);
        
        if (target_id == 'summary') {{
            summary_report.style = 'display:block';
            if (obs_report != null) {{
                obs_report.style = 'display:none';
            }}
            if (brdc_report != null) {{
                brdc_report.style = 'display:none';
            }}
            if (ionex_report != null) {{
                ionex_report.style = 'display:none';
            }}
            if (sp3_report != null) {{
                sp3_report.style = 'display:none';
            }}
            if (meteo_report != null) {{
                meteo_report.style = 'display:none';
            }}
            if (clk_report != null) {{
                clk_report.style = 'display:none';
            }}
            if (nested_sp3 != null) {{
                nested_sp3.style = 'display:none';
            }}
            if (nested_clk != null) {{
                nested_clk.style = 'display:none';
            }}
            if (nested_brdc != null) {{
                nested_brdc.style = 'display:none';
            }}
            if (nested_obs != null) {{
                nested_obs.style = 'display:none';
            }}
            
        }} else if (target_id == 'observations') {{
            summary_report.style = 'display:none';
            if (obs_report != null) {{
                obs_report.style = 'display:block';
            }}
            if (brdc_report != null) {{
                brdc_report.style = 'display:none';
            }}
            if (ionex_report != null) {{
                ionex_report.style = 'display:none';
            }}
            if (sp3_report != null) {{
                sp3_report.style = 'display:none';
            }}
            if (meteo_report != null) {{
                meteo_report.style = 'display:none';
            }}
            if (clk_report != null) {{
                clk_report.style = 'display:none';
            }}
            if (nested_sp3 != null) {{
                nested_sp3.style = 'display:none';
            }}
            if (nested_clk != null) {{
                nested_clk.style = 'display:none';
            }}
            if (nested_brdc != null) {{
                nested_brdc.style = 'display:none';
            }}
            if (nested_obs != null) {{
                nested_obs.style = 'display:block';
            }}
            
        }} else if (target_id == 'sp3') {{
            summary_report.style = 'display:none';
            if (obs_report != null) {{
                obs_report.style = 'display:none';
            }}
            if (brdc_report != null) {{
                brdc_report.style = 'display:none';
            }}
            if (ionex_report != null) {{
                ionex_report.style = 'display:none';
            }}
            if (sp3_report != null) {{
                sp3_report.style = 'display:block';
            }}
            if (meteo_report != null) {{
                meteo_report.style = 'display:none';
            }}
            if (clk_report != null) {{
                clk_report.style = 'display:none';
            }}
            if (nested_sp3 != null) {{
                nested_sp3.style = 'display:block';
            }}
            if (nested_clk != null) {{
                nested_clk.style = 'display:none';
            }}
            if (nested_brdc != null) {{
                nested_brdc.style = 'display:none';
            }}
            if (nested_obs != null) {{
                nested_obs.style = 'display:none';
            }}
            
        }} else if (target_id == 'meteo') {{
            summary_report.style = 'display:none';
            if (obs_report != null) {{
                obs_report.style = 'display:none';
            }}
            if (brdc_report != null) {{
                brdc_report.style = 'display:none';
            }}
            if (ionex_report != null) {{
                ionex_report.style = 'display:none';
            }}
            if (sp3_report != null) {{
                sp3_report.style = 'display:none';
            }}
            if (meteo_report != null) {{
                meteo_report.style = 'display:block';
            }}
            if (clk_report != null) {{
                clk_report.style = 'display:none';
            }}
            if (nested_sp3 != null) {{
                nested_sp3.style = 'display:none';
            }}
            if (nested_clk != null) {{
                nested_clk.style = 'display:none';
            }}
            if (nested_brdc != null) {{
                nested_brdc.style = 'display:none';
            }}
            if (nested_obs != null) {{
                nested_obs.style = 'display:none';
            }}
            
        }} else if (target_id == 'clk') {{
            summary_report.style = 'display:none';
            if (obs_report != null) {{
                obs_report.style = 'display:none';
            }}
            if (brdc_report != null) {{
                brdc_report.style = 'display:none';
            }}
            if (ionex_report != null) {{
                ionex_report.style = 'display:none';
            }}
            if (sp3_report != null) {{
                sp3_report.style = 'display:none';
            }}
            if (meteo_report != null) {{
                meteo_report.style = 'display:none';
            }}
            if (clk_report != null) {{
                clk_report.style = 'display:block';
            }}
            if (nested_sp3 != null) {{
                nested_sp3.style = 'display:none';
            }}
            if (nested_clk != null) {{
                nested_clk.style = 'display:block';
            }}
            if (nested_brdc != null) {{
                nested_brdc.style = 'display:none';
            }}
            if (nested_obs != null) {{
                nested_obs.style = 'display:none';
            }}
            
        }}
    }}"
            }
        }
    }
}
