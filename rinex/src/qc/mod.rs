use horrorshow::{helper::doctype, RenderBox};
use strum_macros::EnumString;

mod context;
pub use context::{QcContext, QcInputData};

mod opts;
pub use opts::{QcClassification, QcOpts};

mod analysis;
use analysis::QcAnalysis;

#[cfg(feature = "processing")]
use crate::preprocessing::*;

/*
 * Array (CSV) pretty formatter
 */
pub(crate) fn pretty_array<A: std::fmt::Display>(list: &Vec<A>) -> String {
    let mut s = String::with_capacity(8 * list.len());
    for index in 0..list.len() - 1 {
        s.push_str(&format!("{}, ", list[index]));
    }
    s.push_str(&list[list.len() - 1].to_string());
    s
}

pub trait HtmlReport {
    /// Renders self to HTML
    fn to_html(&self) -> String;
    /// Renders self to embedded HTML
    fn to_inline_html(&self) -> Box<dyn RenderBox + '_>;
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, EnumString)]
pub enum Grade {
    #[strum(serialize = "A++")]
    GradeApp,
    #[strum(serialize = "A+")]
    GradeAp,
    #[strum(serialize = "A")]
    GradeA,
    #[strum(serialize = "B")]
    GradeB,
    #[strum(serialize = "C")]
    GradeC,
    #[strum(serialize = "D")]
    GradeD,
    #[strum(serialize = "E")]
    GradeE,
    #[strum(serialize = "F")]
    GradeF,
}

pub struct QcReport {}

impl QcReport {
    #[cfg(not(feature = "processing"))]
    fn build_analysis(ctx: &QcContext, opts: &QcOpts) -> Vec<QcAnalysis> {
        vec![QcAnalysis::new(ctx, opts)]
    }
    /*
     * When the "processing" feature is enabled,
     * we can sort the report by desired physics or other criteria
     */
    #[cfg(feature = "processing")]
    fn build_analysis(ctx: &QcContext, opts: &QcOpts) -> Vec<QcAnalysis> {
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
            QcClassification::Sv => {
                for sv in ctx.primary_data().sv() {
                    filter_targets.push(TargetItem::from(sv));
                }
            },
            QcClassification::Physics => {
                let mut observables: Vec<_> =
                    ctx.primary_data().observable().map(|o| o.clone()).collect();
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

            let subset = ctx.primary.rinex.filter(mask.clone().into());

            // also apply to possible NAV augmentation
            let nav_subset = if let Some(nav) = &ctx.nav {
                Some(nav.rinex.filter(mask.clone().into()))
            } else {
                None
            };

            // perform analysis on these subsets
            analysis.push(QcAnalysis::new(&subset, &nav_subset, &opts));
        }
        analysis
    }
    /// Generates a Quality Check Report from provided Context and parametrization,
    /// in html format.
    pub fn html(context: QcContext, opts: QcOpts) -> String {
        let analysis = Self::build_analysis(&context, &opts);
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
                        title: context.primary.path.file_name().unwrap().to_str().unwrap_or("Unknown");
                    }
                    body {
                        div(id="version") {
                            h2(class="title") {
                                : "RINEX Quality Check summary"
                            }
                            table(class="table is-bordered") {
                                tbody {
                                    tr {
                                        th {
                                            : "Version"
                                        }
                                        td {
                                            : format!("rust-rnx: v{}", env!("CARGO_PKG_VERSION"))
                                        }
                                    }
                                }
                            }
                        }//div=header
                        div(id="context") {
                            h3(class="title") {
                                : "Context"
                            }
                            table(class="table is-bordered") {
                                tbody {
                                    : context.to_inline_html()
                                }
                            }
                        }//div=context
                        div(id="parameters") {
                            h3(class="title") {
                                : "Parameters"
                            }
                            table(class="table is-bordered") {
                                tbody {
                                    : opts.to_inline_html()
                                }
                            }
                        } //div=parameters
                        div(id="header") {
                            h3(class="title") {
                                : "File Header"
                            }
                            table(class="table is-bordered") {
                                tbody {
                                    : context.primary_data().header.to_inline_html()
                                }
                            }
                        }
                        /*
                         * Report all analysis that were performed
                         */
                        div(id="analysis") {
                            @ for analysis in &analysis {
                                table(class="table is-bordered") {
                                    tbody {
                                        : analysis.to_inline_html()
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
