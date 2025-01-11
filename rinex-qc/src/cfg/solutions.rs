use maud::{html, Markup, Render};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct QcSolutions {
    /// Automatically attach PPP solutions
    pub ppp: bool,
    /// Automatically attach CGGTTS solutions
    pub cggtts: bool,
}

impl QcSolutions {
    /// True when no solutions should be integrated
    pub(crate) fn is_empty(&self) -> bool {
        self.ppp == false && self.cggtts == false
    }

    /// True when at least one type of solutions should be integrated
    pub(crate) fn is_some(&self) -> bool {
        !self.is_empty()
    }
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
                                button aria-label="PPP solutions attached to this report"
                                data-balloon-pos="right" {
                                    span class="icon" {
                                        i class="fa-solid fa-circle-check" {}
                                    }
                                }
                            }
                        } @ else {
                            td {
                                button aria-label="PPP solutions not attached to this report"
                                data-balloon-pos="right" {
                                    span class="icon" {
                                        i class="fa-solid fa-circle-xmark" {}
                                    }
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
