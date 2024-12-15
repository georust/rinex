//! Generic analysis report
use maud::{html, Markup, PreEscaped, Render, DOCTYPE};

// // shared analysis, that may apply to several products
// mod shared;

mod obs;
pub(crate) mod shared;
mod summary;

use crate::{
    cfg::{QcConfig, QcReportType},
    context::QcContext,
    report::{obs::QcObservationsReport, summary::QcSummary},
};

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
    /// Report Summary (always present)
    summary: QcSummary,
    /// Custom chapters (user created)
    custom_chapters: Vec<QcExtraPage>,
    /// One page per Observations
    observations: Option<QcObservationsReport>,
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
                custom_chapters: Vec::new(),
                observations: if ctx.has_observations() {
                    Some(QcObservationsReport::new(ctx))
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
            custom_chapters: Vec::new(),
            observations: Default::default(),
        }
    }

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

    /// Add a custom chapter to the report
    pub fn add_chapter(&mut self, chapter: QcExtraPage) {
        self.custom_chapters.push(chapter);
    }

    /// Generates a menu bar to nagivate
    fn html_menu_bar(&self) -> Markup {
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
                    @ if let Some(observations) = &self.observations {
                        li {
                            a id="menu:observations" {
                                span class="icon" {
                                    i class="fa-solid fa-tower-broadcast" {}
                                }
                                "Observations"
                            }
                            (observations.html_menu_bar())
                        }
                    }
                    @ if !self.custom_chapters.is_empty() {
                        li {
                            a id="menu:custom-chapters" {
                                ul class="menu-list" {
                                    @for chapter in self.custom_chapters.iter() {
                                        li {
                                            (chapter.tab.render())
                                        }
                                    }
                                }
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
                                    (self.html_menu_bar())
                                }
                                div class="hero is-fullheight" {
                                    div id="summary" class="container is-main" style="display:block" {
                                        div class="section" {
                                            (self.summary.render())
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
