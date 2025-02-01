use crate::prelude::{html, Markup, QcAnalysis, QcHtmlReporting};

#[cfg(feature = "html")]
use maud::{PreEscaped, DOCTYPE};

impl QcHtmlReporting for QcAnalysis {
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
                            @ if let Some(rover) = &self.rovers_analysis {
                                div class="hero is-fullheight" {
                                    div class="section" id="qc-rover-observations" style="display:block" {
                                        div class="container is-main" {
                                            (rover.render())
                                        }
                                    }
                                }
                            }
                            @ if let Some(base) = &self.base_stations_analysis {
                                div class="hero is-fullheight" {
                                    div class="section" id="qc-base-observations" style="display:block" {
                                        div class="container is-main" {
                                            (base.render())
                                        }
                                    }
                                }
                            }//class=hero
                            @ if cfg!(feature = "nav") {
                                @ if let Some(solutions) = &self.solutions {
                                    @ for (meta, nav_post_ppp_solutions) in solutions.ppp.iter() {
                                        div class="hero is-fullheight" {
                                            div class="section" id=(&format!("{}-ppp-solutions", meta.name)) style="display:block" {
                                                div class="container is-main" {
                                                    (nav_post_ppp_solutions.render())
                                                }
                                            }
                                        }
                                    }
                                }
                            }
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
