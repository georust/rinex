use crate::cli::Context;
use rinex_qc::prelude::{html, Marker, MarkerSymbol, Markup, Mode, Plot, Render};
use rtk::prelude::{Epoch, PVTSolution, SV};
use std::collections::BTreeMap;

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
    /// xy_err
    xy_err_plot: Plot,
    /// z_err
    z_err_plot: Plot,
    /// 3d_plot
    plot_3d: Plot,
    /// DOP
    dop_plot: Plot,
    /// NAVI
    navi_plot: Plot,
}

impl ReportContent {
    pub fn new(ctx: &Context, solutions: &BTreeMap<Epoch, PVTSolution>) -> Self {
        let (rx_lat_rad, rx_long_rad) = (0.0_f64, 0.0_f64); // TODO
        let (rx_lat_ddeg, rx_long_ddeg) = (0.0_f64, 0.0_f64); // TODO
        let epochs = solutions.keys().cloned().collect::<Vec<_>>();

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
            dop_plot: {
                let mut plot =
                    Plot::timedomain_plot("dop", "Dillution of Precision", "Error [m]", true);

                let tdop = solutions
                    .iter()
                    .map(|(_, sol)| sol.tdop)
                    .collect::<Vec<_>>();

                let mut trace = Plot::timedomain_chart(
                    "tdop",
                    Mode::LinesMarkers,
                    MarkerSymbol::Cross,
                    &epochs,
                    tdop,
                );
                plot.add_trace(trace);

                let gdop = solutions
                    .iter()
                    .map(|(_, sol)| sol.gdop)
                    .collect::<Vec<_>>();

                let mut trace = Plot::timedomain_chart(
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

                let mut trace = Plot::timedomain_chart(
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

                let mut trace = Plot::timedomain_chart(
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
            xy_err_plot: {
                let mut plot =
                    Plot::timedomain_plot("xy_plot", "X/Y coordinates", "Error [m]", true);
                plot
            },
            z_err_plot: {
                let mut plot = Plot::timedomain_plot("z_plot", "Z coordinate", "Error [m]", true);
                plot
            },
            plot_3d: {
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
    //pub fn formalize(&self) -> QcExtraPage {
    //    QcExtraPage {
    //        tab: Box::new(&self.tab),
    //        content: Box::new(&self.content),
    //    }
    //}
    pub fn new(ctx: &Context, solutions: &BTreeMap<Epoch, PVTSolution>) -> Self {
        Self {
            tab: ReportTab {},
            content: ReportContent::new(ctx, solutions),
        }
    }
}
