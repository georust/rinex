use hifitime::Unit;
use rinex::prelude::{Duration, Epoch, Rinex};

#[cfg(feature = "sp3")]
use sp3::SP3;

/// [SamplingReport] applies to all time domain products
pub struct SamplingReport {
    /// First [`Epoch`] identified in time
    pub first_epoch: Epoch,
    /// Last [`Epoch`] identified in time
    pub last_epoch: Epoch,
    /// Time span of this RINEX context
    pub duration: Duration,
    /// File [`Header`] sample rate
    pub sample_rate: Option<Duration>,
    /// Dominant sample rate
    pub dominant_sample_rate: Option<Duration>,
    /// Unusual data gaps
    pub gaps: Vec<(Epoch, Duration)>,
}

impl SamplingReport {
    pub fn from_rinex(rinex: &Rinex) -> Self {
        Self {
            first_epoch: rinex
                .first_epoch()
                .expect("failed to determine first RINEX epoch, badly formed?"),
            last_epoch: rinex
                .last_epoch()
                .expect("failed to determine last RINEX epoch, badly formed?"),
            duration: rinex
                .duration()
                .expect("failed to determine RINEX time frame, badly formed?"),
            sample_rate: rinex.sample_rate(),
            dominant_sample_rate: rinex.dominant_sample_rate(),
            gaps: rinex.data_gaps(None).collect(),
        }
    }
    #[cfg(feature = "sp3")]
    pub fn from_sp3(sp3: &SP3) -> Self {
        let t_start = sp3.first_epoch().expect("badly formed sp3: empty?");
        let t_end = sp3.last_epoch().expect("badly formed sp3: empty?");
        Self {
            gaps: Vec::new(), // TODO
            last_epoch: t_end,
            first_epoch: t_start,
            duration: t_end - t_start,
            sample_rate: Some(sp3.epoch_interval),
            dominant_sample_rate: Some(sp3.epoch_interval),
        }
    }
}

use qc_traits::html::*;

impl RenderHtml for SamplingReport {
    fn to_inline_html(&self) -> Box<dyn RenderBox + '_> {
        box_html! {
            table {
                thead {
                    tr {
                        th {
                            : "Time frame"
                        }
                    }
                }
                tbody {
                    tr {
                        th {
                            : "Start"
                        }
                        td {
                            : self.first_epoch.to_string()
                        }
                    }
                    tr {
                        th {
                            : "End"
                        }
                        td {
                            : self.last_epoch.to_string()
                        }
                    }
                    tr {
                        th {
                            : "Duration"
                        }
                        td {
                            : self.duration.to_string()
                        }
                    }
                }
            }
            @ if let Some(sample_rate) = self.sample_rate {
                table {
                    thead {
                        tr {
                            th {
                                : "Sampling"
                            }
                        }
                    }
                    tbody {
                        tr {
                            th {
                                : "Rate"
                            }
                            td {
                                : format!("{:.3} Hz", 1.0 / sample_rate.to_unit(Unit::Second))
                            }
                        }
                        tr {
                            th {
                                : "Period"
                            }
                            td {
                                : format!("{}", sample_rate)
                            }
                        }
                    }
                }
            }
            @ if let Some(sample_rate) = self.dominant_sample_rate {
                table {
                    thead {
                        tr {
                            th {
                                : "Sampling"
                            }
                        }
                    }
                    tbody {
                        tr {
                            th {
                                : "Rate"
                            }
                            td {
                                : format!("{:.3} Hz", 1.0 / sample_rate.to_unit(Unit::Second))
                            }
                        }
                        tr {
                            th {
                                : "Period"
                            }
                            td {
                                : format!("{}", sample_rate)
                            }
                        }
                    }
                }
            }
            @ if self.gaps.len() > 0 {
                table {
                    thread {
                        tr {
                            th {
                                : "Gaps"
                            }
                        }
                    }
                    tbody {
                        @ for (start, duration) in self.gaps.iter() {
                            tr {
                                th {
                                    : start.to_string()
                                }
                                td {
                                    : duration.to_string()
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
