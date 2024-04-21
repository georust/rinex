//! RINEX Quality analysis library
//use strum_macros::EnumString;
use horrorshow::helper::doctype;
use horrorshow::html; // RenderBox};
use rinex_qc_traits::HtmlReport;

mod opts;
pub use opts::{QcClassification, QcOpts};

mod analysis;
use analysis::QcAnalysis;

mod context;
use context::DataContext;

pub mod prelude {
    pub use crate::context::DataContext;
}

#[derive(Debug, Clone, Copy)]
pub enum Grade {
    A,
    B,
    C,
    D,
    E,
    F,
}

/*
 * Methods used when reporting lenghty vectors or data subsets in a table.
 * Makes tables cleaner and nicer by wrapping string content, into several paragraphs.
 *
pub(crate) fn table_lengthy_td<A: std::fmt::Display>(
    list: &Vec<A>,
    max_items: usize,
) -> Box<dyn RenderBox + '_> {
    let mut content = String::with_capacity(64 * max_items);
    let mut paragraphs: Vec<String> = Vec::new();

    for i in 0..list.len() {
        content.push_str(&format!("{}, ", list[i]));
        if i.rem_euclid(max_items) == 0 {
            paragraphs.push(content.clone());
            content.clear();
        } else if i == list.len() - 1 {
            paragraphs.push(content.clone());
        }
    }
    box_html! {
        @ for paragraph in paragraphs {
            p {
                : paragraph.to_string()
            }
        }
    }
}
*/

use rinex::preprocessing::{MaskFilter, MaskOperand, Preprocessing, TargetItem};

pub struct QcReport {}

// impl QcReport {
//     fn build_analysis(ctx: &DataContext, opts: &QcOpts) -> Vec<QcAnalysis> {
//         /*
//          * QC analysis not feasible when Observations not provided
//          */
//         if !ctx.has_observation() {
//             return Vec::new();
//         }
//
//         let observation = ctx.observation().unwrap();
//
//         // build analysis to perform
//         let mut analysis: Vec<QcAnalysis> = Vec::new();
//         /*
//          * QC Classification:
//          *    the end user has the ability to sort the generated report per physics,
//          *    signals, or any other usual data subsets.
//          * To support that, we use the preprocessing toolkit, if available,
//          * first convert the classification method to a compatible object,
//          * so we can apply a mask filter
//          */
//         let mut filter_targets: Vec<TargetItem> = Vec::new();
//
//         match opts.classification {
//             QcClassification::GNSS => {
//                 for gnss in observation.constellation() {
//                     filter_targets.push(TargetItem::from(gnss));
//                 }
//             },
//             QcClassification::SV => {
//                 for sv in observation.sv() {
//                     filter_targets.push(TargetItem::from(sv));
//                 }
//             },
//             QcClassification::Physics => {
//                 let mut observables = observation.observable().cloned().collect::<Vec<_>>();
//                 observables.sort(); // improves report rendition
//                 for obsv in observables {
//                     filter_targets.push(TargetItem::from(obsv));
//                 }
//             },
//         }
//         // apply mask filters and generate an analysis on resulting data set
//         for target in filter_targets {
//             let mask = MaskFilter {
//                 item: target,
//                 operand: MaskOperand::Equals,
//             };
//
//             let subset = observation.filter(mask.clone().into());
//
//             // Perform analysis on all grouped subsets.
//             //  Improve this:
//             //   QcAnalysis::new() should construct from Context directly
//             //   and we should have grouped smaller contexts here
//             if let Some(brdc) = ctx.brdc_navigation() {
//                 let brdc = brdc.filter(mask.clone().into());
//
//                 // perform analysis on these subsets
//                 analysis.push(QcAnalysis::new(&subset, &Some(brdc), opts));
//             } else {
//                 // perform analysis on these subsets
//                 analysis.push(QcAnalysis::new(&subset, &None, opts));
//             }
//         }
//         analysis
//     }
//     /// Generates a Quality Check Report from provided Context and parametrization,
//     /// in html format.
//     pub fn html(context: &DataContext, opts: QcOpts) -> String {
//         let analysis = Self::build_analysis(context, &opts);
//         format!(
//             "{}",
//             html! {
//                 : doctype::HTML;
//                 html {
//                     head {
//                         meta(charset="UTF-8");
//                         meta(name="viewport", content="width=device-width, initial-scale=1");
//                         link(rel="stylesheet", href="https:////cdn.jsdelivr.net/npm/bulma@0.9.4/css/bulma.min.css");
//                         script(defer="true", src="https://use.fontawesome.com/releases/v5.3.1/js/all.js");
//                         title: context.name();
//                     }
//                     body {
//                         div(id="version") {
//                             h2(class="title") {
//                                 : "RINEX Quality Check summary"
//                             }
//                             table(class="table is-bordered; style=\"margin-bottom: 20px\"") {
//                                 tbody {
//                                     tr {
//                                         th {
//                                             : "Version"
//                                         }
//                                         td {
//                                             : format!("rinex-qc: v{}", env!("CARGO_PKG_VERSION"))
//                                         }
//                                     }
//                                 }
//                             }
//                         }//div=header
//                         div(id="context") {
//                             table(class="table is-bordered; style=\"margin-bottom: 20px\"") {
//                                 thead {
//                                     th {
//                                         : "Context"
//                                     }
//                                 }
//                                 tbody {
//                                     : context.to_inline_html()
//                                 }
//                             }
//                         }//div=context
//                         div(id="parameters") {
//                             table(class="table is-bordered; style=\"margin-bottom: 20px\"") {
//                                 thead {
//                                     th {
//                                         : "Parameters"
//                                     }
//                                 }
//                                 tbody {
//                                     : opts.to_inline_html()
//                                 }
//                             }
//                         } //div=parameters
//                         div(id="header") {
//                             table(class="table is-bordered; style=\"margin-bottom: 20px\"") {
//                                 thead {
//                                     th {
//                                         : "File Header"
//                                     }
//                                 }
//                                 @ if let Some(data) = context.rinex(ProductType::Observation) {
//                                     tbody {
//                                         : data.header.to_inline_html()
//                                     }
//                                 } else {
//                                     tbody {
//                                         : "Undefined"
//                                     }
//                                 }
//                             }
//                         }
//                         /*
//                          * Report all analysis that were performed
//                          */
//                         div(id="analysis") {
//                             /*
//                              * Report all analysis
//                              * and emphasize how they were sorted (self.opts.classfication)
//                              */
//                             @ for i in 0..analysis.len() {
//                                 table(class="table is-bordered; style=\"margin-bottom: 20px\"") {
//                                     thead {
//                                         @ if opts.classification == QcClassification::GNSS {
//                                             th {
//                                                 : format!("{:X} analysis", context
//                                                     .observation()
//                                                     .unwrap() // infaillible: QC needs observation RINEX
//                                                     .constellation().nth(i).unwrap())
//                                             }
//                                         } else if opts.classification == QcClassification::SV {
//                                             th {
//                                                 : format!("{:X} analysis", context
//                                                     .observation()
//                                                     .unwrap() // infaillible: QC needs observation RINEX
//                                                     .sv().nth(i).unwrap())
//                                             }
//
//                                         } else if opts.classification == QcClassification::Physics {
//                                             th {
//                                                 : format!("{} analysis", context
//                                                     .observation()
//                                                     .unwrap() // infaillible: QC needs observation RINEX
//                                                     .observable().nth(i).unwrap())
//                                             }
//                                         }
//                                     }
//                                     tbody {
//                                         : analysis[i].to_inline_html()
//                                     }
//                                 }
//                             }
//                         }//div=analysis
//                     }
//                 }
//             }
//         )
//     }
// }
