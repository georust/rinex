use crate::prelude::*;
use crate::Carrier;
use crate::Observable;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct QcObsAnalysis {
	pub has_doppler: bool,
	pub observables: Vec<String>,
	pub signals: Vec<Carrier>,
	pub codes: Vec<String>,
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
		let mut signals: Vec<Carrier> = Vec::new();
		let mut codes: Vec<String> = Vec::new();
		//let mut data_missing: HashMap<Sv, HashMap<String, (u32,u32)>> = HashMap::new();

		if let Some(r) = rnx.record.as_obs() {
			for (_, (_, svs)) in r {
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
					: format!("{:?}", self.signals)
				}
			}
			tr {
				th {
					: "Codes"
				}
				td {
					: format!("{:?}", self.codes)
				}
			}
			tr {
				th {
					: "Observables"
				}
				td {
					: format!("{:?}", self.observables)
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
        }
    }
}
