pub mod cggtts;
pub mod ppp;

use cggtts::QcNavPostCggttsSolutions;
use ppp::QcNavPostPPPSolutions;

use log::error;

use itertools::Itertools;

use crate::{
    cfg::rover::QcPreferedRover,
    context::meta::MetaData,
    navigation::cggtts::NavCggttsSolver,
    prelude::{html, Markup, QcContext, Render},
};

use std::collections::HashMap;

#[derive(Default)]
pub struct QcNavPostSolutions {
    /// Possibly attached [QcNavPostPPPSolutions] on a "rover" basis
    pub ppp: HashMap<MetaData, QcNavPostPPPSolutions>,

    /// Possibly attached [QcNavPostCggttsSolutions] on a "rover" basis
    pub cggtts: HashMap<MetaData, QcNavPostCggttsSolutions>,
}

impl QcNavPostSolutions {
    /// Returns list of rovers that have been "solved"
    fn rovers(&self) -> Vec<String> {
        self.ppp
            .keys()
            .chain(self.cggtts.keys())
            .map(|meta| meta.name.to_string())
            .unique()
            .collect::<Vec<_>>()
    }

    /// True is no solutions are attached
    pub fn is_empty(&self) -> bool {
        self.cggtts.is_empty() && self.ppp.is_empty()
    }

    /// Create new [QcNavPostSolutions]
    pub fn new(ctx: &QcContext) -> Self {
        let rtk_config = ctx.cfg.rtk_config();

        let mut cggtts = HashMap::new();
        let mut ppp = HashMap::new();

        let prefered_rover = &ctx.cfg.rover.prefered_rover;

        match prefered_rover {
            QcPreferedRover::Any => Self::new_any_rover(ctx, &mut ppp, &mut cggtts),
            QcPreferedRover::Prefered(rover) => {
                Self::new_prefered_rover(ctx, &rover, &mut ppp, &mut cggtts)
            },
        }

        Self { cggtts, ppp }
    }

    /// Creates new [QcNavPostSolutions] for specific rover in the context pool
    fn new_prefered_rover(
        ctx: &QcContext,
        rover_label: &str,
        ppp: &mut HashMap<MetaData, QcNavPostPPPSolutions>,
        cggtts: &mut HashMap<MetaData, QcNavPostCggttsSolutions>,
    ) {
        for obs_meta in ctx.rover_observations_meta() {
            if obs_meta.meta.name == rover_label {
                if ctx.cfg.solutions.ppp {
                    debug!("integrating {} PPP solutions", obs_meta.meta.name);
                }
                if ctx.cfg.solutions.cggtts {
                    debug!("integrating {} CGGTTS solutions", obs_meta.meta.name);
                }
            }
        }
    }

    /// Creates new [QcNavPostSolutions] for any rover contained in the context pool
    fn new_any_rover(
        ctx: &QcContext,
        ppp: &mut HashMap<MetaData, QcNavPostPPPSolutions>,
        cggtts: &mut HashMap<MetaData, QcNavPostCggttsSolutions>,
    ) {
        for obs_meta in ctx.rover_observations_meta() {
            if ctx.cfg.solutions.ppp {
                debug!("integrating {} PPP solutions", obs_meta.meta.name);
            }
            if ctx.cfg.solutions.cggtts {
                debug!("integrating {} CGGTTS solutions", obs_meta.meta.name);
            }
        }
    }
}

impl Render for QcNavPostSolutions {
    fn render(&self) -> Markup {
        html! {
            table class="table is-bordered" {
                tbody {
                    tr {
                        th class="is-info" {
                            "Solutions"
                        }
                        td {
                            select id="qc-navpost-rovers" {
                                @ for rover in self.rovers() {
                                    option value=(rover) {}
                                }
                            }
                        }
                        td {
                            select id="qc-navpost-solutions" {
                                @ if !self.ppp.is_empty() {
                                    option value="PPP" {}
                                }
                                @ if !self.cggtts.is_empty() {
                                    option value="CGGTTS" {}
                                }
                            }
                        }
                    }
                    @ for (rover, solutions) in &self.ppp {
                        tr {
                            th class="is-info" {
                                (format!("{} (PPP)", rover))
                            }
                            td {
                                (solutions.render())
                            }
                        }
                    }
                    @ for (rover, solutions) in &self.cggtts {
                        tr {
                            th class="is-info" {
                                (format!("{} (CGGTTS)", rover))
                            }
                            td {
                                (solutions.render())
                            }
                        }
                    }
                }
            }
        }
    }
}
