//! RINEX Quality analysis library
//use strum_macros::EnumString;
use horrorshow::helper::doctype;
use horrorshow::html; // RenderBox};
use rinex_qc_traits::HtmlReport;

mod opts;
pub use opts::{QcClassification, QcOpts};

mod analysis;
use analysis::QcAnalysis;

use rinex::prelude::RnxContext;

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

impl QcReport {
    fn build_analysis(ctx: &RnxContext, opts: &QcOpts) -> Vec<QcAnalysis> {
        // build analysis to perform
        let mut analysis: Vec<QcAnalysis> = Vec::new();
        /*
         * QC Classification:
         *    the end user has the ability to sort the generated report per physics,
         *    signals, or any other usual data subsets.
         * To support that, we use the preprocessing toolkit, if available,
         * first convert the classification method to a compatible object,
         * so we can apply a mask filter
         */
        let mut filter_targets: Vec<TargetItem> = Vec::new();

        match opts.classification {
            QcClassification::GNSS => {
                for gnss in ctx.primary_data().constellation() {
                    filter_targets.push(TargetItem::from(gnss));
                }
            },
            QcClassification::SV => {
                for sv in ctx.primary_data().sv() {
                    filter_targets.push(TargetItem::from(sv));
                }
            },
            QcClassification::Physics => {
                let mut observables: Vec<_> = ctx.primary_data().observable().cloned().collect();
                observables.sort(); // improves report rendering
                for obsv in observables {
                    filter_targets.push(TargetItem::from(obsv));
                }
            },
        }
        // apply mask filters and generate an analysis on resulting data set
        for target in filter_targets {
            let mask = MaskFilter {
                item: target,
                operand: MaskOperand::Equals,
            };

            let subset = ctx.primary_data().filter(mask.clone().into());

            // also apply to possible NAV augmentation
            let nav_subset = ctx
                .navigation_data()
                .as_ref()
                .map(|nav| nav.filter(mask.clone().into()));

            // perform analysis on these subsets
            analysis.push(QcAnalysis::new(&subset, &nav_subset, opts));
        }
        analysis
    }
    /// Generates a Quality Check Report from provided Context and parametrization,
    /// in html format.
    pub fn html(context: &RnxContext, opts: QcOpts) -> String {
        let analysis = Self::build_analysis(context, &opts);
        format!(
            "{}",
            html! {
                : doctype::HTML;
                html {
                    head {
                        meta(charset="UTF-8");
                        meta(name="viewport", content="width=device-width, initial-scale=1");
                        link(rel="stylesheet", href="https:////cdn.jsdelivr.net/npm/bulma@0.9.4/css/bulma.min.css");
                        script(defer="true", src="https://use.fontawesome.com/releases/v5.3.1/js/all.js");
                        title:
                            format!("{:?}", context.primary.paths.iter()
                                .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
                                .collect::<Vec<String>>());
                    }
                    body {
                        div(id="version") {
                            h2(class="title") {
                                : "RINEX Quality Check summary"
                            }
                            table(class="table is-bordered; style=\"margin-bottom: 20px\"") {
                                tbody {
                                    tr {
                                        th {
                                            : "Version"
                                        }
                                        td {
                                            : format!("rinex-qc: v{}", env!("CARGO_PKG_VERSION"))
                                        }
                                    }
                                }
                            }
                        }//div=header
                        div(id="context") {
                            table(class="table is-bordered; style=\"margin-bottom: 20px\"") {
                                thead {
                                    th {
                                        : "Context"
                                    }
                                }
                                tbody {
                                    : context.to_inline_html()
                                }
                            }
                        }//div=context
                        div(id="parameters") {
                            table(class="table is-bordered; style=\"margin-bottom: 20px\"") {
                                thead {
                                    th {
                                        : "Parameters"
                                    }
                                }
                                tbody {
                                    : opts.to_inline_html()
                                }
                            }
                        } //div=parameters
                        div(id="header") {
                            table(class="table is-bordered; style=\"margin-bottom: 20px\"") {
                                thead {
                                    th {
                                        : "File Header"
                                    }
                                }
                                tbody {
                                    : context.primary_data().header.to_inline_html()
                                }
                            }
                        }
                        /*
                         * Report all analysis that were performed
                         */
                        div(id="analysis") {
                            /*
                             * Report all analysis
                             * and emphasize how they were sorted (self.opts.classfication)
                             */
                            @ for i in 0..analysis.len() {
                                table(class="table is-bordered; style=\"margin-bottom: 20px\"") {
                                    thead {
                                        @ if opts.classification == QcClassification::GNSS {
                                            th {
                                                : format!("{:X} analysis", context.primary_data().constellation().nth(i).unwrap())
                                            }
                                        } else if opts.classification == QcClassification::SV {
                                            th {
                                                : format!("{:X} analysis", context.primary_data().sv().nth(i).unwrap())
                                            }

                                        } else if opts.classification == QcClassification::Physics {
                                            th {
                                                : format!("{} analysis", context.primary_data().observable().nth(i).unwrap())
                                            }
                                        }
                                    }
                                    tbody {
                                        : analysis[i].to_inline_html()
                                    }
                                }
                            }
                        }//div=analysis
                    }
                }
            }
        )
    }
}
