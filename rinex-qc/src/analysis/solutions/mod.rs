// pub mod cggtts;
pub mod ppp;

// use cggtts::QcNavPostCggttsSolutions;
use ppp::QcNavPostPPPSolutions;

use log::error;

use itertools::Itertools;

use crate::{
    cfg::rover::QcPreferedRover, context::meta::MetaData, navigation::cggtts::NavCggttsSolver,
    prelude::QcContext,
};

use std::collections::HashMap;

#[derive(Default)]
pub struct QcNavPostSolutions {
    /// Possibly attached [QcNavPostPPPSolutions] on a "rover" basis
    pub ppp: HashMap<MetaData, QcNavPostPPPSolutions>,
    // /// Possibly attached [QcNavPostCggttsSolutions] on a "rover" basis
    // pub cggtts: HashMap<MetaData, QcNavPostCggttsSolutions>,
}

impl QcNavPostSolutions {
    /// Returns list of rovers that have been "solved"
    fn rovers(&self) -> Vec<String> {
        self.ppp
            .keys()
            .map(|meta| meta.name.to_string())
            .unique()
            .collect::<Vec<_>>()
    }

    /// True is no solutions are attached
    pub fn is_empty(&self) -> bool {
        // self.cggtts.is_empty()
        self.ppp.is_empty()
    }

    /// Create new [QcNavPostSolutions]
    pub fn new(ctx: &QcContext) -> Self {
        let rtk_config = ctx.cfg.rtk_config();

        let mut ppp = HashMap::new();

        let prefered_rover = &ctx.cfg.rover.prefered_rover;

        match prefered_rover {
            QcPreferedRover::Any => Self::new_any_rover(ctx, &mut ppp),
            QcPreferedRover::Prefered(rover) => Self::new_prefered_rover(ctx, &rover, &mut ppp),
        }

        Self { ppp }
    }

    /// Creates new [QcNavPostSolutions] for specific rover in the context pool
    fn new_prefered_rover(
        ctx: &QcContext,
        rover_label: &str,
        ppp: &mut HashMap<MetaData, QcNavPostPPPSolutions>,
        // cggtts: &mut HashMap<MetaData, QcNavPostCggttsSolutions>,
    ) {
        for obs_meta in ctx.rover_observations_meta() {
            if obs_meta.meta.name == rover_label {
                if ctx.cfg.solutions.ppp {
                    match ctx.nav_pvt_solver(ctx.cfg.rtk_config(), obs_meta, None) {
                        Ok(mut solver) => {
                            debug!("solving {} PPP solutions", obs_meta.meta.name);
                            for solution in solver {
                                debug!("{:?}", solution);
                            }
                        },
                        Err(e) => {
                            error!("ppp error: {}", e);
                        },
                    }
                }
                if ctx.cfg.solutions.cggtts {
                    // match ctx.nav_cggtts_solver(
                    //     ctx.cfg.rtk_config(),
                    //     obs_meta,
                    //     None,
                    //     Default::default(),
                    // ) {
                    //     Ok(mut solver) => {
                    //         debug!("attaching {} CGGTTS solutions", obs_meta.meta.name);
                    //         for track in solver {
                    //             debug!("{:?}", track);
                    //         }
                    //     },
                    //     Err(e) => {
                    //         error!("cggtts error: {}", e);
                    //     },
                    // }
                }
            }
        }
    }

    /// Creates new [QcNavPostSolutions] for any rover contained in the context pool
    fn new_any_rover(
        ctx: &QcContext,
        ppp: &mut HashMap<MetaData, QcNavPostPPPSolutions>,
        // cggtts: &mut HashMap<MetaData, QcNavPostCggttsSolutions>,
    ) {
        for obs_meta in ctx.rover_observations_meta() {
            if ctx.cfg.solutions.ppp {
                match ctx.nav_pvt_solver(ctx.cfg.rtk_config(), obs_meta, None) {
                    Ok(mut solver) => {
                        debug!("solving {} PPP solutions", obs_meta.meta.name);
                        for solution in solver {
                            debug!("{:?}", solution);
                        }
                    },
                    Err(e) => {
                        error!("failed to deploy PPP solver: {}", e);
                    },
                }
            }
            if ctx.cfg.solutions.cggtts {
                debug!("integrating {} CGGTTS solutions", obs_meta.meta.name);
            }
        }
    }
}
