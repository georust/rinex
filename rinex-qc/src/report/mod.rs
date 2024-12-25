//! Generic analysis report
use maud::{html, Markup, PreEscaped, Render, DOCTYPE};

// // shared analysis, that may apply to several products
// mod shared;

mod obs;
mod summary;

pub(crate) mod shared;

use crate::{cfg::QcReportType, context::QcContext, report::summary::QcSummary};

// mod rinex;
// use rinex::RINEXReport;

// mod orbit;
// use orbit::OrbitReport;

// mod iono;
// use iono::IonoReport;

// #[cfg(feature = "sp3")]
// mod sp3;

// preprocessed navi
// mod navi;
// use navi::QcNavi;

// #[cfg(feature = "sp3")]
// use sp3::SP3Report;

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
    // /// Custom chapters (user created)
    // custom_chapters: Vec<QcExtraPage>,
    // /// One page per Observations
    // observations: Option<QcObservationsReport>,
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
            }
        }
    }

    /// Generates a summary only report
    pub fn summary_only(ctx: &QcContext) -> Self {
        let summary = QcSummary::new(&ctx);
        Self { summary }
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
                    (format!("RINEX-Qc v{}", env!("CARGO_PKG_VERSION")))
                }
                ul class="menu-list" {
                    li {
                        a id="qc-summary" class="qc-sidemenu" {
                            span class="icon" {
                                i class="fa fa-home" {}
                            }
                            "Summary"
                        }
                        ul class="menu-list" {
                            li {
                                a id="qc-compliancy" class="qc-sidemenu" {
                                    "Compliancy"
                                }
                            }
                        }
                    }
                    // @ if let Some(observations) = &self.observations {
                    //     li {
                    //         a id="qc-observations" class="qc-sidemenu" {
                    //             span class="icon" {
                    //                 i class="fa-solid fa-tower-broadcast" {}
                    //             }
                    //             "Observations"
                    //         }
                    //     }
                    // }
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
