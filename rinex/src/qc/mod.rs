use super::prelude::*;
use horrorshow::{helper::doctype, RenderBox};
use strum_macros::EnumString;
use crate::types::Type;
use crate::Constellation;

mod opts;
pub use opts::QcOpts;

mod analysis;
use analysis::QcAnalysis;

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
		let classifiers = rnx.list_constellations();
		let mut analysis: Vec<QcAnalysis> = Vec::with_capacity(classifiers.len());
		for classifier in classifiers {
			analysis.push(QcAnalysis::new(classifier, rnx));
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
								: "Name"
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
									: format!("{} {:?} file", gnss, self.rinex.header.rinex_type)
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
						@if let Some((x, y, z)) = &self.rinex.header.coords {
							tr {
								th {
									: "ECEF (WGS84)"
								}
								th {
									: "X"
								}
								th {
									: "Y"
								}
								th {
									: "Z"
								}
							}
							tr {
								td {
									: ""
								}
								td {
									: x.to_string()
								}
								td {
									: y.to_string()
								}
								td {
									: z.to_string()
								}
							}
							tr {
								th {
									: "GEO"
								}
								th {
									: "Latitude"
								}
								th {
									: "Longitude"
								}
								th {
									: "Altitude"
								}
							}
						} else {
							tr {
								th {
									: "Header position"
								}
								td {
									: "Unkonwn"
								}
							}
						}
						@ if let Some(pos) = &self.opts.ground_position {
							tr {
								th {
									: "Manual Ground position"
								}
								tr {
									th {
										: "ECEF (WGS84)"
									}
									th {
										: "X"
									}
									th {
										: "Y"
									}
									th {
										: "Z"
									}
								}
								tr {
									td {
										: ""
									}
									td {
										: pos.ecef.0.to_string()
									}
									td {
										: pos.ecef.1.to_string()
									}
									td {
										: pos.ecef.2.to_string()
									}
								}
								tr {
									th {
										: "GEO"
									}
									th {
										: "Latitude"
									}
									th {
										: "Longitude"
									}
									th {
										: "Altitude"
									}
									td {
										: pos.geo.0.to_string()
									}
									td {
										: pos.geo.1.to_string()
									}
									td {
										: pos.geo.2.to_string()
									}
								}
							}
						} else {
							tr {
								th {
									: "Manual Ground position"
								}
								td {
									: "Undefined"
								}
							}
						}
						tr {
							th {
								: "GNSS Constellations"
							}
							td {
								: format!("{:?}", self.rinex.list_constellations())
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
