use crate::prelude::*;
use horrorshow::{helper::doctype, RenderBox};
use std::str::FromStr;
use strum_macros::EnumString;

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

#[derive(Debug, Clone)]
//TODO: improve this structure's definition:
//      we should only store all file header sections,
//      not file bodies. They were already used when Vec<QcAnalysis>
//      was built.
pub struct QcReport<'a> {
    /// Configuration / options
    opts: QcOpts,
    /// File name
    filename: String,
    /// RINEX context
    rinex: &'a Rinex,
    /// Navigation augmentation context
    nav_filenames: Vec<String>,
    /// Navigation augmentation context
    nav_rinex: Option<Rinex>,
    /// All analysis that were performed, sorted by
    /// opts.classification (if possible)
    analysis: Vec<QcAnalysis>,
}

impl<'a> QcReport<'a> {
    /// Builds a new basic QC Report using default options
    pub fn basic(filename: &str, rnx: &'a Rinex) -> Self {
        Self::new(filename, rnx, Vec::new(), None, QcOpts::default())
    }
    /// Builds a new QC Report, with possible complex RINEX context
    pub fn new(
        filename: &str,
        rnx: &'a Rinex,
        nav_filenames: Vec<String>, // possible augmentation fp
        nav_rinex: Option<Rinex>,   // possiblement augmentation context
        opts: QcOpts,
    ) -> Self {
        if cfg!(feature = "processing") {
            // classification of the QC report is feasible
            Self::make_sorted_report(filename, rnx, nav_filenames, nav_rinex, opts)
        } else {
            // classification is infeasible, create report as is
            Self {
                filename: filename.to_string(),
                rinex: rnx,
                nav_filenames,
                nav_rinex: nav_rinex.clone(),
                analysis: vec![QcAnalysis::new(rnx, &nav_rinex, &opts)],
                opts,
            }
        }
    }
    #[cfg(feature = "processing")]
    /*
     * When the "processing" feature is enabled
     * we have the capacity to let the user classify the generated report
     * per desired physics or other criteria
     */
    fn make_sorted_report(
        filename: &str,
        rnx: &'a Rinex,
        nav_filenames: Vec<String>, // possible augmentation fp
        nav_rinex: Option<Rinex>,   // possiblement augmentation context
        opts: QcOpts,
    ) -> Self {
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
                for gnss in rnx.constellations() {
                    filter_targets.push(TargetItem::from(gnss));
                }
            },
            QcClassification::Sv => {
                for sv in rnx.sv() {
                    filter_targets.push(TargetItem::from(sv));
                }
            },
            QcClassification::Physics => {
                let mut observables = rnx.observables();
                observables.sort(); // makes reportining nicer
                for obsv in observables {
                    if let Ok(obsv) = Observable::from_str(&obsv) {
                        filter_targets.push(TargetItem::from(obsv));
                    }
                }
            },
        }

        // apply all mask, and generate an analysis on such data set
        for target in filter_targets {
            let mask = MaskFilter {
                item: target,
                operand: MaskOperand::Equals,
            };

            // drop any other data set
            let subset = rnx.filter(mask.clone().into());
            // also apply it to other part of the RINEx context
            let nav_subset = if let Some(nav) = &nav_rinex {
                Some(nav.filter(mask.clone().into()))
            } else {
                None
            };

            // perform analysis on left overs
            analysis.push(QcAnalysis::new(&subset, &nav_subset, &opts));
        }

        Self {
            filename: filename.to_string(),
            opts,
            rinex: rnx,
            analysis,
            nav_rinex,
            nav_filenames,
        }
    }
}

impl<'a> HtmlReport for QcReport<'a> {
    fn to_html(&self) -> String {
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
                        title: format!("{}", self.filename);
                    }
                    body {
                        : self.to_inline_html()
                    }
                }
            }
        )
    }
    fn to_inline_html(&self) -> Box<dyn RenderBox + '_> {
        box_html! {
            div(id="general") {
                h3(class="title") {
                    : "RINEX Quality Check summary"
                }
                div(id="file") {
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
                            tr {
                                th {
                                    p {
                                        : "Name"
                                    }
                                }
                                td {
                                    p {
                                        : self.filename.to_string()
                                    }
                                    @ for fp in &self.nav_filenames {
                                        p {
                                            : fp.to_string()
                                        }
                                    }
                                }
                            }
                            tr {
                                th {
                                    : "Type"
                                }
                                td {
                                    @ if let Some(gnss) = self.rinex.header.constellation {
                                        p {
                                            : format!("{} {:?}", gnss, self.rinex.header.rinex_type)
                                        }
                                        @ if let Some(nav) = &self.nav_rinex {
                                            @ if let Some(gnss) = nav.header.constellation {
                                                p {
                                                    : format!("{} {:?}", gnss, nav.header.rinex_type)
                                                }
                                            } else {
                                                p {
                                                    : format!("{:?} file", nav.header.rinex_type)
                                                }
                                            }
                                        }
                                    } else {
                                        p {
                                            : format!("{:?} file", self.rinex.header.rinex_type)
                                        }
                                    }
                                }
                            }
                        }
                    }
                }//div="file"
                div(id="parameters") {
                    table(class="table is-bordered") {
                        thead {
                            th {
                                : "Parameters"
                            }
                        }
                        tbody {
                            : self.opts.to_inline_html()
                        }
                    }
                }//div="parameters"
            }
            div(id="header") {
                table(class="table is-bordered") {
                    thead {
                        th {
                            : "Header"
                        }
                    }//header/tablehead
                    tbody {
                        @if let Some(ant) = &self.rinex.header.rcvr_antenna {
                            tr {
                                th {
                                    : "Antenna model"
                                }
                                th {
                                    : "SN#"
                                }
                            }
                            tr {
                                td {
                                    : ant.model.clone()
                                }
                                td {
                                    : ant.model.clone()
                                }
                            }
                        } else {
                            tr {
                                th {
                                    : "Antenna"
                                }
                                td {
                                    : "Unknown"
                                }
                            }
                        }
                        @if let Some(rcvr) = &self.rinex.header.rcvr {
                            tr {
                                th {
                                    : "Receiver model"
                                }
                                th {
                                    : "SN#"
                                }
                                th {
                                    : "Firmware"
                                }
                            }
                            tr {
                                td {
                                    : rcvr.model.clone()
                                }
                                td {
                                    : rcvr.sn.clone()
                                }
                                td {
                                    : rcvr.firmware.clone()
                                }
                            }
                        }
                        table(class="table is-bordered") {
                            thead {
                                th {
                                    : "Antenna"
                                }
                            }
                            tbody {
                                tr {
                                    th {
                                        : "Header position"
                                    }
                                    @if let Some(ground_pos) = &self.rinex.header.ground_position {
                                        : ground_pos.to_inline_html()
                                    } else {
                                        td {
                                            : "Undefined"
                                        }
                                    }
                                }
                                tr {
                                    th {
                                        : "User defined position"
                                    }
                                    @ if let Some(ground_pos) = &self.opts.ground_position {
                                        : ground_pos.to_inline_html()
                                    } else {
                                        td {
                                            : "None"
                                        }
                                    }
                                }
                            }
                        }
                        table(class="table is-bordered") {
                            th {
                                : "GNSS Constellations"
                            }
                            td {
                                : pretty_array(&self.rinex.constellations())
                            }
                        }
                    }//header/tablebody
                }//table
            }//div=header
            /*
             * Report all analysis that were performed
             */
            div(id="analysis") {
                @ for analysis in &self.analysis {
                    table(class="table is-bordered") {
                        tbody {
                            : analysis.to_inline_html()
                        }
                    }
                }
            }
        }
    }
}
