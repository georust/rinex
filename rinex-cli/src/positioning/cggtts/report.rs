use crate::cli::Context;
use itertools::Itertools;

use cggtts::prelude::{CommonViewClass, Duration, Epoch, Track, SV};
use rinex_qc::prelude::{html, Marker, MarkerSymbol, Markup, Mode, Plot, QcExtraPage, Render};

struct ReportTab {}

impl Render for ReportTab {
    fn render(&self) -> Markup {
        html! {}
    }
}

struct Summary {
    satellites: Vec<SV>,
    trk_duration: Duration,
    first_epoch: Epoch,
    last_epoch: Epoch,
    duration: Duration,
    cv_class: CommonViewClass,
}

impl Summary {
    fn new(ctx: &Context, solutions: &Vec<Track>) -> Self {
        let mut trk_duration = Duration::default();
        let mut cv_class = CommonViewClass::default();
        let (mut first_epoch, mut last_epoch) = (Epoch::default(), Epoch::default());
        let satellites = solutions.iter().map(|trk| trk.sv).collect::<Vec<_>>();
        for (trk_index, track) in solutions.iter().enumerate() {
            if trk_index == 0 {
                cv_class = track.class;
                first_epoch = track.epoch;
                trk_duration = track.duration;
            }
            last_epoch = track.epoch;
        }
        Self {
            satellites,
            trk_duration,
            cv_class,
            first_epoch,
            last_epoch,
            duration: last_epoch - first_epoch,
        }
    }
}

impl Render for Summary {
    fn render(&self) -> Markup {
        html! {
            div class="table-container" {
                div class="table is-bordered" {
                    tbody {
                        tr {
                            th class="is-info" {
                                "Satellites"
                            }
                            td {
                                (self.satellites.iter().join(", "))
                            }
                        }
                        tr {
                            th class="is-info" {
                                "Common View"
                            }
                            td {
                                (self.cv_class.to_string())
                            }
                        }
                        tr {
                        tr {
                            th class="is-info" {
                                "Track duration"
                            }
                            td {
                                (self.trk_duration.to_string())
                            }
                        }
                            th class="is-info" {
                                "First Epoch"
                            }
                            td {
                                (self.first_epoch.to_string())
                            }
                        }
                        tr {
                            th class="is-info" {
                                "Last Epoch"
                            }
                            td {
                                (self.last_epoch.to_string())
                            }
                        }
                        tr {
                            th class="is-info" {
                                "Duration"
                            }
                            td {
                                (self.duration.to_string())
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Solutions report
struct ReportContent {
    summary: Summary,
    sv_plot: Plot,
    elev_plot: Plot,
    sky_plot: Plot,
    ionod_plot: Plot,
    srsv_plot: Plot,
    refsys_plot: Plot,
    tropod_plot: Plot,
}

impl ReportContent {
    pub fn new(ctx: &Context, solutions: &Vec<Track>) -> Self {
        let epochs = solutions.iter().map(|trk| trk.epoch).collect::<Vec<_>>();
        Self {
            summary: Summary::new(ctx, solutions),
            sv_plot: {
                let mut plot = Plot::timedomain_plot("sv_plot", "SV Plot", "PRN #", true);
                plot
            },
            elev_plot: {
                let mut plot =
                    Plot::timedomain_plot("elev_plot", "Elevation", "Elevation [°]", true);
                plot
            },
            ionod_plot: {
                let mut plot =
                    Plot::timedomain_plot("ionod_plot", "Ionospheric Delay", "Error [m]", true);
                plot
            },
            tropod_plot: {
                let mut plot =
                    Plot::timedomain_plot("tropod_plot", "Tropospheric Delay", "Error [m]", true);
                plot
            },
            refsys_plot: {
                let mut plot = Plot::timedomain_plot("refsys_plot", "REFSYS", "REFSYS [s]", true);
                plot
            },
            srsv_plot: {
                let mut plot = Plot::timedomain_plot("srsv_plot", "SRSV", "SRSV [s]", true);
                plot
            },
            sky_plot: {
                let mut plot = Plot::sky_plot("sky_plot", "Sky Plot", true);
                plot
            },
        }
    }
}

impl Render for ReportContent {
    fn render(&self) -> Markup {
        html! {
            div class="table-container" {
                table class="is-bordered" {
                    tbody {
                        tr {
                            th class="is-info" {
                                "Summary"
                            }
                            td {
                                (self.summary.render())
                            }
                        }
                        tr {
                            th class="is-info" {
                                "SV Plot"
                            }
                            td {
                                (self.sv_plot.render())
                            }
                        }
                        tr {
                            th class="is-info" {
                                "Elevation"
                            }
                            td {
                                (self.elev_plot.render())
                            }
                        }
                        tr {
                            th class="is-info" {
                                "Sky Plot"
                            }
                            td {
                                (self.sky_plot.render())
                            }
                        }
                        tr {
                            th class="is-info" {
                                "REFSYS"
                            }
                            td {
                                (self.refsys_plot.render())
                            }
                        }
                        tr {
                            th class="is-info" {
                                "SRSV"
                            }
                            td {
                                (self.srsv_plot.render())
                            }
                        }
                        tr {
                            th class="is-info" {
                                "Ionosphere"
                            }
                            td {
                                (self.ionod_plot.render())
                            }
                        }
                        tr {
                            th class="is-info" {
                                "Troposphere"
                            }
                            td {
                                (self.tropod_plot.render())
                            }
                        }
                    }
                }
            }
        }
    }
}

pub struct Report {
    tab: ReportTab,
    content: ReportContent,
}

impl Report {
    pub fn formalize(self) -> QcExtraPage {
        QcExtraPage {
            tab: Box::new(self.tab),
            content: Box::new(self.content),
        }
    }
    pub fn new(ctx: &Context, solutions: &Vec<Track>) -> Self {
        Self {
            tab: ReportTab {},
            content: ReportContent::new(ctx, solutions),
        }
    }
}
