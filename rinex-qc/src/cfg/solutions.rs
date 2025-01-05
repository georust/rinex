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
                            "PPP"
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
                            "CGGTTS"
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
