use crate::cli::Context;
use itertools::Itertools;

use cggtts::prelude::{CommonViewClass, Duration, Epoch, Track, SV};
use gnss_qc::prelude::{html, MarkerSymbol, Markup, Mode, Plot, QcExtraPage, Render};

struct ReportTab {}

impl Render for ReportTab {
    fn render(&self) -> Markup {
        html! {
            a id="menu:cggtts" {
                span class="icon" {
                    i class="fa-solid fa-clock" {}
                }
                "CGGTTS Solutions"
            }
        }
    }
}

struct Summary {
    last_epoch: Epoch,
    first_epoch: Epoch,
    duration: Duration,
    satellites: Vec<SV>,
    trk_duration: Duration,
    cv_class: CommonViewClass,
    // TODO ground_pos: GroundPosition,
}

impl Summary {
    fn new(ctx: &Context, solutions: &Vec<Track>) -> Self {
        let mut trk_duration = Duration::default();
        let mut cv_class = CommonViewClass::default();
        let (mut first_epoch, mut last_epoch) = (Epoch::default(), Epoch::default());
        let satellites = solutions
            .iter()
            .map(|trk| trk.sv)
            .unique()
            .collect::<Vec<_>>();
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
            // TODO ground_pos: ctx.data.reference_position().unwrap(),
        }
    }
}

impl Render for Summary {
    fn render(&self) -> Markup {
        html! {
            div class="table-container" {
                table class="table is-bordered" {
                    tbody {
                        tr {
                            th class="is-info" {
                                "Common View"
                            }
                            td {
                                (self.cv_class.to_string())
                            }
                        }
                        tr {
                            th class="is-info" {
                                "Track duration"
                            }
                            td {
                                (self.trk_duration.to_string())
                            }
                        }
                        // tr {
                        //     th class="is-info" {
                        //         "Position"
                        //     }
                        //     td {
                        //         (self.ground_pos.render())
                        //     }
                        // }
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
    refsv_plot: Plot,
    srsv_plot: Plot,
    refsys_plot: Plot,
    srsys_plot: Plot,
    tropod_plot: Plot,
}

impl ReportContent {
    pub fn new(ctx: &Context, solutions: &Vec<Track>) -> Self {
        //let epochs = solutions.iter().map(|trk| trk.epoch).collect::<Vec<_>>();
        let summary = Summary::new(ctx, solutions);
        Self {
            sv_plot: {
                let mut plot = Plot::timedomain_plot("sv_plot", "SV Plot", "PRN #", true);
                for sv in summary.satellites.iter() {
                    let x = solutions
                        .iter()
                        .filter_map(|trk| if trk.sv == *sv { Some(trk.epoch) } else { None })
                        .collect::<Vec<_>>();
                    let y = solutions
                        .iter()
                        .filter_map(|trk| if trk.sv == *sv { Some(sv.prn) } else { None })
                        .collect::<Vec<_>>();
                    let trace = Plot::timedomain_chart(
                        &sv.to_string(),
                        Mode::Markers,
                        MarkerSymbol::Cross,
                        &x,
                        y,
                        true,
                    );
                    plot.add_trace(trace);
                }
                plot
            },
            elev_plot: {
                let mut plot =
                    Plot::timedomain_plot("elev_plot", "Elevation", "Elevation [°]", true);
                for sv in summary.satellites.iter() {
                    let x = solutions
                        .iter()
                        .filter_map(|trk| if trk.sv == *sv { Some(trk.epoch) } else { None })
                        .collect::<Vec<_>>();
                    let y = solutions
                        .iter()
                        .filter_map(|trk| {
                            if trk.sv == *sv {
                                Some(trk.elevation)
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>();
                    let trace = Plot::timedomain_chart(
                        &sv.to_string(),
                        Mode::Markers,
                        MarkerSymbol::Cross,
                        &x,
                        y,
                        true,
                    );
                    plot.add_trace(trace);
                }
                plot
            },
            ionod_plot: {
                let mut plot =
                    Plot::timedomain_plot("ionod_plot", "Ionospheric Delay", "Delay [s]", true);
                for sv in summary.satellites.iter() {
                    let x = solutions
                        .iter()
                        .filter_map(|trk| if trk.sv == *sv { Some(trk.epoch) } else { None })
                        .collect::<Vec<_>>();
                    let y = solutions
                        .iter()
                        .filter_map(|trk| {
                            if trk.sv == *sv {
                                Some(trk.data.mdio)
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>();
                    let trace = Plot::timedomain_chart(
                        &format!("{}(mdio)", sv),
                        Mode::Markers,
                        MarkerSymbol::Cross,
                        &x,
                        y,
                        true,
                    );
                    plot.add_trace(trace);

                    let x = solutions
                        .iter()
                        .filter_map(|trk| {
                            if trk.sv == *sv && trk.iono.is_some() {
                                Some(trk.epoch)
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>();
                    let y = solutions
                        .iter()
                        .filter_map(|trk| {
                            if trk.sv == *sv {
                                let iono = trk.iono?;
                                Some(iono.msio)
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>();
                    let trace = Plot::timedomain_chart(
                        &format!("{}(mdio)", sv),
                        Mode::Markers,
                        MarkerSymbol::Cross,
                        &x,
                        y,
                        true,
                    );
                    plot.add_trace(trace);
                }
                plot
            },
            tropod_plot: {
                let mut plot =
                    Plot::timedomain_plot("tropod_plot", "Tropospheric Delay", "Delay [s]", true);
                for sv in summary.satellites.iter() {
                    let x = solutions
                        .iter()
                        .filter_map(|trk| if trk.sv == *sv { Some(trk.epoch) } else { None })
                        .collect::<Vec<_>>();
                    let y = solutions
                        .iter()
                        .filter_map(|trk| {
                            if trk.sv == *sv {
                                Some(trk.data.mdtr)
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>();
                    let trace = Plot::timedomain_chart(
                        &sv.to_string(),
                        Mode::Markers,
                        MarkerSymbol::Cross,
                        &x,
                        y,
                        true,
                    );
                    plot.add_trace(trace);
                }
                plot
            },
            refsys_plot: {
                let mut plot = Plot::timedomain_plot("refsys_plot", "REFSYS", "REFSYS [s]", true);
                for sv in summary.satellites.iter() {
                    let x = solutions
                        .iter()
                        .filter_map(|trk| if trk.sv == *sv { Some(trk.epoch) } else { None })
                        .collect::<Vec<_>>();
                    let y = solutions
                        .iter()
                        .filter_map(|trk| {
                            if trk.sv == *sv {
                                Some(trk.data.refsys)
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>();
                    let trace = Plot::timedomain_chart(
                        &sv.to_string(),
                        Mode::Markers,
                        MarkerSymbol::Cross,
                        &x,
                        y,
                        true,
                    );
                    plot.add_trace(trace);
                }
                plot
            },
            srsys_plot: {
                let mut plot = Plot::timedomain_plot("srsys_plot", "SRSYS", "SRSYS [s/s]", true);
                for sv in summary.satellites.iter() {
                    let x = solutions
                        .iter()
                        .filter_map(|trk| if trk.sv == *sv { Some(trk.epoch) } else { None })
                        .collect::<Vec<_>>();
                    let y = solutions
                        .iter()
                        .filter_map(|trk| {
                            if trk.sv == *sv {
                                Some(trk.data.srsys)
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>();
                    let trace = Plot::timedomain_chart(
                        &sv.to_string(),
                        Mode::Markers,
                        MarkerSymbol::Cross,
                        &x,
                        y,
                        true,
                    );
                    plot.add_trace(trace);
                }
                plot
            },
            refsv_plot: {
                let mut plot = Plot::timedomain_plot("refsv_plot", "REFSV", "REFSV [s]", true);
                for sv in summary.satellites.iter() {
                    let x = solutions
                        .iter()
                        .filter_map(|trk| if trk.sv == *sv { Some(trk.epoch) } else { None })
                        .collect::<Vec<_>>();
                    let y = solutions
                        .iter()
                        .filter_map(|trk| {
                            if trk.sv == *sv {
                                Some(trk.data.refsv)
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>();
                    let trace = Plot::timedomain_chart(
                        &sv.to_string(),
                        Mode::Markers,
                        MarkerSymbol::Cross,
                        &x,
                        y,
                        true,
                    );
                    plot.add_trace(trace);
                }
                plot
            },
            srsv_plot: {
                let mut plot = Plot::timedomain_plot("srsv_plot", "SRSV", "SRSV [s/s]", true);
                for sv in summary.satellites.iter() {
                    let x = solutions
                        .iter()
                        .filter_map(|trk| if trk.sv == *sv { Some(trk.epoch) } else { None })
                        .collect::<Vec<_>>();
                    let y = solutions
                        .iter()
                        .filter_map(|trk| {
                            if trk.sv == *sv {
                                Some(trk.data.srsv)
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>();
                    let trace = Plot::timedomain_chart(
                        &sv.to_string(),
                        Mode::Markers,
                        MarkerSymbol::Cross,
                        &x,
                        y,
                        true,
                    );
                    plot.add_trace(trace);
                }
                plot
            },
            sky_plot: {
                let plot = Plot::sky_plot("sky_plot", "Sky Plot", true);
                plot
            },
            summary,
        }
    }
}

impl Render for ReportContent {
    fn render(&self) -> Markup {
        html! {
            div class="table-container" {
                table class="table is-bordered" {
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
                                "SRSYS"
                            }
                            td {
                                (self.srsys_plot.render())
                            }
                        }
                        tr {
                            th class="is-info" {
                                "REFSV"
                            }
                            td {
                                (self.refsv_plot.render())
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
            html_id: "cggtts".to_string(),
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
