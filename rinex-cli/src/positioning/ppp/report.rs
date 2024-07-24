use crate::cli::Context;
use std::collections::BTreeMap;

use rinex_qc::prelude::{html, Marker, MarkerSymbol, Markup, Mode, Plot, QcExtraPage, Render};
use rtk::prelude::{Epoch, PVTSolution, SV};

use itertools::Itertools;

struct ReportTab {}

impl Render for ReportTab {
    fn render(&self) -> Markup {
        html! {}
    }
}

struct ReportContent {
    /// satellites
    satellites: Vec<SV>,
    /// sv_plot
    sv_plot: Plot,
    /// clk_plot
    clk_plot: Plot,
    /// north_east
    north_east_plot: Plot,
    /// altitude
    altitude_plot: Plot,
    /// coords_err
    coords_err_plot: Plot,
    /// 3d_plot
    coords_err3d_plot: Plot,
    /// DOP
    dop_plot: Plot,
    /// TDOP
    tdop_plot: Plot,
    /// NAVI
    navi_plot: Plot,
    /// tropod
    tropod_plot: Plot,
    /// ionod
    ionod_plot: Plot,
}

impl ReportContent {
    pub fn new(ctx: &Context, solutions: &BTreeMap<Epoch, PVTSolution>) -> Self {
        let (rx_lat_rad, rx_long_rad) = (0.0_f64, 0.0_f64); // TODO
        let (rx_lat_ddeg, rx_long_ddeg) = (0.0_f64, 0.0_f64); // TODO
        let epochs = solutions.keys().cloned().collect::<Vec<_>>();

        let (x0_ecef, y0_ecef, z0_ecef) = ctx.rx_ecef.unwrap_or_default();

        let satellites = solutions
            .values()
            .map(|sol| sol.sv())
            .fold(vec![], |mut list, svnn| {
                for sv in svnn {
                    list.push(sv);
                }
                list
            })
            .into_iter()
            .unique()
            .collect::<Vec<_>>();

        Self {
            sv_plot: {
                let mut plot = Plot::timedomain_plot("sv_plot", "SV ID#", "PRN #", true);
                plot
            },
            north_east_plot: {
                let mut plot = Plot::timedomain_plot(
                    "north_east",
                    "North/East Coordinates",
                    "Coordinates [m]",
                    true,
                );
                plot
            },
            altitude_plot: {
                let mut plot = Plot::timedomain_plot("altitude", "Altitude", "Altitude [m]", true);
                plot
            },
            tropod_plot: {
                let mut plot =
                    Plot::timedomain_plot("tropo", "Troposphere Bias", "Error [m]", true);
                plot
            },
            ionod_plot: {
                let mut plot = Plot::timedomain_plot("iono", "Ionosphere Bias", "Error [m]", true);
                plot
            },
            tdop_plot: {
                let mut plot = Plot::timedomain_plot(
                    "tdop",
                    "Temporal dillution of precision",
                    "Error [m]",
                    true,
                );
                let tdop = solutions
                    .iter()
                    .map(|(_, sol)| sol.tdop)
                    .collect::<Vec<_>>();

                let trace = Plot::timedomain_chart(
                    "tdop",
                    Mode::LinesMarkers,
                    MarkerSymbol::Cross,
                    &epochs,
                    tdop,
                );
                plot.add_trace(trace);
                plot
            },
            dop_plot: {
                let mut plot =
                    Plot::timedomain_plot("dop", "Dillution of Precision", "Error [m]", true);

                let gdop = solutions
                    .iter()
                    .map(|(_, sol)| sol.gdop)
                    .collect::<Vec<_>>();

                let trace = Plot::timedomain_chart(
                    "gdop",
                    Mode::LinesMarkers,
                    MarkerSymbol::Cross,
                    &epochs,
                    gdop,
                );
                plot.add_trace(trace);

                let vdop = solutions
                    .iter()
                    .map(|(_, sol)| sol.vdop(rx_lat_rad, rx_long_rad))
                    .collect::<Vec<_>>();

                let trace = Plot::timedomain_chart(
                    "vdop",
                    Mode::LinesMarkers,
                    MarkerSymbol::Cross,
                    &epochs,
                    vdop,
                );
                plot.add_trace(trace);

                let hdop = solutions
                    .iter()
                    .map(|(_, sol)| sol.hdop(rx_lat_rad, rx_long_rad))
                    .collect::<Vec<_>>();

                let trace = Plot::timedomain_chart(
                    "hdop",
                    Mode::LinesMarkers,
                    MarkerSymbol::Cross,
                    &epochs,
                    hdop,
                );
                plot.add_trace(trace);
                plot
            },
            clk_plot: {
                let mut plot =
                    Plot::timedomain_plot("clk_offset", "Clock Offset", "Offset [s]", true);

                let dt = solutions
                    .iter()
                    .map(|(_, sol)| sol.dt.to_seconds())
                    .collect::<Vec<_>>();

                let mut trace = Plot::timedomain_chart(
                    "offset",
                    Mode::LinesMarkers,
                    MarkerSymbol::Cross,
                    &epochs,
                    dt,
                );
                plot.add_trace(trace);
                plot
            },
            coords_err_plot: {
                let mut plot = Plot::timedomain_plot("xy_plot", "X/Y Error", "Error [m]", true);
                let trace = Plot::timedomain_chart(
                    "x err",
                    Mode::LinesMarkers,
                    MarkerSymbol::Cross,
                    &epochs,
                    solutions
                        .values()
                        .map(|sol| sol.position.x - x0_ecef)
                        .collect(),
                );
                plot.add_trace(trace);
                let trace = Plot::timedomain_chart(
                    "y err",
                    Mode::LinesMarkers,
                    MarkerSymbol::Cross,
                    &epochs,
                    solutions
                        .values()
                        .map(|sol| sol.position.y - y0_ecef)
                        .collect(),
                );
                plot.add_trace(trace);
                let trace = Plot::timedomain_chart(
                    "z err",
                    Mode::LinesMarkers,
                    MarkerSymbol::Cross,
                    &epochs,
                    solutions
                        .values()
                        .map(|sol| sol.position.z - z0_ecef)
                        .collect(),
                );
                plot.add_trace(trace);
                plot
            },
            coords_err3d_plot: {
                let mut plot = Plot::plot_3d(
                    "3d_sphere",
                    "3D errors",
                    "X error [m]",
                    "Y Error [m]",
                    "Z Error [m]",
                    true,
                );
                plot
            },
            navi_plot: {
                let mut plot = Plot::timedomain_plot("navi_plot", "NAVI Plot", "Error [m]", true);
                plot
            },
            satellites,
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
                                "Satellites"
                            }
                            td {
                                (self.satellites.iter().join(" ,"))
                            }
                        }
                        tr {
                            th class="is-info" {
                                "NAVI"
                            }
                            td {
                                (self.navi_plot.render())
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
                                "RX Clock offset"
                            }
                            td {
                                (self.clk_plot.render())
                            }
                        }
                        tr {
                            th class="is-info" {
                                "N/E/U coordinates"
                            }
                            td {
                                table class="table is-bordered" {
                                    tr {
                                        th class="is-info" {
                                            "North/East"
                                        }
                                        td {
                                            (self.north_east_plot.render())
                                        }
                                    }
                                    tr {
                                        th class="is-info" {
                                            "Altitude"
                                        }
                                        td {
                                            (self.altitude_plot.render())
                                        }
                                    }
                                }
                            }
                        }
                        tr {
                            th class="is-info" {
                                "Errors"
                            }
                            td {
                                table class="table is-bordered" {
                                    tr {
                                        th class="is-info" {
                                            "Coordinates"
                                        }
                                        td {
                                            (self.coords_err_plot.render())
                                        }
                                    }
                                    tr {
                                        th class="is-info" {
                                            "3D"
                                        }
                                        td {
                                            (self.coords_err3d_plot.render())
                                        }
                                    }
                                }
                            }
                        }
                        tr {
                            th class="is-info" {
                                "DOP"
                            }
                            td {
                                (self.dop_plot.render())
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

/// Solutions report
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
    pub fn new(ctx: &Context, solutions: &BTreeMap<Epoch, PVTSolution>) -> Self {
        Self {
            tab: ReportTab {},
            content: ReportContent::new(ctx, solutions),
        }
    }
}
