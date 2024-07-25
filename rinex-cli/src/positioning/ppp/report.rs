use crate::cli::Context;
use std::collections::BTreeMap;

use rtk::prelude::{
    Config as NaviConfig, Duration, Epoch, Filter as NaviFilter, Method as NaviMethod, PVTSolution,
    TimeScale, SV,
};

use rinex_qc::{
    plot::MapboxStyle,
    prelude::{html, Marker, MarkerSymbol, Markup, Mode, Plot, QcExtraPage, Render},
};

use itertools::Itertools;

use map_3d::{ecef2geodetic, Ellipsoid};

struct ReportTab {}

impl Render for ReportTab {
    fn render(&self) -> Markup {
        html! {}
    }
}

enum Technique {
    GeodeticSurvey,
}

impl std::fmt::Display for Technique {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::GeodeticSurvey => write!(f, "Geodetic Survey"),
        }
    }
}

impl Technique {
    fn tooltip(&self) -> String {
        match self {
            Self::GeodeticSurvey => {
                "Static Geodetic survey (fixed point coordinates evaluation)".to_string()
            },
        }
    }
}

struct Summary {
    technique: Technique,
    method: NaviMethod,
    filter: NaviFilter,
    first_epoch: Epoch,
    last_epoch: Epoch,
    duration: Duration,
    timescale: TimeScale,
    final_neu: (f64, f64, f64),
    final_err_m: (f64, f64, f64),
    worst_err_m: (f64, f64, f64),
}

impl Render for Summary {
    fn render(&self) -> Markup {
        html! {
            div class="table-container" {
                table class="table is-bordered" {
                    tbody {
                        tr {
                            th class="is-info" {
                                (self.technique.to_string())
                            }
                        }
                        tr {
                            th class="is-info" {
                                "Navigation method"
                            }
                            td {
                                (self.method.to_string())
                            }
                        }
                        tr {
                            th class="is-info" {
                                "First solution"
                            }
                            td {
                                (self.first_epoch.to_string())
                            }
                        }
                        tr {
                            th class="is-info" {
                                "Last solution"
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
                        tr {
                            th class="is-info" {
                                "Timescale"
                            }
                            td {
                                (self.timescale.to_string())
                            }
                        }
                        tr {
                            th class="is-info" {
                                "Navigation Filter"
                            }
                            td {
                                (self.filter.to_string())
                            }
                        }
                        tr {
                            th class="is-info" {
                                "Final"
                            }
                        }
                    }
                }
            }
        }
    }
}

impl Summary {
    fn new(
        cfg: &NaviConfig,
        ctx: &Context,
        solutions: &BTreeMap<Epoch, PVTSolution>,
        err_ecef: (f64, f64, f64),
    ) -> Self {
        let (x0, y0, z0) = err_ecef;
        let mut timescale = TimeScale::default();
        let (mut first_epoch, mut last_epoch) = (Epoch::default(), Epoch::default());
        let (mut final_err_x, mut final_err_y, mut final_err_z) = (0.0_f64, 0.0_f64, 0.0_f64);
        let (mut final_neu_x, mut final_neu_y, mut final_neu_z) = (0.0_f64, 0.0_f64, 0.0_f64);
        let (mut worst_err_x, mut worst_err_y, mut worst_err_z) = (0.0_f64, 0.0_f64, 0.0_f64);
        for (index, (t, sol)) in solutions.iter().enumerate() {
            if index == 0 {
                first_epoch = *t;
            }
            let (err_x, err_y, err_z) = (
                sol.position.x - x0,
                sol.position.y - y0,
                sol.position.z - z0,
            );
            final_err_x = err_x;
            final_err_y = err_y;
            final_err_z = err_z;

            if err_x > worst_err_x {
                worst_err_x = err_x;
            }
            if err_y > worst_err_y {
                worst_err_y = err_y;
            }
            if err_z > worst_err_z {
                worst_err_z = err_z;
            }

            last_epoch = *t;
            timescale = sol.timescale;
        }
        Self {
            first_epoch,
            last_epoch,
            timescale,
            method: cfg.method,
            filter: cfg.solver.filter,
            duration: last_epoch - first_epoch,
            technique: Technique::GeodeticSurvey,
            final_err_m: (final_err_x, final_err_y, final_err_z),
            worst_err_m: (worst_err_x, worst_err_y, worst_err_z),
            final_neu: (final_neu_x, final_neu_y, final_neu_z),
        }
    }
}

struct ReportContent {
    /// satellites
    satellites: Vec<SV>,
    /// summary
    summary: Summary,
    /// sv_plot
    sv_plot: Plot,
    /// map_proj
    map_proj: Plot,
    /// clk_plot
    clk_plot: Plot,
    /// neu_plot
    neu_plot: Plot,
    /// coords_err
    coords_err_plot: Plot,
    /// 3d_plot
    coords_err3d_plot: Plot,
    /// velocity_plot
    vel_plot: Plot,
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
    pub fn new(cfg: &NaviConfig, ctx: &Context, solutions: &BTreeMap<Epoch, PVTSolution>) -> Self {
        let epochs = solutions.keys().cloned().collect::<Vec<_>>();

        let (x0_ecef, y0_ecef, z0_ecef) = ctx.rx_ecef.unwrap_or_default();
        let (lat0_ddeg, lon0_ddeg, _) = ecef2geodetic(x0_ecef, y0_ecef, z0_ecef, Ellipsoid::WGS84);
        let (lat0_rad, lon0_rad) = (lat0_ddeg.to_radians(), lon0_ddeg.to_radians());

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
            .sorted()
            .collect::<Vec<_>>();

        Self {
            summary: Summary::new(cfg, ctx, solutions, (x0_ecef, y0_ecef, z0_ecef)),
            map_proj: {
                let mut map_proj = Plot::world_map(
                    "map_proj",
                    "Map Projection",
                    MapboxStyle::StamenTerrain,
                    (lat0_ddeg, lon0_ddeg),
                    18,
                    true,
                );
                map_proj
            },
            sv_plot: {
                let mut plot = Plot::timedomain_plot("sv_plot", "SV ID#", "PRN #", true);
                for sv in satellites.iter() {
                    let epochs = solutions
                        .iter()
                        .filter_map(|(t, sol)| {
                            if sol.sv.keys().contains(sv) {
                                Some(*t)
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>();
                    let prn = epochs.iter().map(|_| sv.prn).collect::<Vec<_>>();
                    let trace = Plot::timedomain_chart(
                        &sv.to_string(),
                        Mode::Markers,
                        MarkerSymbol::Cross,
                        &epochs,
                        prn,
                    );
                    plot.add_trace(trace);
                }
                plot
            },
            neu_plot: {
                let mut plot = Plot::timedomain_plot(
                    "neu_plot",
                    "North/East/Up Coordinates",
                    "Coordinates [m]",
                    true,
                );
                plot
            },
            vel_plot: {
                let mut plot =
                    Plot::timedomain_plot("vel_plot", "Velocity", "Velocity [m/s]", true);
                let x = solutions
                    .iter()
                    .map(|(_, sol)| sol.velocity.x)
                    .collect::<Vec<_>>();
                let y = solutions
                    .iter()
                    .map(|(_, sol)| sol.velocity.y)
                    .collect::<Vec<_>>();
                let z = solutions
                    .iter()
                    .map(|(_, sol)| sol.velocity.z)
                    .collect::<Vec<_>>();
                let trace = Plot::timedomain_chart(
                    "vel_x",
                    Mode::LinesMarkers,
                    MarkerSymbol::Cross,
                    &epochs,
                    x,
                );
                let trace = Plot::timedomain_chart(
                    "vel_y",
                    Mode::LinesMarkers,
                    MarkerSymbol::Cross,
                    &epochs,
                    y,
                );
                let trace = Plot::timedomain_chart(
                    "vel_z",
                    Mode::LinesMarkers,
                    MarkerSymbol::Cross,
                    &epochs,
                    z,
                );
                plot
            },
            tropod_plot: {
                let mut plot =
                    Plot::timedomain_plot("tropo", "Troposphere Bias", "Error [m]", true);
                for sv in satellites.iter() {
                    let x = solutions
                        .iter()
                        .filter_map(|(t, sol)| {
                            if sol.sv.keys().contains(sv) {
                                Some(*t)
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>();
                    let y = solutions
                        .iter()
                        .filter_map(|(_, sol)| {
                            if let Some(value) =
                                sol.sv.iter().filter(|(s, _)| *s == sv).reduce(|k, _| k)
                            {
                                value.1.tropo_bias.value()
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
                    );
                    plot.add_trace(trace);
                }
                plot
            },
            ionod_plot: {
                let mut plot = Plot::timedomain_plot("iono", "Ionosphere Bias", "Error [m]", true);
                for sv in satellites.iter() {
                    let x = solutions
                        .iter()
                        .filter_map(|(t, sol)| {
                            if sol.sv.keys().contains(sv) {
                                Some(*t)
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>();
                    let y = solutions
                        .iter()
                        .filter_map(|(_, sol)| {
                            if let Some(value) =
                                sol.sv.iter().filter(|(s, _)| *s == sv).reduce(|k, _| k)
                            {
                                value.1.iono_bias.value()
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
                    );
                    plot.add_trace(trace);
                }
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
                    .map(|(_, sol)| sol.vdop(lat0_rad, lon0_rad))
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
                    .map(|(_, sol)| sol.hdop(lat0_rad, lon0_rad))
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
                                button aria-label="Satellites that contributed to the solutions" data-balloon-pos="right" {
                                    "Satellites"
                                }
                            }
                            td {
                                (self.satellites.iter().join(" ,"))
                            }
                        }
                        tr {
                            th class="is-info" {
                                button aria-label="NAVI Plot" data-balloon-pos="right" {
                                    "NAVI"
                                }
                            }
                            td {
                                (self.navi_plot.render())
                            }
                        }
                        tr {
                            th class="is-info" {
                                button aria-label="SV Contribution over time" data-balloon-pos="right" {
                                    "SV Plot"
                                }
                            }
                            td {
                                (self.sv_plot.render())
                            }
                        }
                        tr {
                            th class="is-info" {
                                button aria-label="Receiver Clock Offset with respected to Timescale" data-balloon-pos="right" {
                                    "Clock offset"
                                }
                            }
                            td {
                                (self.clk_plot.render())
                            }
                        }
                        tr {
                            th class="is-info" {
                                button aria-label="Absolute North / East and Altitude coordinates" data-balloon-pos="right" {
                                    "N/E/U coordinates"
                                }
                            }
                            td {
                                (self.neu_plot.render())
                            }
                        }
                        tr {
                            th class="is-info" {
                                button aria-label="3D errors (surveying applications only)" data-balloon-pos="right" {
                                    "Errors"
                                }
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
                                "Velocity"
                            }
                            td {
                                (self.vel_plot.render())
                            }
                        }
                        tr {
                            th class="is-info" {
                                button aria-label="Geometric Dillution of Precision" data-balloon-pos="right" {
                                    "DOP"
                                }
                            }
                            td {
                                (self.dop_plot.render())
                            }
                        }
                        tr {
                            th class="is-info" {
                                button aria-label="Error due to Ionospheric delay" data-balloon-pos="right" {
                                    "Ionosphere"
                                }
                            }
                            td {
                                (self.ionod_plot.render())
                            }
                        }
                        tr {
                            th class="is-info" {
                                button aria-label="Error due to Tropospheric delay" data-balloon-pos="right" {
                                    "Troposphere"
                                }
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
    pub fn new(cfg: &NaviConfig, ctx: &Context, solutions: &BTreeMap<Epoch, PVTSolution>) -> Self {
        Self {
            tab: ReportTab {},
            content: ReportContent::new(cfg, ctx, solutions),
        }
    }
}
