use super::prelude::*;
use horrorshow::{helper::doctype, RenderBox};
use strum_macros::EnumString;
use crate::types::Type;
use crate::Constellation;
use std::str::FromStr;
use crate::processing::{MaskFilter, Preprocessing};

mod opts;
pub use opts::QcOpts;

mod analysis;
use analysis::QcAnalysis;

/*
 * Array (CSV) pretty formatter
 */
pub (crate)fn pretty_array<A: std::fmt::Display>(list: &Vec<A>) -> String {
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
pub struct QcReport<'a> { 
	/// Configuration / options 
	opts: QcOpts,
	/// File name 
	filename: String,
	/// RINEX context
	rinex: &'a Rinex,
	/// Analysis that were performed
	analysis: Vec<QcAnalysis>,
}

#[derive(Clone, Debug, PartialEq, Eq, EnumString)]
#[cfg_attr(feature = "serde", derive(Deserialize))]
pub enum QcAnalysisStrategy {
	/// Analyze per Sv and per signal
	#[strum(serialize = "all")]
	All,
	/// Analyze per Sv: average signals together 
	#[strum(serialize = "per-sv")]
	PerSv,
	/// Analyze per Obserfable: average Sv together 
	#[strum(serialize = "per-observable")]
	PerObservable,
}

impl <'a> QcReport<'a> {
    /// Builds a new basic QC Report using default options
    pub fn basic(filename: &str, rnx: &'a Rinex) -> Self {
        Self::new(filename, rnx, QcOpts::default())
    }
    /// Builds a new QC Report
    pub fn new(filename: &str, rnx: &'a Rinex, opts: QcOpts) -> Self {
		/*
		 * Currently, we only sort analysis by GNSS system
		 */
		let mut classifiers = rnx.list_constellations();
		classifiers.sort();
		/*
		 * Build analysis
		 */
		let mut analysis: Vec<QcAnalysis> = Vec::with_capacity(classifiers.len());
		for classifier in classifiers {
			// create a retain mask
			let mask = MaskFilter::from_str(&format!("eq:{}", classifier))
				.expect("invalid analysis subset");
			// apply it 
			let subset = rnx.filter(mask.into());
			// and analyze this subset
			analysis.push(QcAnalysis::new(classifier, &subset));
		}
		
		Self {
			filename: filename.to_string(),
			opts,	
			rinex: rnx, 
			analysis,
        }
    }
}

impl <'a> HtmlReport for QcReport<'a> {
    fn to_html(&self) -> String {
        format!(
            "{}",
            html! {
                : doctype::HTML;
                html {
                    head {
                        meta(charset="utf-8");
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
				table(class="table is-bordered") {
					thead {
						: "File"
					}
					tbody {
						tr {
							th {
								: "Program"
							}
							th {
								: "File"
							}
							th {
								: "Type"
							}
						}
						tr {
							td {
								: format!("rust-rnx: v{}", env!("CARGO_PKG_VERSION"))
							}
							td {
								: self.filename.to_string()
							}
							@ if let Some(gnss) = self.rinex.header.constellation {
								td {
									: format!("{} {:?}", gnss, self.rinex.header.rinex_type)
								}
							} else {
								td {
									: format!("{:?} file", self.rinex.header.rinex_type)
								}
							}
						}
					}
				}
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
									: "Header Ground Position"
								}
							}
							tbody {
								@if let Some(ground_pos) = &self.rinex.header.ground_position {
									: ground_pos.to_inline_html()
								} else {
									: "Undefined"
								}
							}
						}
						table(class="table is-bordered") {
							thead {
								th {
									: "Manual Ground position"
								}
							}
							tbody {
								@ if let Some(ground_pos) = &self.opts.ground_position {
									td {
										: ground_pos.to_inline_html()
									}
								} else {
									td {
										: "Unknown"
									}
								}
							}
						}
						table(class="table is-bordered") {
							th {
								: "GNSS Constellations"
							}
							td {
								: pretty_array(&self.rinex.list_constellations())
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
						thead {
							th {
								: analysis.classifier.to_string()
							}
						}
						tbody {
							: analysis.to_inline_html()
						}
					}
				}
			}
        }
    }
}
