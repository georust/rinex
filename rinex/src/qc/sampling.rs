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
        let sample_rate = rnx
            .sampling_interval()
            .expect("failed to determine sample rate");
        Self {
            first_epoch,
            last_epoch,
            sample_rate,
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
                table(id="gap-analysis") {
                    : Self::gap_analysis(&self.gaps)
                }
            }
        }
    }
}
