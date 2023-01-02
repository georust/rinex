use std::str::FromStr;
use crate::{
	prelude::*,
	processing::{
		MaskFilter, 
		TargetItem, 
		Preprocessing,
	},
};
use strum_macros::EnumString;
use horrorshow::{helper::doctype, RenderBox};

mod opts;
pub use opts::{
	QcOpts,
	QcClassificationMethod,
};

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
	/// Navigation augmentation context
    nav_filenames: Vec<String>,
	/// Navigation augmentation context
	nav_rinex: Option<Rinex>,
	/// Analysis that were performed per classification method
	analysis: Vec<QcAnalysis>,
}

impl <'a> QcReport<'a> {
    /// Builds a new basic QC Report using default options
    pub fn basic(filename: &str, rnx: &'a Rinex) -> Self {
        Self::new(filename, rnx, Vec::new(), None, QcOpts::default())
    }
    /// Builds a new QC Report
    pub fn new(filename: &str, 
        rnx: &'a Rinex, 
        nav_filenames: Vec<String>, // possible augmentation fp
        nav_rinex: Option<Rinex>, // possiblement augmentation context
        opts: QcOpts,
    ) -> Self {
        /*
         * Classification Method
         */
		let mut classifier: TargetItem = match opts.classification {
			QcClassificationMethod::GNSS => {
				let mut gnss = rnx.list_constellations();
				gnss.sort();
				TargetItem::from(gnss)
			},
			QcClassificationMethod::Sv => {
				let mut sv = rnx.space_vehicules();
				sv.sort();
				TargetItem::from(sv)
			},
			QcClassificationMethod::Physics => {
				let mut observables: Vec<Observable> = rnx.observables()
					.iter()
					.map(|k| Observable::from_str(k).unwrap())
					.collect();
				observables.sort();
				TargetItem::from(observables)
			},
		};
		/*
		 * Build (record) analysis
		 */
		let mut analysis: Vec<QcAnalysis> = Vec::new();
		match classifier {
			TargetItem::ConstellationItem(cs) => {
				for c in cs {
					// create the classification mask
					let mask = MaskFilter::from_str(&format!("eq:{}", c))
						.expect("invalid classification mask");
					// apply it 
					let subset = rnx.filter(mask.clone().into());
					/*
					 * possible NAV subset:
					 *  + apply same GNSS filter
					 */
					let nav_subset = match nav_rinex {
						Some(ref rnx) => Some(rnx.filter(mask.clone().into())),
						_ => None,
					};
					// and analyze this subset
					analysis.push(QcAnalysis::new(TargetItem::from(c), &subset, &nav_subset, &opts));
				}
			},
			/* >>>>TODO<<<<<
			TargetItem::SvItem(svs) => {
				for sv in svs {
					// create the classification mask
					let mask = MaskFilter::from_str(&format!("eq:{}", sv))
						.expect("invalid classification mask");
					// apply it 
					let subset = rnx.filter(mask.into());
					// and analyze this subset
					analysis.push(QcAnalysis::new(
						TargetItem::from(sv), &subset, &opts));
				}
			},
			TargetItem::ObservableItem(obs) => {
				for ob in obs {
					// create the classification mask
					let mask = MaskFilter::from_str(&format!("eq:{}", ob))
						.expect("invalid classification mask");
					// apply it 
					let subset = rnx.filter(mask.into());
					// and analyze this subset
					analysis.push(QcAnalysis::new(TargetItem::from(ob), &subset, &opts));
				}
			},*/
			_ => unreachable!(),
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

impl <'a> HtmlReport for QcReport<'a> {
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
