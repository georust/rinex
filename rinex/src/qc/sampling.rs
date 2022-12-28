use crate::prelude::*;
use horrorshow::RenderBox;
use hifitime::Unit;

/// Sampling QC report
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct QcSamplingAnalysis {
    /// First epoch identified
    pub first_epoch: Epoch,
    /// Last epoch identified
    pub last_epoch: Epoch,
    /// Time line
    pub time_line: Duration,
    /// Dominant sample rate
    pub sample_interval: Duration,
	pub sample_rate_hz: f64,
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
            sample_interval,
			sample_rate_hz: 1.0 / sample_interval.to_unit(Unit::Second),
            time_line: last_epoch - first_epoch,
            gaps: rnx.data_gaps(),
        }
    }
    fn gap_analysis(gaps: &Vec<(Epoch, Duration)>) -> Box<dyn RenderBox + '_> {
        box_html! {
            @ if gaps.len() == 0 {
                tr {
                    th {
                        b {
                            : "Gap analysis: "
                        }
                    }
                    td {
                        : "Data missing"
                    }
                }
            } else {
                tr {
                    td {
                        b {
                            : "Gap Aanalysis:"
                        }
                    }
                }
                tr {
                    td {
                        : ""
                    }
                    td {
                        : "Epoch (start)"
                    }
                    td {
                        : "Duration"
                    }
                }
                @ for (epoch, duration) in gaps {
                    tr {
                        td {
                            : ""
                        }
                        td {
                            : epoch.to_string()
                        }
                        td {
                            : duration.to_string()
                        }
                    }
                }
            }
        }
    }
    pub fn to_inline_html(&self) -> Box<dyn RenderBox + '_> {
        box_html! {
			div(id="sampling") {
				h4(class="title") {
					: "Sampling analysis"
				}
				table(class="table") {
					tr {
						th {
							: "Epochs"
						}
						td {
							: "First"
						}
						td {
							: "Last"
						}
						td {
							: "Time line"
						}
					}
					tr {
						td {
							: ""
						}
						td {
							: self.first_epoch.to_string()
						}
						td {
							: self.last_epoch.to_string()
						}
						td {
							: self.time_line.to_string()
						}
					}
					tr {
						td {
							: "Sample rate:"
						}
						td {
							: format!("{:.1} Hz", self.sample_rate_hz)
						}
					}
					/*table(id="gap-analysis") {
						: Self::gap_analysis(&self.gaps)
					}*/
				}//table
			}//div=sampling
		}
	}
}
