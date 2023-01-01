use hifitime::Unit;
use crate::prelude::*;

#[derive(Debug, Clone)]
pub struct QcSamplingAnalysis {
	pub first_epoch: Epoch,
	pub last_epoch: Epoch,
	pub epoch_span: Duration,
    /// Dominant sample rate
    pub sample_interval: Duration,
	pub sample_rate_hz: f64,
	/// Epoch span
    pub time_line: Duration,
    /// Unusual data gaps
    pub gaps: Vec<(Epoch, Duration)>,
}

impl QcSamplingAnalysis {
    pub fn new(rnx: &Rinex) -> Self {
        let first_epoch = rnx
            .first_epoch()
            .expect("Sampling QC expects a RINEX indexed by epochs");
        let last_epoch = rnx
            .last_epoch()
            .expect("Sampling QC expects a RINEX indexed by epochs");
        let sample_interval = rnx
            .sampling_interval()
            .expect("failed to determine sample rate");
        Self {
            first_epoch,
            last_epoch,
			epoch_span: (last_epoch - first_epoch) + sample_interval,
            sample_interval,
			sample_rate_hz: 1.0 / sample_interval.to_unit(Unit::Second),
            time_line: last_epoch - first_epoch,
            gaps: rnx.data_gaps(),
        }
    }
}

use crate::qc::HtmlReport;
use horrorshow::{RenderBox};

impl HtmlReport for QcSamplingAnalysis {
	fn to_html(&self) -> String {
		todo!()
	}
    fn to_inline_html(&self) -> Box<dyn RenderBox + '_> {
        box_html! {
			tr {
				th { 
					: "Start"
				}
				th {
					: "End"
				}
				th {
					: "Span"
				}
			}
			tr {
				td {
					: self.first_epoch.to_string()
				}
				td {
					: self.last_epoch.to_string()
				}
				td {
					: self.epoch_span.to_string()
				}
			}
			tr {
				th {
					: "Sampling"
				}
				td {
					: format!("{} ({:.3} Hz)", self.sample_interval, self.sample_rate_hz)
				}
			}
			@ if self.gaps.len() == 0 {
				th {
					: "Gap analysis"
				}
				td {
					: "None"
				}
			} else {
				div(class="table-container") {
					table(class="table is-bordered") {
						thead {
							th {
								: "Gap analysis"
							}
						}
						tbody {
							@ for (epoch, dt) in &self.gaps {

							}
						}
					}
				}//gap analysis/table
			}
        }
    }
}
