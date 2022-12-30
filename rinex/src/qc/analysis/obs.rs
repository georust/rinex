use crate::prelude::*;
use crate::Carrier;
use crate::Observable;
use std::collections::HashMap;
use super::pretty_array;
use crate::carrier;

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

/*
 * Epoch Epoch completion,
 * defined as at least 1 Sv with PR + PH observed on both L1 and 
 * "rhs" signal, 
 * also SNR condition for both signals above current mask
 */
fn report_epoch_completion(total: usize, total_with_obs: usize, complete: &HashMap<Carrier, usize>) -> Box<dyn RenderBox + '_> {
	box_html! {
		table(class="table is-bordered") {
			thead {
				th {
					: "Total"
				}
				th {
					: "Epochs w/ observations"
				}
				@ for (signal, _) in complete {
					th {
						: format!("Complete (L1/{})", signal)
					}
				}
			}
			tbody {
				td {
					: total.to_string()
				}
				td {
					: format!("{} ({}%)", total_with_obs, total_with_obs * 100 / total)
				}
				@ for (_, count) in complete {
					td {
						: format!("{} ({}%)", count, count * 100 / total)
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
	/// Abnormal events, by chronological epochs
	anomalies: Vec<(Epoch, EpochFlag)>,
	/// Total epochs
	total_epochs: usize,
	/// Epochs with at least 1 observation
	total_with_obs: usize,
	/// Complete epochs, with respect to given signal
	complete_epochs: HashMap<Carrier, usize>,
}

impl QcObsAnalysis {
    pub fn new(rnx: &Rinex) -> Self {
		let sv = rnx
			.space_vehicules();
		let obs = rnx.header.obs.as_ref().unwrap();
		let mut observables = obs.codes.clone();
		let mut observables = observables
			.get_mut(&sv[0].constellation)
			.unwrap();
		let mut signals: Vec<Carrier> = Vec::new();
		let mut codes: Vec<String> = Vec::new();
		let mut anomalies: Vec<(Epoch, EpochFlag)> = Vec::new();
		let mut total_epochs: usize = 0;
		let mut epoch_with_obs: Vec<Epoch> = Vec::new();
		let mut complete_epochs: HashMap<Carrier, usize> = HashMap::new();

		if let Some(r) = rnx.record.as_obs() {
			total_epochs = r.len();
			for ((epoch, flag), (_, svs)) in r {
				if !flag.is_ok() {
					anomalies.push((*epoch, *flag));
				}
				for (sv, observables) in svs {
					if observables.len() > 0 && !epoch_with_obs.contains(&epoch) {
						epoch_with_obs.push(*epoch);
					}

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

			/*
			 * Now that signals have been determined,
			 * determine observation completion
			 */
			for (_, (_, svs)) in r {
				let mut complete: HashMap<Carrier, bool> = HashMap::new();
				for (sv, observables) in svs {
					for (observable, _) in observables {
						if !observable.is_phase_observable() {
							if !observable.is_pseudorange_observable() {
								continue ;
							}
						}
						
						let carrier_code = &observable.to_string()[1..2];
						if carrier_code == "1" { // we only search for other signals
							continue;
						}
						
						let code = observable.code()
							.unwrap();
						let carrier = Carrier::from_code(
							sv.constellation,
							&code)
							.unwrap();
						
						if let Some(complete) = complete.get_mut(&carrier) {
							if !*complete {
								for k_code in carrier::KNOWN_CODES.iter() {
									if !k_code.starts_with("1") {
										continue; // we're looking for a "1" reference
									}
									let to_find = match observable.is_phase_observable() {
										true => "C".to_owned() + k_code, // looking for PR
										false => "L".to_owned() + k_code, // looking for PH
									};
									for (observable, observation) in observables {
										if observable.to_string() == to_find {
											*complete = true;
										}
									}
									if *complete {
										break; 
									}
								}
							}
						} else {
							complete.insert(carrier, false);
						}
					}
				}
				for (carrier, completed) in complete {
					if completed {
						if let Some(count) = complete_epochs.get_mut(&carrier) {
							*count += 1;
						} else {
							complete_epochs.insert(carrier, 1);
						}
					}
				}
			}
		}
		
		codes.sort();
		signals.sort();
		observables.sort();
		//TODO
		//complete_epochs.sort_by_key();

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
			codes,
			signals,
			anomalies,
			total_epochs,
			total_with_obs: epoch_with_obs.len(),
			complete_epochs,
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
				@ if self.has_doppler {
					td {
						: "True"
					}
				} else {
					td {
						: "False"
					}
				}
			}
			div(class="table-container") {	
				: report_anomalies(&self.anomalies)
			}
			div(class="epoch-completion") {
				: report_epoch_completion(self.total_epochs, self.total_with_obs, &self.complete_epochs)
			}
        }
    }
}
