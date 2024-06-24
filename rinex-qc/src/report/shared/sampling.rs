use hifitime::Unit;
use maud::{html, Markup, Render};
use rinex::prelude::{Duration, Epoch, Rinex, SV};
use std::collections::HashMap;

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
    /// longest gap detected
    pub longest_gap: Option<(Epoch, Duration)>,
    /// shortest gap detected
    pub shortest_gap: Option<(Epoch, Duration)>,
}

impl SamplingReport {
    pub fn from_rinex(rinex: &Rinex) -> Self {
        let gaps = rinex.data_gaps(None).collect::<Vec<_>>();
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
            shortest_gap: gaps
                .iter()
                .min_by(|(t_a, dur_a), (t_b, dur_b)| dur_a.partial_cmp(dur_b).unwrap())
                .copied(),
            longest_gap: gaps
                .iter()
                .max_by(|(t_a, dur_a), (t_b, dur_b)| dur_a.partial_cmp(dur_b).unwrap())
                .copied(),
            gaps,
        }
    }
    #[cfg(feature = "sp3")]
    pub fn from_sp3(sp3: &SP3) -> Self {
        let t_start = sp3.first_epoch().expect("badly formed sp3: empty?");
        let t_end = sp3.last_epoch().expect("badly formed sp3: empty?");
        Self {
            gaps: Vec::new(),   // TODO
            longest_gap: None,  //TODO
            shortest_gap: None, //TODO
            last_epoch: t_end,
            first_epoch: t_start,
            duration: t_end - t_start,
            sample_rate: Some(sp3.epoch_interval),
            dominant_sample_rate: Some(sp3.epoch_interval),
        }
    }
}

impl Render for SamplingReport {
    fn render(&self) -> Markup {
        html! {
            div class="table-container" {
                table class="table is-bordered" {
                    tbody {
                        tr {
                            th class="is-info" {
                                "Time Frame"
                            }
                        }
                        tr {
                            th {
                                "Start"
                            }
                            td {
                                (self.first_epoch.to_string())
                            }
                            th {
                                "End"
                            }
                            td {
                                (self.last_epoch.to_string())
                            }
                        }
                        tr {
                            th {
                                "Duration"
                            }
                            td {
                                (self.duration.to_string())
                            }
                        }
                        @if let Some(sample_rate) = self.sample_rate {
                            tr {
                                th class="is-info" {
                                    "Sampling"
                                }
                            }
                            tr {
                                th {
                                    "Rate"
                                }
                                td {
                                    (format!("{:.3} Hz", 1.0 / sample_rate.to_unit(Unit::Second)))
                                }
                                th {
                                    "Period"
                                }
                                td {
                                    (sample_rate.to_string())
                                }
                            }
                        }
                        @if let Some(sample_rate) = self.dominant_sample_rate {
                            tr {
                                th class="is-info" {
                                    "Dominant Sampling"
                                }
                            }
                            tr {
                                th {
                                    "Rate"
                                }
                                td {
                                    (format!("{:.3} Hz", 1.0 / sample_rate.to_unit(Unit::Second)))
                                }
                                th {
                                    "Period"
                                }
                                td {
                                    (sample_rate.to_string())
                                }
                            }
                        }
                        @if self.gaps.len() == 0 {
                            tr {
                                th class="is-primary" {
                                    "Data gaps"
                                }
                                td {
                                    "No gaps detected"
                                }
                            }
                        } @else {
                            tr {
                                th class="is-warning" {
                                    "Data gaps"
                                }
                                td {
                                    (format!("{} Data gaps", self.gaps.len()))
                                }
                            }
                            @if let Some((t_start, dur)) = self.shortest_gap {
                                tr {
                                    th {
                                        "Shortest"
                                    }
                                    td {
                                        (t_start.to_string())
                                    }
                                    td {
                                        (dur.to_string())
                                    }
                                }
                            }
                            @if let Some((t_start, dur)) = self.longest_gap {
                                tr {
                                    th {
                                        "Longest"
                                    }
                                    td {
                                        (t_start.to_string())
                                    }
                                    td {
                                        (dur.to_string())
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
