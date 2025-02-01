use crate::analysis::solutions::QcNavPostSolutions;
use crate::prelude::{html, Markup, QcAnalysis, QcHtmlReporting};

mod ppp;

use maud::{PreEscaped, DOCTYPE};

// impl Render for QcNavPostSolutions {
//     fn render(&self) -> Markup {
//         html! {
//             table class="table is-bordered" {
//                 tbody {
//                     tr {
//                         th class="is-info" {
//                             "Solutions"
//                         }
//                         td {
//                             select id="qc-navpost-rovers" {
//                                 @ for rover in self.rovers() {
//                                     option value=(rover) {}
//                                 }
//                             }
//                         }
//                         td {
//                             select id="qc-navpost-solutions" {
//                                 @ if !self.ppp.is_empty() {
//                                     option value="PPP" {}
//                                 }
//                                 @ if !self.cggtts.is_empty() {
//                                     option value="CGGTTS" {}
//                                 }
//                             }
//                         }
//                     }
//                     @ for (rover, solutions) in &self.ppp {
//                         tr {
//                             th class="is-info" {
//                                 (format!("{} (PPP)", rover))
//                             }
//                             td {
//                                 (solutions.render())
//                             }
//                         }
//                     }
//                     @ for (rover, solutions) in &self.cggtts {
//                         tr {
//                             th class="is-info" {
//                                 (format!("{} (CGGTTS)", rover))
//                             }
//                             td {
//                                 (solutions.render())
//                             }
//                         }
//                     }
//                 }
//             }
//         }
//     }
// }

// impl Render for QcNavPostPPPSolutions {
//     fn render(&self) -> Markup {
//         html! {
//             div class="table-container" {
//                 table class="table is-bordered" {
//                     tbody {
//                         tr {
//                             th class="is-info" {
//                                 "Summary"
//                             }
//                             td {
//                                 (self.summary.render())
//                             }
//                         }
//                     }
//                 }
//             }
//         }
//     }
// }

// impl Render for Summary {
//     fn render(&self) -> Markup {
//         html! {
//             div class="table-container" {
//                 table class="table is-bordered" {
//                     tbody {
//                         tr {
//                             th class="is-info" {
//                                 "Time frame"
//                             }
//                             td {
//                                 tr {
//                                     td {
//                                         "First"
//                                     }
//                                     td {
//                                         "Last"
//                                     }
//                                 }
//                                 tr {
//                                     td {
//                                         (self.first_epoch.to_string())
//                                     }
//                                     td {
//                                         (self.first_epoch.to_string())
//                                     }
//                                 }
//                                 tr {
//                                     td {
//                                         "Duration"
//                                     }
//                                     td {
//                                         ((self.last_epoch - self.first_epoch).to_string())
//                                     }
//                                 }
//                             }
//                         }
//                     }
//                 }
//             }
//         }
//     }
// }
