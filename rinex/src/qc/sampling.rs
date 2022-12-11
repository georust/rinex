use crate::prelude::*;
use horrorshow::RenderBox;

/// Sampling QC report
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct QcReport {
    /// First epoch identified
    pub first_epoch: Epoch,
    /// Last epoch identified
    pub last_epoch: Epoch,
    /// Time line
    pub time_line: Duration,
    /// Dominant sample rate
    pub sample_rate: Duration,
    /// Unusual data gaps
    pub gaps: Vec<(Epoch, Duration)>,
}

impl QcReport {
    pub fn new(rnx: &Rinex) -> Self {
        let first_epoch = rnx
            .first_epoch()
            .expect("Sampling QC expects a RINEX indexed by epochs");
        let last_epoch = rnx
            .last_epoch()
            .expect("Sampling QC expects a RINEX indexed by epochs");
        Self {
            first_epoch,
            last_epoch,
            time_line: last_epoch - first_epoch,
            sample_rate: {
                let dominant = rnx
                    .epoch_intervals()
                    .into_iter()
                    .max_by(|(_, x_pop), (_, y_pop)| x_pop.cmp(y_pop))
                    .unwrap();
                dominant.0
            },
            gaps: rnx.data_gaps(),
        }
    }
    pub fn to_html(&self) -> Box<dyn RenderBox + '_> {
        box_html! {
            table(id="sampling") {
                tr {
                    td {
                        : "First Epoch:"
                    }
                    td {
                        : self.first_epoch.to_string()
                    }
                }
                tr {
                    td {
                        : "Last Epoch:"
                    }
                    td {
                        : self.last_epoch.to_string()
                    }
                }
                tr {
                    td {
                        : "Time line:"
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
                        : self.sample_rate.to_string()
                    }
                }
                tr {
                    td {
                        : "Gaps analysis"
                    }
                @ if self.gaps.len() > 0 {
                    table(id="sampling-gaps") {
                        tr {
                            th {
                                : "Epoch"
                            }
                            th {
                                : "Duration"
                            }
                        }
                        @ for (epoch, duration) in &self.gaps {
                            tr {
                                td {
                                    : epoch.to_string()
                                }
                                td {
                                    : duration.to_string()
                                }
                            }
                        }
                    }
                } else {
                    td {
                        : "NONE"
                    }
                }}
            }
        }
    }
}
