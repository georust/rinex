useirtk::prelude::{Carrier, Config, Epoch, Method, PVTSolution, SV, Mode, Profile};
use std::collections::{BTreeMap, HashMap},
use maud::{Render, Markup, html};

struct ComparisonGraphs {
    err_x: Plot,
    err_y: Plot,
    err_z: Plot,
    err_sphere: Plot,
}

impl ComparisonGraphs {
    fn new(ctx: &Context, epochs: &Vec<Epoch>, solutions: &BTreeMap<Epoch, PVTSolution>) -> Self {
        let (x0, y0, z0) = ctx.rx_ecef;
        Self {
            position: GroundPosition::from_ecef_wgs84(ctx.rx_ecef),
            err_x: {
                let err_x = solutions.value()
                    .map(|p| p.position.x - x0)
                    .collect();
                let mut plot = Plot::new();
                plot.add_trace(trace);
                plot
            },
            err_y: {},
            err_z: {},
            err_sphere: {},
        }
    }
}

impl Render for ComparisonGraphs {
    fn render(&self) -> Markup {
        html! {
            div class="table-container" {
                table class="table is-bordered" {
                    tr {
                        th class="is-info" {
                            "X/Y/Z/3D slider"
                        }
                        td {
                            (self.err_x.render())
                        }
                    }
                }
            }
        }
    }
}

/// Comparison report, in case
/// apriori knowledge was forwarded
struct ComparisonReport {
    position: GroundPosition,
    #[cfg(feature = "plot")]
    graphs: ComparisonGraphs,
}

impl ComparisonReport {
    fn new(ctx: &Context) -> Self {
        Self {
            #[cfg(feature = "plot")]
            graphs: ComparisonGraphs::new(ctx),
            position: GroundPosition::from_ecef_wgs84(ctx.rx_ecef),
        }
    }
}

impl Render for ComparisonReport {
    fn render(&self) -> Markup {
        html! {
            div class="table is-bordered" {
                table class="table is-bordered" {
                    tr {
                        th class="is-info" {
                            "Reference coordinates"
                        }
                        td {
                            (self.position.render())
                        }
                    }
                    @if let Some(graphs) = self.graphs {
                        tr {
                            td {
                                (graphs.render())
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Graph report in case plots eanbled
struct Graphs {
    map_proj: Plot,
    navi_plot: Plot,
    clock_plot: Plot,
    clock_drift_plot: Plot,
    x_plot: Plot,
    y_plot: Plot,
    z_plot: Plot,
    vel_x_plot: Plot,
    vel_y_plot: Plot,
    vel_z_plot: Plot,
    hdop_plot: Plot,
    vdop_plot: Plot,
    gdop_plot: Plot, 
    tdop_plot: Plot, 
}

impl Render for Graphs {
    fn render(&self) -> Markup {
        html! {
            div class="table-container" {
                table class="table is-bordered" {
                    tr {
                        td {
                            (self.map_proj.render())
                        }
                    }
                    tr {
                        td {
                            (self.navi_plot.render())
                        }
                    }
                    tr {
                        th class="is-info" {
                            "Vel X/Y/Z/All slider"
                        }
                        td {
                            (self.x_plot.render())
                        }
                        td {
                            (self.vel_x_plot.render())
                        }
                    }
                    tr {
                        th class="is-info" {
                            "DOP slider"
                        }
                        td {
                            (self.gdop_plot.render())
                        }
                    }
                    tr {
                        th class="is-info" {
                            "Clock/Drift slider"
                        }
                        td {
                            (self.clock_plot.render())
                        }
                    }
                }
            }
        }
    }
}

/// Positioning solutions report
pub struct Report {
    nb_solutions: usize,
    positioning: Mode,
    profile: Profile,
    #[cfg(feature = "plot")]
    graphs: Graphs,
    comparison: Option<ComparisonReport>,
}

impl Render for Report {
    fn render(&self) -> Markup {
        html! {
            div class="table-container" {
                table class="table is-bordered" {
                    tr {
                        th class="is-info" {
                            "Post Processed Position Solutions"
                        }
                    }
                    tr {
                        td {
                            "Number of solutions"
                        }
                    }
                    tr {
                        td {
                            "Positioning"
                        }
                    }
                    tr {
                        td {
                            "Profile"
                        }
                    }
                    tr {
                        td {
                            "Graphs"
                        }
                        td {
                            (self.graphs.render())
                        }
                    }
                    @if let Some(cmp) = self.comparison {
                        tr {
                            td {
                                "Comparison"
                            }
                            td {
                                (cmp.render())
                            }
                        }
                    }
                }
            }
        }
    }
}

impl Report {
    pub fn new(ctx: &Context, cfg: &Config, solutions: &BTreeMap<Epoch, PVTSolution>) -> Self {
        let nb_solutions = solutions.len();
        #[cfg(feature = "plot")]
        let mut graphs = Graphs::new(ctx);
        Self {
            nb_solutions,
            positioning: cfg.mode,
            profile: cfg.profile,
            comparison: if let Some(ecef) = ctx.rx_ecef {
                Some(ComparisonReport::new(ctx, ecef))
            } else {
                None
            },
            #[cfg(feature = "plot")]
            graphs,
        }
    }
}
