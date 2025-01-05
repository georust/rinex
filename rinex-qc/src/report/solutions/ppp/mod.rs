use crate::prelude::{Markup, html, Render};

mod summary;
use summary::QcNavPostPPPSummary;

struct QcNavPostPPPSolutions {
    summary: QcNavPostPPPSummary,
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

impl QcNavPostPPPSolutions {
    pub fn new(ctx: &Context, solutions: &Vec<Track>) -> Self {
        let summary = QcNavPostPPPSummary::new(ctx, solutions);

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
