use crate::cli::Context;
use std::collections::BTreeMap;

use rtk::prelude::{
    Config as NaviConfig, Duration, Epoch, Filter as NaviFilter, Method as NaviMethod, PVTSolution,
    TimeScale, SV,
};

use rinex_qc::{
    plot::{MapboxStyle, NamedColor},
    prelude::{html, MarkerSymbol, Markup, Mode, Plot, QcExtraPage, Render},
};

use itertools::Itertools;

use map_3d::{
    //ecef2enu,
    ecef2geodetic,
    geodetic2enu,
    Ellipsoid,
};

struct ReportTab {}

impl Render for ReportTab {
    fn render(&self) -> Markup {
        html! {
            a id="menu:ppp" {
                span class="icon" {
                    i class="fa-solid fa-location-crosshairs" {}
                }
                "PPP Solutions"
            }
        }
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
    orbit: String,
    first_epoch: Epoch,
    last_epoch: Epoch,
    duration: Duration,
    satellites: Vec<SV>,
    timescale: TimeScale,
    final_geo: (f64, f64, f64),
    final_err_m: (f64, f64, f64),
}

impl Render for Summary {
    fn render(&self) -> Markup {
        html! {
            div class="table-container" {
                table class="table is-bordered" {
                    tbody {
                        tr {
                            th class="is-info" {
                                button aria-label=(self.technique.tooltip()) data-balloon-pos="right" {
                                    (self.technique.to_string())
                                }
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
                            }
                            td {
                                (self.orbit)
                            }
                        }
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
                            td {
                                table class="table is-bordered" {
                                    tr {
                                        th class="is-info" {
                                            "WGS84"
                                        }
                                        td {
                                            (format!("x={:.5}°", self.final_geo.0.to_degrees()))
                                        }
                                        td {
                                            (format!("x={:.5}°", self.final_geo.1.to_degrees()))
                                        }
                                        td {
                                            (format!("alt={:.3}°", self.final_geo.2))
                                        }
                                    }
                                    tr {
                                        th class="is-info" {
                                            "Error (m)"
                                        }
                                        td {
                                            (format!("x={:.3E}", self.final_err_m.0))
                                        }
                                        td {
                                            (format!("y={:.3E}", self.final_err_m.1))
                                        }
                                        td {
                                            (format!("z={:.3E}", self.final_err_m.2))
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
}

impl Summary {
    fn new(
        cfg: &NaviConfig,
        ctx: &Context,
        solutions: &BTreeMap<Epoch, PVTSolution>,
        rx_ecef: (f64, f64, f64),
    ) -> Self {
        let (x0, y0, z0) = rx_ecef;
        let (lat0_rad, lon0_rad, alt0_m) = ecef2geodetic(x0, y0, z0, Ellipsoid::WGS84);

        let mut timescale = TimeScale::default();
        let (mut first_epoch, mut last_epoch) = (Epoch::default(), Epoch::default());
        let mut final_err_m = (0.0_f64, 0.0_f64, 0.0_f64);
        let mut final_geo = (0.0_f64, 0.0_f64, 0.0_f64);

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

        for (index, (t, sol)) in solutions.iter().enumerate() {
            if index == 0 {
                first_epoch = *t;
            }
            let (err_x, err_y, err_z) = (
                sol.position.x - x0,
                sol.position.y - y0,
                sol.position.z - z0,
            );
            final_err_m = (err_x, err_y, err_z);
            final_geo = ecef2geodetic(
                sol.position.x,
                sol.position.y,
                sol.position.z,
                Ellipsoid::WGS84,
            );

            last_epoch = *t;
            timescale = sol.timescale;
        }
        Self {
            first_epoch,
            last_epoch,
            timescale,
            satellites,
            final_geo,
            final_err_m,
            orbit: {
                if ctx.data.has_sp3() {
                    format!("Interpolation X{}", cfg.interp_order)
                } else {
                    "Kepler".to_string()
                }
            },
            method: cfg.method,
            filter: cfg.solver.filter,
            duration: last_epoch - first_epoch,
            technique: Technique::GeodeticSurvey,
        }
    }
}

struct ReportContent {
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
        let nb_solutions = solutions.len();
        let epochs = solutions.keys().cloned().collect::<Vec<_>>();

        let (x0_ecef, y0_ecef, z0_ecef) = ctx.rx_ecef.unwrap_or_default();
        let (lat0_rad, lon0_rad, alt0_m) =
            ecef2geodetic(x0_ecef, y0_ecef, z0_ecef, Ellipsoid::WGS84);
        let (lat0_ddeg, lon0_ddeg) = (lat0_rad.to_degrees(), lon0_rad.to_degrees());

        let summary = Summary::new(cfg, ctx, solutions, (x0_ecef, y0_ecef, z0_ecef));

        Self {
            map_proj: {
                let mut map_proj = Plot::world_map(
                    "map_proj",
                    "Map Projection",
                    MapboxStyle::OpenStreetMap,
                    (lat0_ddeg, lon0_ddeg),
                    18,
                    true,
                );
                let apriori = Plot::mapbox(
                    vec![lat0_ddeg],
                    vec![lon0_ddeg],
                    "apriori",
                    MarkerSymbol::Circle,
                    NamedColor::Red,
                    1.0,
                    true,
                );
                map_proj.add_trace(apriori);
                let mut prev_pct = 0;
                for (index, (_, sol_i)) in solutions.iter().enumerate() {
                    let pct = index * 100 / nb_solutions;
                    if pct % 10 == 0 && index > 0 && pct != prev_pct || index == nb_solutions - 1 {
                        let (name, visible) = if index == nb_solutions - 1 {
                            ("FINAL".to_string(), true)
                        } else {
                            (format!("Solver: {:02}%", pct), false)
                        };
                        let (lat_rad, lon_rad, _) = ecef2geodetic(
                            sol_i.position.x,
                            sol_i.position.y,
                            sol_i.position.z,
                            Ellipsoid::WGS84,
                        );
                        let (lat_ddeg, lon_ddeg) = (lat_rad.to_degrees(), lon_rad.to_degrees());
                        let scatter = Plot::mapbox(
                            vec![lat_ddeg],
                            vec![lon_ddeg],
                            &name,
                            MarkerSymbol::Circle,
                            NamedColor::Black,
                            1.0,
                            visible,
                        );
                        map_proj.add_trace(scatter);
                    }
                    prev_pct = pct;
                }
                map_proj
            },
            sv_plot: {
                let mut plot = Plot::timedomain_plot("sv_plot", "SV ID#", "PRN #", true);
                for sv in summary.satellites.iter() {
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
                    "North / East / Up Coordinates",
                    "Coordinates [m]",
                    true,
                );
                let neu = solutions
                    .iter()
                    .map(|(_, sol)| {
                        let (lat_rad, lon_rad, alt_m) = ecef2geodetic(
                            sol.position.x,
                            sol.position.y,
                            sol.position.z,
                            Ellipsoid::WGS84,
                        );
                        let enu = geodetic2enu(
                            lat_rad,
                            lon_rad,
                            alt_m,
                            lat0_rad,
                            lon0_rad,
                            alt0_m,
                            Ellipsoid::WGS84,
                        );
                        (enu.1.to_degrees(), enu.0.to_degrees(), enu.2)
                    })
                    .collect::<Vec<_>>();
                let north = neu.iter().map(|neu| neu.0).collect::<Vec<_>>();
                let east = neu.iter().map(|neu| neu.1).collect::<Vec<_>>();
                let up = neu.iter().map(|neu| neu.2).collect::<Vec<_>>();
                let trace = Plot::timedomain_chart(
                    "north",
                    Mode::Markers,
                    MarkerSymbol::Cross,
                    &epochs,
                    north,
                );
                plot.add_trace(trace);
                let trace = Plot::timedomain_chart(
                    "east",
                    Mode::Markers,
                    MarkerSymbol::Cross,
                    &epochs,
                    east,
                );
                plot.add_trace(trace);
                let trace =
                    Plot::timedomain_chart("upt", Mode::Markers, MarkerSymbol::Cross, &epochs, up);
                plot.add_trace(trace);
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
                let trace =
                    Plot::timedomain_chart("vel_x", Mode::Markers, MarkerSymbol::Cross, &epochs, x);
                plot.add_trace(trace);
                let trace =
                    Plot::timedomain_chart("vel_y", Mode::Markers, MarkerSymbol::Cross, &epochs, y);
                plot.add_trace(trace);
                let trace =
                    Plot::timedomain_chart("vel_z", Mode::Markers, MarkerSymbol::Cross, &epochs, z);
                plot.add_trace(trace);
                plot
            },
            tropod_plot: {
                let mut plot =
                    Plot::timedomain_plot("tropo", "Troposphere Bias", "Error [m]", true);
                for sv in summary.satellites.iter() {
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
                for sv in summary.satellites.iter() {
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
                    Mode::Markers,
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
                    Mode::Markers,
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
                    Mode::Markers,
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
                    Mode::Markers,
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
                    Mode::Markers,
                    MarkerSymbol::Cross,
                    &epochs,
                    dt,
                );
                plot.add_trace(trace);
                plot
            },
            coords_err_plot: {
                let mut plot = Plot::timedomain_plot("xy_plot", "X/Y/Z Error", "Error [m]", true);
                let trace = Plot::timedomain_chart(
                    "x err",
                    Mode::Markers,
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
                    Mode::Markers,
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
                    Mode::Markers,
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
                let trace = Plot::chart_3d(
                    "Error",
                    Mode::Markers,
                    MarkerSymbol::Cross,
                    &epochs,
                    solutions
                        .values()
                        .map(|sol| sol.position.x - x0_ecef)
                        .collect(),
                    solutions
                        .values()
                        .map(|sol| sol.position.y - y0_ecef)
                        .collect(),
                    solutions
                        .values()
                        .map(|sol| sol.position.z - z0_ecef)
                        .collect(),
                );
                plot.add_trace(trace);
                plot
            },
            navi_plot: {
                let mut plot = Plot::timedomain_plot("navi_plot", "NAVI Plot", "Error [m]", true);
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
                                "Map Proj"
                            }
                            td {
                                (self.map_proj.render())
                            }
                        }
                        tr {
                            th class="is-info" {
                                "NAVI Plot"
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
                                "DOP"
                            }
                            td {
                                div class="table-container" {
                                    table class="table is-bordered" {
                                        tr {
                                            th class="is-info" {
                                                "Geometric DOP"
                                            }
                                            td {
                                                (self.dop_plot.render())
                                            }
                                        }
                                        tr {
                                            th class="is-info" {
                                                "Temporal DOP"
                                            }
                                            td {
                                                (self.tdop_plot.render())
                                            }
                                        }
                                        button aria-label="Geometric Dillution of Precision" data-balloon-pos="right" {
                                        }
                                    }
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
            html_id: "ppp".to_string(),
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
