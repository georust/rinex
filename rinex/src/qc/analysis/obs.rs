use crate::prelude::*;
use crate::Carrier;
use crate::Observable;
use std::collections::HashMap;

/*
 * Array (CSV) pretty formatter
 */
fn pretty_array<A: std::fmt::Display>(list: &Vec<A>) -> String {
    let mut s = String::with_capacity(8 * list.len());
    for index in 0..list.len() - 1 {
        s.push_str(&format!("{}, ", list[index]));
    }
    s.push_str(&list[list.len() - 1].to_string());
    s
}

/*
 * Lx signals special formatting
 */
fn report_signals(list: &Vec<Carrier>) -> String {
	let mut s = String::with_capacity(3*list.len());
	for index in 0..list.len() -1 {
		s.push_str(&format!("{} ({:.3} MHz), ", list[index],
			list[index].carrier_frequency_mhz()));
	}
	s.push_str(&format!("{} ({:.3} MHz)", list[list.len()-1],
		list[list.len()-1].carrier_frequency_mhz()));
	s
}

/*
 * Epoch anomalies formatter
 */
fn report_anomalies(anomalies: &Vec<(Epoch, EpochFlag)>) -> Box<dyn RenderBox + '_> {
	if anomalies.len() == 0 {
		box_html! {
			table(class="table is-bordered") {
				th {
					: "Anomalies"
				}
				td {
					: "None"
				}
			}
		}
	} else {
		box_html! {
			table(class="table is-bordered") {
				thead {
					th {
						: "Anomalies"
					}
					th {
						: "Power failure"
					}
					th {
						: "Antenna movement detected"
					}
					th {
						: "Kinematic"
					}
					th {
						: "External event"
					}
					th {
						: "Cycle Slips"
					}
				}
				tbody {
					@ for (epoch, flag) in anomalies {
						tr { 
							td {
								: epoch.to_string()
							}
							/*@match flag {
								EpochFlag::PowerFailure => {
									td {
										: "x"
									}
								},
								EpochFlag::AntennaBeingMoved => {
									td {
										: ""
									}
									td {
										: "x"
									}
								},
								EpochFlag::NewSiteOccupation => {
									td {
										: ""
									}
									td {
										: ""
									}
									td {
										: "x"
									}
								},
								EpochFlag::ExternalEvent => {
									td {
										: ""
									}
									td {
										: ""
									}
									td {
										: ""
									}
									td {
										: "x"
									}
								},
								EpochFlag::CycleSlip => {
									td {
										: ""
									}
									td {
										: ""
									}
									td {
										: ""
									}
									td {
										: ""
									}
									td {
										: "x"
									}
								},
							}*/
						}
					}
				}
			}
		}
	}
}

#[derive(Debug, Clone)]
pub struct QcObsAnalysis {
	/// list of observables identified
	observables: Vec<String>,
	/// list of signals identified
	signals: Vec<Carrier>,
	/// list of codes encountered
	codes: Vec<String>,
	/// true if doppler observation is present
	has_doppler: bool,
	/// Abornmal events, by chronological epochs
	anomalies: Vec<(Epoch, EpochFlag)>,
	//pub data_missing: HashMap<Sv, HashMap<String, (u32, u32)>>,
}

impl QcObsAnalysis {
    pub fn new(rnx: &Rinex) -> Self {
		let sv = rnx
			.space_vehicules();
		let observables = &rnx
			.header
			.obs
			.as_ref()
			.unwrap()
			.codes
			.get(&sv[0].constellation)
			.unwrap();
		let mut nb_epochs: usize = 0;;
		let mut signals: Vec<Carrier> = Vec::new();
		let mut codes: Vec<String> = Vec::new();
		let mut anomalies: Vec<(Epoch, EpochFlag)> = Vec::new();
		//let mut data_missing: HashMap<Sv, HashMap<String, (u32,u32)>> = HashMap::new();

		if let Some(r) = rnx.record.as_obs() {
			nb_epochs = r.len();

			for ((epoch, flag), (_, svs)) in r {
				if !flag.is_ok() {
					anomalies.push((*epoch, *flag));
				}
				for (sv, observables) in svs {
					for (observable, observation) in observables {
						let code = observable.code()
							.unwrap();
						let carrier = Carrier::from_code(
							sv.constellation,
							&code)
								.unwrap();
						if !signals.contains(&carrier) {
							signals.push(carrier);
						}
						if !codes.contains(&code) {
							codes.push(code);
						}
					}
				}
			}
		}
        Self {
			observables: {
				observables.iter()
					.map(|v| v.to_string())
					.collect()
			},
			has_doppler: {
				let mut ret = false;
				for obs in observables.iter() {
					if obs.is_doppler_observable() {
						ret = true;
						break;
					}
				}
				ret
			},
			//data_missing,
			codes,
			signals,
			anomalies,
        }
    }
}

use crate::qc::HtmlReport;
use horrorshow::{helper::doctype, RenderBox};

impl HtmlReport for QcObsAnalysis {
	fn to_html(&self) -> String {
		todo!()
	}
    fn to_inline_html(&self) -> Box<dyn RenderBox + '_> {
        box_html! {
			tr {
				th {
					: "Signals"
				}
				td {
					: report_signals(&self.signals)
				}
			}
			tr {
				th {
					: "Codes"
				}
				td {
					: pretty_array(&self.codes)
				}
			}
			tr {
				th {
					: "Observables"
				}
				td {
					: pretty_array(&self.observables)
				}
			}
			tr {
				th {
					: "Has Doppler"
				}
				td {
					: self.has_doppler.to_string()
				}
			}
			div(class="table-container") {	
				: report_anomalies(&self.anomalies)
			}
        }
    }
}
