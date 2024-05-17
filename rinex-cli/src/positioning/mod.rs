use crate::cli::Context;
use std::cell::RefCell;
use std::fs::read_to_string;

mod ppp; // precise point positioning
use ppp::post_process as ppp_post_process;
use ppp::PostProcessingError as PPPPostProcessingError;

mod cggtts; // CGGTTS special solver
use cggtts::post_process as cggtts_post_process;
use cggtts::PostProcessingError as CGGTTSPostProcessingError;

use clap::ArgMatches;
use gnss::prelude::Constellation; // SV};
use rinex::carrier::Carrier;
use rinex::prelude::{Observable, Rinex};

use rtk::prelude::{
    AprioriPosition, BdModel, Carrier as RTKCarrier, Config, Duration, Epoch, Error as RTKError,
    KbModel, Method, NgModel, PVTSolutionType, Solver, Vector3,
};

use map_3d::{ecef2geodetic, rad2deg, Ellipsoid};
use thiserror::Error;

mod orbit;

mod interp;
use interp::OrbitInterpolator;

#[derive(Debug, Error)]
pub enum Error {
    #[error("solver error")]
    SolverError(#[from] RTKError),
    #[error("undefined apriori position")]
    UndefinedAprioriPosition,
    #[error("ppp post processing error")]
    PPPPostProcessingError(#[from] PPPPostProcessingError),
    #[error("cggtts post processing error")]
    CGGTTSPostProcessingError(#[from] CGGTTSPostProcessingError),
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
        Carrier::E5 | Carrier::E5a | Carrier::E5b => RTKCarrier::E5,
        Carrier::E6 => RTKCarrier::E6,
        Carrier::L1 | _ => RTKCarrier::L1,
    }
}

pub fn tropo_components(meteo: Option<&Rinex>, t: Epoch, lat_ddeg: f64) -> Option<(f64, f64)> {
    const MAX_LATDDEG_DELTA: f64 = 15.0;
    let max_dt = Duration::from_hours(24.0);
    let rnx = meteo?;
    let meteo = rnx.header.meteo.as_ref().unwrap();

    let delays: Vec<(Observable, f64)> = meteo
        .sensors
        .iter()
        .filter_map(|s| match s.observable {
            Observable::ZenithDryDelay => {
                let (x, y, z, _) = s.position?;
                let (lat, _, _) = ecef2geodetic(x, y, z, Ellipsoid::WGS84);
                let lat = rad2deg(lat);
                if (lat - lat_ddeg).abs() < MAX_LATDDEG_DELTA {
                    let value = rnx
                        .zenith_dry_delay()
                        .filter(|(t_sens, _)| (*t_sens - t).abs() < max_dt)
                        .min_by_key(|(t_sens, _)| (*t_sens - t).abs());
                    let (_, value) = value?;
                    debug!("{:?} lat={} zdd {}", t, lat_ddeg, value);
                    Some((s.observable.clone(), value))
                } else {
                    None
                }
            },
            Observable::ZenithWetDelay => {
                let (x, y, z, _) = s.position?;
                let (mut lat, _, _) = ecef2geodetic(x, y, z, Ellipsoid::WGS84);
                lat = rad2deg(lat);
                if (lat - lat_ddeg).abs() < MAX_LATDDEG_DELTA {
                    let value = rnx
                        .zenith_wet_delay()
                        .filter(|(t_sens, _)| (*t_sens - t).abs() < max_dt)
                        .min_by_key(|(t_sens, _)| (*t_sens - t).abs());
                    let (_, value) = value?;
                    debug!("{:?} lat={} zdd {}", t, lat_ddeg, value);
                    Some((s.observable.clone(), value))
                } else {
                    None
                }
            },
            _ => None,
        })
        .collect();

    if delays.len() < 2 {
        None
    } else {
        let zdd = delays
            .iter()
            .filter_map(|(obs, value)| {
                if obs == &Observable::ZenithDryDelay {
                    Some(*value)
                } else {
                    None
                }
            })
            .reduce(|k, _| k)
            .unwrap();

        let zwd = delays
            .iter()
            .filter_map(|(obs, value)| {
                if obs == &Observable::ZenithWetDelay {
                    Some(*value)
                } else {
                    None
                }
            })
            .reduce(|k, _| k)
            .unwrap();

        Some((zwd, zdd))
    }
}

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

pub fn precise_positioning(ctx: &Context, matches: &ArgMatches) -> Result<(), Error> {
    /* Load customized config script, or use defaults */
    let cfg = match matches.get_one::<String>("cfg") {
        Some(fp) => {
            let content = read_to_string(fp)
                .unwrap_or_else(|e| panic!("failed to read configuration: {}", e));
            let mut cfg: Config = serde_json::from_str(&content)
                .unwrap_or_else(|e| panic!("failed to parse configuration: {}", e));

            /*
             * CGGTTS special case
             */
            if matches.get_flag("cggtts") {
                cfg.sol_type = PVTSolutionType::TimeOnly;
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
            if matches.get_flag("cggtts") {
                cfg.sol_type = PVTSolutionType::TimeOnly;
            }

            info!("Using {:?} default preset: {:#?}", method, cfg);
            cfg
        },
    };

    /* Verify requirements and print helpful comments */
    let apriori_ecef = ctx.rx_ecef.ok_or(Error::UndefinedAprioriPosition)?;

    let apriori = Vector3::<f64>::new(apriori_ecef.0, apriori_ecef.1, apriori_ecef.2);
    let apriori = AprioriPosition::from_ecef(apriori);
    let rx_lat_ddeg = apriori.geodetic()[0];

    assert!(
        ctx.data.observation().is_some(),
        "Positioning requires Observation RINEX"
    );
    assert!(
        ctx.data.brdc_navigation().is_some(),
        "Positioning requires Navigation RINEX"
    );

    if cfg.interp_order > 5 && ctx.data.sp3().is_none() {
        error!("High interpolation orders are likely incompatible with navigation based on broadcast radio.");
        warn!("It is possible that this configuration does not generate any solutions.");
        info!("Consider loading high precision SP3 data to use high interpolation orders.");
    }

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
                }
            }
        }
    }

    let orbit = RefCell::new(OrbitInterpolator::from_ctx(
        ctx,
        cfg.interp_order,
        apriori.clone(),
    ));
    debug!("Orbit interpolator created");

    // print config to be used
    info!("Using {:?} method", cfg.method);

    let solver = Solver::new(
        &cfg,
        apriori,
        /* state vector interpolator */
        |t, sv, _order| orbit.borrow_mut().next_at(t, sv),
    )?;

    if matches.get_flag("cggtts") {
        /* CGGTTS special opmode */
        let tracks = cggtts::resolve(ctx, solver, rx_lat_ddeg, matches)?;
        cggtts_post_process(ctx, tracks, matches)?;
    } else {
        /* PPP */
        let pvt_solutions = ppp::resolve(ctx, solver, rx_lat_ddeg);
        /* save solutions (graphs, reports..) */
        ppp_post_process(ctx, pvt_solutions, matches)?;
    }
    Ok(())
}
