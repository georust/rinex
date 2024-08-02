use crate::cli::{Cli, Context};
use clap::ArgMatches;
use std::fs::read_to_string;

mod ppp; // precise point positioning
use ppp::{
    post_process::{post_process as ppp_post_process, Error as PPPPostError},
    Report as PPPReport,
};

#[cfg(feature = "cggtts")]
mod cggtts; // CGGTTS special solver

#[cfg(feature = "cggtts")]
use cggtts::{post_process as cggtts_post_process, Report as CggttsReport};

mod rtk;
use rtk::BaseStation;

use rinex::{
    carrier::Carrier,
    prelude::{Constellation, Rinex},
};

use rinex_qc::prelude::QcExtraPage;

use gnss_rtk::prelude::{
    Almanac, BdModel, Carrier as RTKCarrier, Config, Duration, Epoch, Error as RTKError, KbModel,
    Method, NgModel, PVTSolutionType, Position, Solver, Vector3,
};

use thiserror::Error;

mod orbit;
pub use orbit::Orbit;

mod time;
pub use time::Time;

mod interp;
pub use interp::Buffer as BufferTrait;

#[derive(Debug, Error)]
pub enum Error {
    #[error("solver error")]
    SolverError(#[from] RTKError),
    #[error("no solutions: check your settings or input")]
    NoSolutions,
    #[error("i/o error")]
    StdioError(#[from] std::io::Error),
    #[error("post process error")]
    PPPPost(#[from] PPPPostError),
}

/*
 * Converts `RTK Carrier` into compatible struct
 */
pub fn rtk_carrier_cast(carrier: RTKCarrier) -> Carrier {
    match carrier {
        RTKCarrier::L2 => Carrier::L2,
        RTKCarrier::L5 => Carrier::L5,
        RTKCarrier::L6 => Carrier::L6,
        RTKCarrier::E1 => Carrier::E1,
        RTKCarrier::E5 => Carrier::E5,
        RTKCarrier::E6 => Carrier::E6,
        RTKCarrier::E5A => Carrier::E5a,
        RTKCarrier::E5B => Carrier::E5b,
        RTKCarrier::B1I => Carrier::B1I,
        RTKCarrier::B2 => Carrier::B2,
        RTKCarrier::B3 => Carrier::B3,
        RTKCarrier::B2A => Carrier::B2A,
        RTKCarrier::B2iB2b => Carrier::B2I,
        RTKCarrier::B1aB1c => Carrier::B1A,
        RTKCarrier::L1 => Carrier::L1,
    }
}

/*
 * Converts `Carrier` into RTK compatible struct
 */
pub fn cast_rtk_carrier(carrier: Carrier) -> RTKCarrier {
    match carrier {
        Carrier::L2 => RTKCarrier::L2,
        Carrier::L5 => RTKCarrier::L5,
        Carrier::L6 => RTKCarrier::L6,
        Carrier::E1 => RTKCarrier::E1,
        Carrier::E5 => RTKCarrier::E5,
        Carrier::E6 => RTKCarrier::E6,
        Carrier::E5a => RTKCarrier::E5A,
        Carrier::E5b => RTKCarrier::E5B,
        Carrier::B1I => RTKCarrier::B1I,
        Carrier::B2 => RTKCarrier::B2,
        Carrier::B3 | Carrier::B3A => RTKCarrier::B3,
        Carrier::B2A => RTKCarrier::B2A,
        Carrier::B2I | Carrier::B2B => RTKCarrier::B2iB2b,
        Carrier::B1A | Carrier::B1C => RTKCarrier::B1aB1c,
        Carrier::L1 | _ => RTKCarrier::L1,
    }
}

// helper in reference signal determination
fn rtk_reference_carrier(carrier: RTKCarrier) -> bool {
    matches!(
        carrier,
        RTKCarrier::L1 | RTKCarrier::E1 | RTKCarrier::B1aB1c | RTKCarrier::B1I
    )
}

//use map_3d::{ecef2geodetic, rad2deg, Ellipsoid};

//pub fn tropo_components(meteo: Option<&Rinex>, t: Epoch, lat_ddeg: f64) -> Option<(f64, f64)> {
//    const MAX_LATDDEG_DELTA: f64 = 15.0;
//    let max_dt = Duration::from_hours(24.0);
//    let rnx = meteo?;
//    let meteo = rnx.header.meteo.as_ref().unwrap();
//
//    let delays: Vec<(Observable, f64)> = meteo
//        .sensors
//        .iter()
//        .filter_map(|s| match s.observable {
//            Observable::ZenithDryDelay => {
//                let (x, y, z, _) = s.position?;
//                let (lat, _, _) = ecef2geodetic(x, y, z, Ellipsoid::WGS84);
//                let lat = rad2deg(lat);
//                if (lat - lat_ddeg).abs() < MAX_LATDDEG_DELTA {
//                    let value = rnx
//                        .zenith_dry_delay()
//                        .filter(|(t_sens, _)| (*t_sens - t).abs() < max_dt)
//                        .min_by_key(|(t_sens, _)| (*t_sens - t).abs());
//                    let (_, value) = value?;
//                    debug!("{:?} lat={} zdd {}", t, lat_ddeg, value);
//                    Some((s.observable.clone(), value))
//                } else {
//                    None
//                }
//            },
//            Observable::ZenithWetDelay => {
//                let (x, y, z, _) = s.position?;
//                let (mut lat, _, _) = ecef2geodetic(x, y, z, Ellipsoid::WGS84);
//                lat = rad2deg(lat);
//                if (lat - lat_ddeg).abs() < MAX_LATDDEG_DELTA {
//                    let value = rnx
//                        .zenith_wet_delay()
//                        .filter(|(t_sens, _)| (*t_sens - t).abs() < max_dt)
//                        .min_by_key(|(t_sens, _)| (*t_sens - t).abs());
//                    let (_, value) = value?;
//                    debug!("{:?} lat={} zdd {}", t, lat_ddeg, value);
//                    Some((s.observable.clone(), value))
//                } else {
//                    None
//                }
//            },
//            _ => None,
//        })
//        .collect();
//
//    if delays.len() < 2 {
//        None
//    } else {
//        let zdd = delays
//            .iter()
//            .filter_map(|(obs, value)| {
//                if obs == &Observable::ZenithDryDelay {
//                    Some(*value)
//                } else {
//                    None
//                }
//            })
//            .reduce(|k, _| k)
//            .unwrap();
//
//        let zwd = delays
//            .iter()
//            .filter_map(|(obs, value)| {
//                if obs == &Observable::ZenithWetDelay {
//                    Some(*value)
//                } else {
//                    None
//                }
//            })
//            .reduce(|k, _| k)
//            .unwrap();
//
//        Some((zwd, zdd))
//    }
//}

/*
 * Grabs nearest KB model (in time)
 */
pub fn kb_model(nav: &Rinex, t: Epoch) -> Option<KbModel> {
    let (_, sv, kb_model) = nav
        .klobuchar_models()
        .min_by_key(|(t_i, _, _)| (t - *t_i).abs())?;
    Some(KbModel {
        h_km: {
            match sv.constellation {
                Constellation::BeiDou => 375.0,
                // we only expect GPS or BDS here,
                // badly formed RINEX will generate errors in the solutions
                _ => 350.0,
            }
        },
        alpha: kb_model.alpha,
        beta: kb_model.beta,
    })
}

/*
 * Grabs nearest BD model (in time)
 */
pub fn bd_model(nav: &Rinex, t: Epoch) -> Option<BdModel> {
    nav.bdgim_models()
        .min_by_key(|(t_i, _)| (t - *t_i).abs())
        .map(|(_, model)| BdModel { alpha: model.alpha })
}

/*
 * Grabs nearest NG model (in time)
 */
pub fn ng_model(nav: &Rinex, t: Epoch) -> Option<NgModel> {
    nav.nequick_g_models()
        .min_by_key(|(t_i, _)| (t - *t_i).abs())
        .map(|(_, model)| NgModel { a: model.a })
}

pub fn precise_positioning(
    cli: &Cli,
    ctx: &Context,
    is_rtk: bool,
    matches: &ArgMatches,
) -> Result<QcExtraPage, Error> {
    // Load custom configuration script, or Default
    let cfg = match matches.get_one::<String>("cfg") {
        Some(fp) => {
            let content = read_to_string(fp)
                .unwrap_or_else(|e| panic!("failed to read configuration: {}", e));
            let mut cfg: Config = serde_json::from_str(&content)
                .unwrap_or_else(|e| panic!("failed to parse configuration: {}", e));

            /*
             * CGGTTS special case
             */
            #[cfg(feature = "cggtts")]
            if matches.get_flag("cggtts") {
                cfg.sol_type = PVTSolutionType::TimeOnly;
            }
            #[cfg(not(feature = "cggtts"))]
            if matches.get_flag("cggtts") {
                panic!("--cggtts option not available: compile with cggtts option");
            }

            info!("Using custom solver configuration: {:#?}", cfg);
            cfg
        },
        None => {
            let method = Method::default();
            let mut cfg = Config::static_preset(method);

            /*
             * CGGTTS special case
             */
            #[cfg(feature = "cggtts")]
            if matches.get_flag("cggtts") {
                cfg.sol_type = PVTSolutionType::TimeOnly;
            }
            #[cfg(not(feature = "cggtts"))]
            if matches.get_flag("cggtts") {
                panic!("--cggtts option not available: compile with cggtts option");
            }

            info!("Using {:?} default preset: {:#?}", method, cfg);
            cfg
        },
    };
    /* Verify requirements and print helpful comments */
    assert!(
        ctx.data.observation().is_some(),
        "Positioning requires Observation RINEX"
    );
    assert!(
        ctx.data.brdc_navigation().is_some(),
        "Positioning requires Navigation RINEX"
    );

    if let Some(obs_rinex) = ctx.data.observation() {
        if let Some(obs_header) = &obs_rinex.header.obs {
            if let Some(time_of_first_obs) = obs_header.time_of_first_obs {
                if let Some(clk_rinex) = ctx.data.clock() {
                    if let Some(clk_header) = &clk_rinex.header.clock {
                        if let Some(time_scale) = clk_header.timescale {
                            if time_scale == time_of_first_obs.time_scale {
                                info!("Temporal PPP compliancy");
                            } else {
                                error!("Working with different timescales in OBS/CLK RINEX is not PPP compatible and will generate tiny errors");
                                warn!("Consider using OBS/CLK RINEX files expressed in the same timescale for optimal results");
                            }
                        }
                    }
                } else if let Some(sp3) = ctx.data.sp3() {
                    if ctx.data.sp3_has_clock() {
                        if sp3.time_scale == time_of_first_obs.time_scale {
                            info!("Temporal PPP compliancy");
                        } else {
                            error!("Working with different timescales in OBS/SP3 is not PPP compatible and will generate tiny errors");
                            if sp3.epoch_interval >= Duration::from_seconds(300.0) {
                                warn!("Interpolating clock states from low sample rate SP3 will most likely introduce errors");
                            }
                        }
                    }
                }
            }
        }
    }

    // print config to be used
    info!("Using {:?} method", cfg.method);

    let almanac =
        Almanac::until_2035().unwrap_or_else(|e| panic!("failed to build Almanac: {}", e));

    let orbits = Orbit::from_ctx(ctx, cfg.interp_order, almanac);
    debug!("Orbits created");

    // The CGGTTS opmode (TimeOnly) is not designed
    // to support lack of apriori knowledge
    #[cfg(feature = "cggtts")]
    let apriori = if matches.get_flag("cggtts") {
        if let Some((x, y, z)) = ctx.rx_ecef {
            let apriori_ecef = Vector3::new(x, y, z);
            Some(Position::from_ecef(apriori_ecef))
        } else {
            panic!(
                "--cggtts opmode cannot work without a priori position knowledge.
You either need to specify it manually (see --help), or use RINEX files that define
a static reference position"
            );
        }
    } else {
        None
    };

    #[cfg(not(feature = "cggtts"))]
    let apriori = None;

    //let almanac = Almanac::until_2035()
    //    .unwrap_or_else(|e| panic!("failed to retrieve latest Almanac: {}", e));

    let solver = if is_rtk {
        let base_station = BaseStation::from_ctx(ctx);
        Solver::rtk(&cfg, apriori, orbits, base_station)
            .unwrap_or_else(|e| panic!("failed to deploy RTK solver: {}", e))
    } else {
        Solver::ppp(&cfg, apriori, orbits)
            .unwrap_or_else(|e| panic!("failed to deploy PPP solver: {}", e))
    };

    #[cfg(feature = "cggtts")]
    if matches.get_flag("cggtts") {
        //* CGGTTS special opmode */
        let tracks = cggtts::resolve(ctx, solver, matches)?;
        if !tracks.is_empty() {
            cggtts_post_process(&ctx, &tracks, matches)?;
            let report = CggttsReport::new(&ctx, &tracks);
            return Ok(report.formalize());
        } else {
            error!("solver did not generate a single solution");
            error!("verify your input data and configuration setup");
            return Err(Error::NoSolutions);
        }
    }

    /* PPP */
    let solutions = ppp::resolve(ctx, solver);
    if !solutions.is_empty() {
        ppp_post_process(&ctx, &solutions, matches)?;
        let report = PPPReport::new(&cfg, &ctx, &solutions);
        Ok(report.formalize())
    } else {
        error!("solver did not generate a single solution");
        error!("verify your input data and configuration setup");
        Err(Error::NoSolutions)
    }
}
