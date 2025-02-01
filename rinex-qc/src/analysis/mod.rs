//! Generic analysis report
use maud::{html, Markup};

// // shared analysis, that may apply to several products
// mod shared;

pub mod obs;
use obs::{QcBasesObservationsAnalysis, QcRoversObservationAnalysis};

pub mod summary;
use summary::QcSummary;

#[cfg(feature = "nav")]
#[cfg_attr(docsrs, doc(cfg(feature = "nav")))]
pub mod solutions;

// #[cfg(feature = "sp3")]
// #[cfg_attr(docsrs, doc(cfg(feature = "sp3")))]
// mod sp3;

#[cfg(feature = "nav")]
use solutions::QcNavPostSolutions;

// #[cfg(feature = "sp3")]
// use sp3::QcHighPrecisionNavigationReports;

pub(crate) mod shared;

use crate::{cfg::QcReportType, context::QcContext};

/// [QcAnalysis] is the overall and generic analysis that results
/// from the complex analysis of the entire [QcContext].
pub struct QcAnalysis {
    /// [QcSummary] report (always present)
    pub(crate) summary: QcSummary,

    /// "Rover(s)" analysis
    pub(crate) rovers_analysis: Option<QcRoversObservationAnalysis>,

    /// Base stations analysis
    pub(crate) base_stations_analysis: Option<QcBasesObservationsAnalysis>,
    // /// Possible "BRDC" Navigation report
    // brdc_nav: Option<QcBrdcNavigationReport>,

    // /// SP3 (high precision orbital) report any time
    // /// [SP3] present at synthesis time
    // #[cfg(feature = "sp3")]
    // #[cfg_attr(docsrs, doc(cfg(feature = "sp3")))]
    // sp3_nav: Option<QcHighPrecisionNavigationReports>,
    /// Possibly attached [QcNavPostSolutions], depending on
    /// [QcConfig] applied at synthesis time.
    #[cfg(feature = "nav")]
    #[cfg_attr(docsrs, doc(cfg(feature = "nav")))]
    pub(crate) solutions: Option<QcNavPostSolutions>,
}

impl QcAnalysis {
    /// Synthsize a complete GNSS report from the dataset.
    /// [QcAnalysis] is then ready to be rendered in the desired format.
    pub fn new(ctx: &QcContext) -> Self {
        // special case: summary report only
        let summary_only = ctx.cfg.report.report_type == QcReportType::Summary;
        if summary_only {
            return Self::summary_only(ctx);
        }

        // auto integrated nav solutions
        let has_solutions = ctx.cfg.solutions.is_some();

        Self {
            summary: QcSummary::new(ctx),

            rovers_analysis: {
                if ctx.has_rover_observations() {
                    Some(QcRoversObservationAnalysis::new(&ctx))
                } else {
                    None
                }
            },

            base_stations_analysis: {
                if ctx.has_base_observations() {
                    Some(QcBasesObservationsAnalysis::new(&ctx))
                } else {
                    None
                }
            },
            // brdc_nav: if ctx.has_navigation_data() {
            //     None
            // } else {
            //     None
            // },

            // #[cfg(feature = "sp3")]
            // sp3_nav: if ctx.has_precise_orbits() {
            //     Some(QcHighPrecisionNavigationReports::new(ctx))
            // } else {
            //     None
            // },
            #[cfg(feature = "nav")]
            solutions: if has_solutions {
                Some(QcNavPostSolutions::new(ctx))
            } else {
                None
            },
        }
    }

    /// Generates a summary report (shortest and quickest rendition).
    /// Use this to summarize and visualize what your dataset permits.
    pub fn summary_only(ctx: &QcContext) -> Self {
        let summary = QcSummary::new(&ctx);
        Self {
            summary,
            base_stations_analysis: Default::default(),
            rovers_analysis: Default::default(),
            #[cfg(feature = "nav")]
            solutions: None,
        }
    }

    /// Generates a menu bar to nagivate
    pub(crate) fn html_menu_bar(&self) -> Markup {
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
                    @ if let Some(rovers) = &self.rovers_analysis {
                        @ if rovers.analysis.len() == 1 {
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
                    @ if let Some(bases) = &self.base_stations_analysis {
                        @ if bases.analysis.len() == 1 {
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


                    // @ if self.brdc_nav.is_some() {
                    //     a class="qc-sidemenu" onclick="onQcNavigationClicks()" {
                    //         span class="icon" {
                    //             i class="fa-solid fa-satellite-dish" {}
                    //         }
                    //         "Broadcast Navigation (BRDC)"
                    //     }
                    // }

                    // @ if cfg!(feature = "sp3") {
                    //     @ if self.sp3_nav.is_some() {
                    //         a class="qc-sidemenu" onclick="onQcHighPrecisionOrbitsClicks()" {
                    //             span class="icon" {
                    //                 i class="fa-solid fa-satellite-dish" {}
                    //             }
                    //             "High Precision Orbits (SP3)"
                    //         }
                    //     }
                    // }

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
