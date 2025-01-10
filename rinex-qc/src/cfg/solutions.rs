use maud::{html, Markup, Render};
use serde::{Deserialize, Serialize};

use crate::cfg::QcConfigError;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct QcSolutions {
    pub ppp: bool,
    pub cggtts: bool,
}

impl Render for QcSolutions {
    fn render(&self) -> Markup {
        html! {
            div class="table-container" {
                table class="table is-bordered" {
                    tr {
                        th class="is-info" {
                            @ if self.ppp {
                                button aria-label="PPP solutions attached to this report"
                                data-balloon-pos="right" {
                                    "PPP"
                                }
                            } @ else {
                                button aria-label="PPP solutions not attached to this report"
                                data-balloon-pos="right" {
                                    "PPP"
                                }
                            }
                        }
                        @ if self.ppp {
                            td {
                                span class="icon" {
                                    i class="fa-solid fa-circle-check" {}
                                }
                            }
                        } @ else {
                            td {
                                span class="icon" {
                                    i class="fa-solid fa-circle-xmark" {}
                                }
                            }
                        }
                    }
                    tr {
                        th class="is-info" {
                            @ if self.cggtts {
                                button aria-label="CGGTTS solutions attached to this report"
                                data-balloon-pos="right" {
                                    "CGGTTS"
                                }
                            } @ else {
                                button aria-label="CGGTTS solutions not attached to this report"
                                data-balloon-pos="right" {
                                    "CGGTTS"
                                }
                            }
                        }
                        @ if self.cggtts {
                            td {
                                span class="icon" {
                                    i class="fa-solid fa-circle-check" {}
                                }
                            }
                        } @ else {
                            td {
                                span class="icon" {
                                    i class="fa-solid fa-circle-xmark" {}
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
