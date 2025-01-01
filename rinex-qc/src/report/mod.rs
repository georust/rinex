//! Generic analysis report
use maud::{html, Markup, PreEscaped, Render, DOCTYPE};

// // shared analysis, that may apply to several products
// mod shared;

mod obs;
use obs::QcObservationsReport;

mod nav;
use nav::QcBrdcNavigationReport;

mod summary;

#[cfg(feature = "sp3")]
mod sp3;

#[cfg(feature = "sp3")]
use sp3::QcHighPrecisionNavigationReports;

pub(crate) mod shared;

use crate::{cfg::QcReportType, context::QcContext, report::summary::QcSummary};

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
    /// Summary report (always present)
    summary: QcSummary,
    /// Observations report
    observations: Option<QcObservationsReport>,
    /// BRDC Navigation report
    brdc_nav: Option<QcBrdcNavigationReport>,
    /// SP3 high precision report
    sp3_nav: Option<QcHighPrecisionNavigationReports>,
}

impl QcReport {
    /// Builds a new GNSS report, ready to be rendered
    pub fn new(ctx: &QcContext) -> Self {
        let summary_only = ctx.cfg.report.report_type == QcReportType::Summary;
        if summary_only {
            Self::summary_only(ctx)
        } else {
            Self {
                summary: QcSummary::new(ctx),
                observations: if ctx.has_observations() {
                    Some(QcObservationsReport::new(ctx))
                } else {
                    None
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
            }
        }
    }

    /// Generates a summary only report
    pub fn summary_only(ctx: &QcContext) -> Self {
        let summary = QcSummary::new(&ctx);
        Self {
            summary,
            observations: None,
            brdc_nav: None,
            #[cfg(feature = "sp3")]
            sp3_nav: None,
        }
    }

    // /// Add a custom chapter to the report
    // pub fn add_chapter(&mut self, chapter: QcExtraPage) {
    //     self.custom_chapters.push(chapter);
    // }

    /// Generates a menu bar to nagivate
    fn html_menu_bar(&self) -> Markup {
        html! {
            aside class="menu" {
                p class="menu-label" {
                    (format!("RINEx-Qc v{}", env!("CARGO_PKG_VERSION")))
                }
                ul class="menu-list" {
                    li {
                        a id="qc-summary" class="qc-sidemenu" onclick="onQcSummaryClicks()" {
                            span class="icon" {
                                i class="fa fa-home" {}
                            }
                            "Summary"
                        }
                    }
                    @ if self.observations.is_some() {
                        li {
                            a id="qc-observations" class="qc-sidemenu" onclick="onQcObservationsClicks()" {
                                span class="icon" {
                                    i class="fa-solid fa-tower-broadcast" {}
                                }
                                "Observations"
                            }
                        }
                    }
                    @ if self.brdc_nav.is_some() {
                        a id="qc-navigation" class="qc-sidemenu" onclick="onQcNavigationClicks()" {
                            span class="icon" {
                                i class="fa-solid fa-satellite-dish" {}
                            }
                            "Broadcast Navigation (BRDC)"
                        }
                    }
                    @ if self.sp3_nav.is_some() {
                        a id="qc-navigation" class="qc-sidemenu" onclick="onQcHighPrecisionOrbitsClicks()" {
                            span class="icon" {
                                i class="fa-solid fa-satellite-dish" {}
                            }
                            "High Precision Orbits (Sp3)"
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
                            "RINEX QC"
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
                                @ if let Some(obs) = &self.observations {
                                    div class="section" id="qc-observations" style="display:none" {
                                        div class="container is-main" {
                                            (obs.render())
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
