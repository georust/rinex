//! Generic analysis report
use itertools::Itertools;
use maud::{html, Markup, PreEscaped, Render, DOCTYPE};
use std::collections::HashMap;
use thiserror::Error;

use crate::prelude::{ProductType, QcConfig, QcContext, QcReportType};

// shared analysis, that may apply to several products
mod shared;

mod summary;
use summary::QcSummary;

mod rinex;
use rinex::RINEXReport;

mod orbit;
use orbit::OrbitReport;

mod iono;
use iono::IonoReport;

#[cfg(feature = "sp3")]
mod sp3;

// preprocessed navi
// mod navi;
// use navi::QcNavi;

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

/// [QcExtraPage] you can add to customize [QcReport]
pub struct QcExtraPage {
    /// tab for pagination
    pub tab: Box<dyn Render>,
    /// content
    pub content: Box<dyn Render>,
    /// HTML id
    pub html_id: String,
}

/// [QcReport] is a generic structure to report complex analysis results
pub struct QcReport {
    /// Report Summary (always present)
    summary: QcSummary,
    // /// Preprocessed NAVI (only when compatible)
    // navi: Option<QcNavi>,
    /// Orbital projections (only when compatible)
    orbit: Option<OrbitReport>,
    /// Ionosphere Reporting (only when compatible)
    iono: Option<IonoReport>,
    /// One tab per input product.
    products: HashMap<ProductType, ProductReport>,
    /// Custom chapters
    custom_chapters: Vec<QcExtraPage>,
}

impl QcReport {
    /// Builds a new GNSS report, ready to be rendered
    pub fn new(context: &QcContext, cfg: QcConfig) -> Self {
        let ref_position = if let Some(position) = cfg.manual_reference {
            Some(position)
        } else {
            context.reference_position()
        };

        let summary = QcSummary::new(&context, &cfg);
        let summary_only = cfg.report == QcReportType::Summary;

        let iono_report = IonoReport::new(&context, &cfg);

        Self {
            custom_chapters: Vec::new(),
            // navi: {
            //    if summary.navi.nav_compatible && !summary_only {
            //        Some(QcNavi::new(context))
            //    } else {
            //        None
            //    }
            //},
            // Build the report, which comprises
            //   1. one general (high level) context tab
            //   2. one tab per product type (which can have sub tabs itself)
            //   3. one complex tab for "shared" analysis
            products: {
                let mut items = HashMap::<ProductType, ProductReport>::new();
                if !summary_only {
                    // one tab per RINEX product
                    for product_id in [
                        ProductType::Observation,
                        ProductType::DORIS,
                        ProductType::MeteoObservation,
                        ProductType::BroadcastNavigation,
                        ProductType::HighPrecisionClock,
                        ProductType::IONEX,
                        ProductType::ANTEX,
                    ] {
                        if let Some(rinex) = context.get_rinex_data(product_id) {
                            if let Ok(report) = RINEXReport::new(rinex) {
                                items.insert(product_id, ProductReport::RINEX(report));
                            }
                        }
                    }

                    // one tab for SP3 when supported
                    #[cfg(feature = "sp3")]
                    if let Some(sp3) = context.sp3_data() {
                        items.insert(
                            ProductType::HighPrecisionOrbit,
                            ProductReport::SP3(SP3Report::new(sp3)),
                        );
                    }
                }
                items
            },
            iono: iono_report,
            #[cfg(not(feature = "sp3"))]
            orbit: {
                if context.has_brdc_navigation() && !summary_only {
                    Some(OrbitReport::new(
                        context,
                        ref_position,
                        cfg.force_brdc_skyplot,
                    ))
                } else {
                    None
                }
            },
            #[cfg(feature = "sp3")]
            orbit: {
                if (context.has_sp3() || context.has_brdc_navigation()) && !summary_only {
                    Some(OrbitReport::new(
                        context,
                        ref_position,
                        cfg.force_brdc_skyplot,
                    ))
                } else {
                    None
                }
            },
            summary,
        }
    }
    /// Add a custom chapter to the report
    pub fn add_chapter(&mut self, chapter: QcExtraPage) {
        self.custom_chapters.push(chapter);
    }
    /// Generates a menu bar to nagivate [Self]
    #[cfg(not(feature = "sp3"))]
    fn menu_bar(&self) -> Markup {
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
                    @for product in self.products.keys().sorted() {
                        @if let Some(report) = self.products.get(&product) {
                            li {
                                (report.html_inline_menu_bar())
                            }
                        }
                    }
                    @for chapter in self.custom_chapters.iter() {
                        li {
                            (chapter.tab.render())
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
    /// Generates a menu bar to nagivate [Self]
    #[cfg(feature = "sp3")]
    fn menu_bar(&self) -> Markup {
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
                    @for product in self.products.keys().sorted() {
                        @if let Some(report) = self.products.get(&product) {
                            li {
                                (report.html_inline_menu_bar())
                            }
                        }
                    }
                    @if let Some(orbit) = &self.orbit {
                        li {
                            (orbit.html_inline_menu_bar())
                        }
                    }
                    @for chapter in self.custom_chapters.iter() {
                        li {
                            (chapter.tab.render())
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
                    link rel="stylesheet" href="https://unpkg.com/balloon-css/balloon.min.css";
                }//head
                body {
                        div id="title" {
                            title {
                                "RINEX QC"
                            }
                        }
                        div id="body" {
                            div class="columns is-fullheight" {
                                div id="menubar" class="column is-3 is-sidebar-menu is-hidden-mobile" {
                                    (self.menu_bar())
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
                                    // TODO: it should be feasible to run without SP3 support
                                    @if let Some(orbit) = &self.orbit {
                                        div id="orbit" class="container is-main" style="display:none" {
                                            (orbit.render())
                                        }
                                    }
                                    div id="extra-chapters" class="container" style="display:block" {
                                        @for chapter in self.custom_chapters.iter() {
                                            div id=(chapter.html_id) class="container is-main" style="display:none" {
                                                (chapter.content.render())
                                            }
                                            div id=(&format!("end:{}", chapter.html_id)) style="display:none" {
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

		sidebar_menu.onclick = function (evt) {
			var clicked_id = evt.originalTarget.id;
			var category = clicked_id.substring(5).split(':')[0];
			var tab = clicked_id.substring(5).split(':')[1];
			var is_tab = clicked_id.split(':').length == 3;
			var menu_subtabs = document.getElementsByClassName('menu:subtab');
			console.log('clicked id: ' + clicked_id + ' category: ' + category + ' tab: ' + is_tab);

			if (is_tab == true) {
				// show tabs for this category
				var cat_tabs = 'menu:'+category;
				for (var i = 0; i < menu_subtabs.length; i++) {
					if (menu_subtabs[i].id.startsWith(cat_tabs)) {
						menu_subtabs[i].style = 'display:block';
					} else {
						menu_subtabs[i].style = 'display:none';
					}
				}
				// hide any other main page
				for (var i = 0; i < main_pages.length; i++) {
					if (main_pages[i].id != category) {
						main_pages[i].style = 'display:none';
					}
				}
				// show specialized content
				var targetted_content = 'body:' + category + ':' + tab;
				for (var i = 0; i < sub_pages.length; i++) {
					if (sub_pages[i].id == targetted_content) {
						sub_pages[i].style = 'display:block';
					} else {
						sub_pages[i].style = 'display:none';
					}
				}
			} else {
				// show tabs for this category
				var cat_tabs = 'menu:'+category;
				for (var i = 0; i < menu_subtabs.length; i++) {
					if (menu_subtabs[i].id.startsWith(cat_tabs)) {
						menu_subtabs[i].style = 'display:block';
					} else {
						menu_subtabs[i].style = 'display:none';
					}
				}
				// hide any other main page
				for (var i = 0; i < main_pages.length; i++) {
					if (main_pages[i].id == category) {
						main_pages[i].style = 'display:block';
					} else {
						main_pages[i].style = 'display:none';
					}
				}
				// click on parent: show first specialized content
				var done = false;
				for (var i = 0; i < sub_pages.length; i++) {
					if (done == false) {
						if (sub_pages[i].id.includes('body:'+category)) {
							sub_pages[i].style = 'display:block';
							done = true;
						} else {
							sub_pages[i].style = 'display:none';
						}
					} else {
						sub_pages[i].style = 'display:none';
					}
				}
			}
		}
"
                        ))} //JS
                }//body
            }
        }
    }
}
