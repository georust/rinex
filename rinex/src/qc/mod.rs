use super::prelude::*;
use horrorshow::{helper::doctype, RenderBox};
use strum_macros::EnumString;
use crate::types::Type;
use crate::Constellation;

mod opts;
pub use opts::QcOpts;

mod sampling;
use sampling::QcSamplingAnalysis;

pub trait Report {
	/// Renders self to HTML
	fn to_html(&self) -> String;  
	/// Renders self to embedded HTML
	fn to_inline_html(&self) -> Box<dyn RenderBox + '_>;
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, EnumString)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
pub struct QcReport { 
	filename: String,
	rinex_type: Type,
	rinex_constellation: Option<Constellation>,
	constellations: Vec<Constellation>,
	opts: QcOpts,
    // stored header information for later use
    header: Header,
}

/// Types of analysis we can perform
#[derive(Clone, Debug, PartialEq, Eq, EnumString)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum QcAnalysisType {
	#[strum(serialize = "sampling")]
	Sampling,
	#[strum(serialize = "gaps")]
	DataGaps,
	#[strum(serialize = "ground-pos")]
	GroundPosition,
	#[strum(serialize = "obs-quality")]
	ObsQuality,
	#[strum(serialize = "nav-quality")]
	NavQuality,
	#[strum(serialize = "phase-slips")]
	PhaseSlipAnalysis,
	#[strum(serialize = "dcb")]
	DcbAnalysis,
	#[strum(serialize = "mp")]
	MpAnalysis,
	#[strum(serialize = "iono")]
	IonoAnalysis,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum QcAbstractAnalysis {
	Sampling(QcSamplingAnalysis),	
}

pub struct QcAnalysis {
	strategy: QcAnalysisStrategy,
	analysis: QcAbstractAnalysis
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

impl Default for ReportingStrategy {
	fn default() -> Self {
		Self::PerConstellation
	}
}

impl QcReport {
    /// Builds a new basic QC Report using default options
    pub fn basic(filename: &str, rnx: &Rinex) -> Self {
        Self::new(filename, rnx, QcOpts::default())
    }
    /// Builds a new QC Report
    pub fn new(filename: &str, rnx: &Rinex, opts: QcOpts) -> Self {
		let constellations = rnx.list_constellations();
        Self {
			filename: filename.to_string(),
			constellations: constellations.clone(),
			rinex_type: rnx.header.rinex_type,
			rinex_constellation: rnx.header.constellation.clone(),
			opts,	
            header: rnx.header.clone(),
			/*analysis: {
				let mut analysis: Vec<QcAnalysis> = Vec::with_capacity(constellations.len());
				/*for constellation in constellations {
					let mask = MaskFilter {
						operand: MaskOperand::Equals,
						item: TargetItem::ConstellationItem(vec![constellation]),
					};
					let rnx = rnx.apply(mask);
					let sampling = QcSamplingAnalsysis::new(&rnx);
				}*/
				analysis
			},*/
        }
    }
    /// Dumps self into (self sufficient) HTML
    pub fn to_html(&self) -> String {
        format!(
            "{}",
            html! {
                : doctype::HTML;
                html {
                    head {
                        meta(charset="utf-8");
						meta(name="viewport", content="width=device-width, initial-scale=1");
						link(rel="stylesheet", href="https:////cdn.jsdelivr.net/npm/bulma@0.9.4/css/bulma.min.css");
                        title: format!("{}", self.filename);
                    }
                    body {
                        : self.to_inline_html()
                    }
                }
            }
        )
    }
    /// Dumps self into HTML <div> section, named as suggested
    pub fn to_inline_html(&self) -> Box<dyn RenderBox + '_> {
        box_html! {
			div(id="general") {
				h3(class="title") {
					: "RINEX Quality Check summary"
				}
				table(class="table is-bordered") {
					thead {
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
					}
					tbody {
						tr {
							td {
								: format!("rust-rnx: v{}", env!("CARGO_PKG_VERSION"))
							}
							td {
								: self.filename.to_string()
							}
							@ if let Some(gnss) = self.rinex_constellation {
								td {
									: format!("{} {:?} file", gnss, self.rinex_type)
								}
							} else {
								td {
									: format!("{:?} file", self.rinex_type)
								}
							}
						}
					}
				}
            }
            div(id="header") {
				h4(class="title") {
					: "Header"
				}
				table(class="table is-bordered") {
					@if let Some(ant) = &self.header.rcvr_antenna {
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
					@if let Some(rcvr) = &self.header.rcvr {
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
					@if let Some((x, y, z)) = &self.header.coords {
						tr {
							th {
								: "Header position"
							}
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
							/*@let geo = map_3d::ecef2geodetic(pos.0, pos.1, pos.2, map_3d::Ellipsoid::WGS84) {
								(pos_x, pos_y, pos_z) => {
									td {
										: format!("lat: {} lon: {} alt: {}", pos_x, pos_y, pos_z) 
									},
								}
							}*/
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
					@ if let Some((x, y, z)) = self.opts.manual_pos_ecef {
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
								/*td {
									@ let geo = map_3d::ecef2geodetic(pos.0, pos.1, pos.2, map_3d::Ellipsoid::WGS84);
									: format!("lat: {} lon: {} alt: {}", geo.0, geo.1, geo.2)
								}*/
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
							: format!("{:?}", self.constellations)
						}
					}
                }//table
			}//div=header
			/*
			* Report all analysis performed
			*/
        }
    }
}
