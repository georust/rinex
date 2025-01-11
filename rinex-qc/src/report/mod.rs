//! Generic analysis report
use maud::{html, Markup, PreEscaped, Render, DOCTYPE};

// // shared analysis, that may apply to several products
// mod shared;

use crate::context::meta::MetaData;

use std::collections::HashMap;

mod obs;
use obs::{QcBasesObservationsReport, QcRoversObservationsReport};

mod combinations;
use combinations::QcSignalCombinationsReport;

mod nav;
use nav::QcBrdcNavigationReport;

mod summary;
use summary::QcSummary;

#[cfg(feature = "nav")]
#[cfg_attr(docsrs, doc(cfg(feature = "nav")))]
mod solutions;

#[cfg(feature = "sp3")]
#[cfg_attr(docsrs, doc(cfg(feature = "sp3")))]
mod sp3;

#[cfg(feature = "nav")]
use solutions::QcNavPostSolutions;

#[cfg(feature = "sp3")]
use sp3::QcHighPrecisionNavigationReports;

pub(crate) mod shared;

use crate::{cfg::QcReportType, context::QcContext};

/// [QcExtraPage] you can add to customize [QcReport]
pub struct QcExtraPage {
    /// HTML id
    pub html_id: String,
    /// Menu for pagination
    pub menu: Box<dyn Render>,
    /// Page content
    pub content: Box<dyn Render>,
}

/// [QcReport] is a generic structure to report complex analysis results
pub struct QcReport {
    /// [QcSummary] report (always present)
    summary: QcSummary,

    /// "Rover" observations report
    rover_observations: Option<QcRoversObservationsReport>,

    /// "Base stations" observations report
    base_observations: Option<QcBasesObservationsReport>,

    /// possible signal combinations
    signal_combinations: HashMap<MetaData, QcSignalCombinationsReport>,

    /// Possible "BRDC" Navigation report
    brdc_nav: Option<QcBrdcNavigationReport>,

    /// SP3 (high precision orbital) report any time
    /// [SP3] present at synthesis time
    #[cfg(feature = "sp3")]
    #[cfg_attr(docsrs, doc(cfg(feature = "sp3")))]
    sp3_nav: Option<QcHighPrecisionNavigationReports>,

    /// Possibly attached [QcNavPostSolutions], depending on
    /// [QcConfig] applied at synthesis time.
    #[cfg(feature = "nav")]
    #[cfg_attr(docsrs, doc(cfg(feature = "nav")))]
    solutions: Option<QcNavPostSolutions>,
}

impl QcReport {
    /// Synthsize a complete GNSS report from the dataset.
    /// [QcReport] is then ready to be rendered in the desired format.
    pub fn new(ctx: &QcContext) -> Self {
        let has_solutions = ctx.cfg.solutions.is_some();
        let summary_only = ctx.cfg.report.report_type == QcReportType::Summary;

        if summary_only {
            Self::summary_only(ctx)
        } else {
            Self {
                summary: QcSummary::new(ctx),
                rover_observations: {
                    if ctx.has_rover_observations() {
                        Some(QcRoversObservationsReport::new(&ctx))
                    } else {
                        None
                    }
                },
                base_observations: {
                    if ctx.has_base_observations() {
                        Some(QcBasesObservationsReport::new(&ctx))
                    } else {
                        None
                    }
                },
                signal_combinations: if ctx.cfg.report.signal_combinations {
                    let mut map = HashMap::new();
                    for (obs_meta, rinex) in ctx.obs_dataset.iter() {
                        map.insert(
                            obs_meta.meta.clone(),
                            QcSignalCombinationsReport::new(&rinex),
                        );
                    }
                    map
                } else {
                    Default::default()
                },
                brdc_nav: if ctx.has_navigation_data() {
                    None
                } else {
                    None
                },
                #[cfg(feature = "sp3")]
                sp3_nav: if ctx.has_precise_orbits() {
                    Some(QcHighPrecisionNavigationReports::new(ctx))
                } else {
                    None
                },
                #[cfg(feature = "nav")]
                solutions: if has_solutions {
                    Some(QcNavPostSolutions::new(ctx))
                } else {
                    None
                },
            }
        }
    }

    /// Generates a summary report (shortest and quickest rendition).
    /// Use this to summarize and visualize what your dataset permits.
    pub fn summary_only(ctx: &QcContext) -> Self {
        let summary = QcSummary::new(&ctx);
        Self {
            summary,
            brdc_nav: None,
            base_observations: Default::default(),
            rover_observations: Default::default(),
            signal_combinations: Default::default(),
            #[cfg(feature = "sp3")]
            sp3_nav: None,
            #[cfg(feature = "nav")]
            solutions: None,
        }
    }

    /// Generates a menu bar to nagivate
    fn html_menu_bar(&self) -> Markup {
        html! {
            aside class="menu" {
                p class="menu-label" {
                    (format!("RINEx-Qc v{}", env!("CARGO_PKG_VERSION")))
                }
                ul class="menu-list" {
                    li {
                        a class="qc-sidemenu" onclick="onQcSummaryClicks()" {
                            span class="icon" {
                                i class="fa fa-home" {}
                            }
                            "Summary"
                        }
                    }
                    @ if let Some(rovers) = &self.rover_observations {
                        @ if rovers.reports.len() == 1 {
                            li {
                                a class="qc-sidemenu" onclick="onQcRoverObservationsClicks()" {
                                    span class="icon" {
                                        i class="fa-solid fa-tower-broadcast" {}
                                    }
                                    "Rover"
                                }
                            }
                        } @ else {
                            li {
                                a class="qc-sidemenu" onclick="onQcRoverObservationsClicks()" {
                                    span class="icon" {
                                        i class="fa-solid fa-tower-broadcast" {}
                                    }
                                    "Rovers"
                                }
                            }

                        }
                    }
                    @ if let Some(bases) = &self.base_observations {
                        @ if bases.reports.len() == 1 {
                            li {
                                a class="qc-sidemenu" onclick="onQcBaseObservationsClicks()" {
                                    span class="icon" {
                                        i class="fa-solid fa-tower-broadcast" {}
                                    }
                                    "Base station"
                                }
                            }

                        } else {
                            li {
                                a class="qc-sidemenu" onclick="onQcBaseObservationsClicks()" {
                                    span class="icon" {
                                        i class="fa-solid fa-tower-broadcast" {}
                                    }
                                    "Base stations"
                                }
                            }
                        }
                    }
                    @ if !self.signal_combinations.is_empty() {
                        li {
                            a class="qc-sidemenu" onclick="onQcSignalCombinationsClicks()" {
                                span class="icon" {
                                    i class="fa-solid fa-tower-broadcast" {}
                                }
                                "Signal Combinations"
                            }
                        }
                    }

                    @ if self.brdc_nav.is_some() {
                        a class="qc-sidemenu" onclick="onQcNavigationClicks()" {
                            span class="icon" {
                                i class="fa-solid fa-satellite-dish" {}
                            }
                            "Broadcast Navigation (BRDC)"
                        }
                    }

                    @ if cfg!(feature = "sp3") {
                        @ if self.sp3_nav.is_some() {
                            a class="qc-sidemenu" onclick="onQcHighPrecisionOrbitsClicks()" {
                                span class="icon" {
                                    i class="fa-solid fa-satellite-dish" {}
                                }
                                "High Precision Orbits (SP3)"
                            }
                        }
                    }

                    @ if cfg!(feature = "nav") {
                        @ if self.solutions.is_some() {
                            a class="qc-sidemenu" onclick="onQcNavSolutionsClicks()" {
                                span class="icon" {
                                    i class="fa-solid fa-location-crosshairs" {}
                                }
                                "Solutions"
                            }
                        }
                    }

                    p class="menu-label" {
                        a href="https://github.com/georust/rinex/wiki" style="margin-left:29px" {
                            "Wiki"
                        }
                    }
                    p class="menu-label" {
                        a href="https://github.com/georust/rinex/tree/main/tutorials" style="margin-left:29px" {
                            "Tutorials"
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
                    script src="/hdd/git/rinex/rinex-qc/web/rinex-qc.js";
                    script defer="true" src="https://use.fontawesome.com/releases/v5.3.1/js/all.js" {};
                    script src="https://cdn.jsdelivr.net/npm/mathjax@3.2.2/es5/tex-svg.js" {};
                    link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/bulma@1.0.0/css/bulma.min.css";
                    link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.5.2/css/all.min.css";
                    link rel="stylesheet" href="https://unpkg.com/balloon-css/balloon.min.css";
                    link rel="stylesheet" href="/hdd/git/rinex/rinex-qc/web/rinex-qc.css";
                }//head
                body {
                    div id="title" {
                        title {
                            "RINEX Qc"
                        }
                    }
                    div id="body" {
                        div class="columns is-fullheight" {
                            div class="column is-3 is-sidebar-menu is-hidden-mobile" {
                                (self.html_menu_bar())
                            }
                            div class="hero is-fullheight" {
                                div class="section" id="qc-summary" style="display:block" {
                                    div class="container is-main" {
                                        (self.summary.render())
                                    }
                                }
                            }
                            @ if let Some(rover) = &self.rover_observations {
                                div class="hero is-fullheight" {
                                    div class="section" id="qc-rover-observations" style="display:block" {
                                        div class="container is-main" {
                                            (rover.render())
                                        }
                                    }
                                }
                            }
                            @ if let Some(base) = &self.base_observations {
                                div class="hero is-fullheight" {
                                    div class="section" id="qc-base-observations" style="display:block" {
                                        div class="container is-main" {
                                            (base.render())
                                        }
                                    }
                                }
                            }//class=hero
                        } // class=columns
                    }
                    // minimum JS required
                    script {
                        (PreEscaped(
                            "buildPageListeners();"
                        ))
                    }
                }
            }
        }
    }
}
