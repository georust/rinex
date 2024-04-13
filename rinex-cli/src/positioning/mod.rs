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
use rinex::navigation::Ephemeris;
use rinex::prelude::{Observable, Rinex};

use rtk::prelude::{
    AprioriPosition, BdModel, Config, Duration, Epoch, InterpolationResult, KbModel, Method,
    NgModel, Solver, Vector3,
};

use map_3d::{ecef2geodetic, rad2deg, Ellipsoid};
use thiserror::Error;

mod interp;
// use interp::TimeInterpolator;
use interp::OrbitInterpolator;

#[derive(Debug, Error)]
pub enum Error {
    #[error("solver error")]
    SolverError(#[from] rtk::Error),
    #[error("undefined apriori position")]
    UndefinedAprioriPosition,
    #[error("ppp post processing error")]
    PPPPostProcessingError(#[from] PPPPostProcessingError),
    #[error("cggtts post processing error")]
    CGGTTSPostProcessingError(#[from] CGGTTSPostProcessingError),
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
    let kb_model = nav
        .klobuchar_models()
        .min_by_key(|(t_i, _, _)| (t - *t_i).abs());

    if let Some((_, sv, kb_model)) = kb_model {
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
    } else {
        /* RINEX 3 case */
        let iono_corr = nav.header.ionod_correction?;
        iono_corr.as_klobuchar().map(|kb_model| KbModel {
            h_km: 350.0, //TODO improve this
            alpha: kb_model.alpha,
            beta: kb_model.beta,
        })
    }
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
    let method = match matches.get_flag("spp") {
        true => Method::SPP,
        false => Method::PPP,
    };

    let cfg = match matches.get_one::<String>("cfg") {
        Some(fp) => {
            let content = read_to_string(fp)
                .unwrap_or_else(|_| panic!("failed to read configuration: permission denied"));
            let cfg = serde_json::from_str(&content)
                .unwrap_or_else(|_| panic!("failed to parse configuration: invalid content"));
            info!("using custom solver configuration: {:#?}", cfg);
            cfg
        },
        None => {
            let cfg = Config::preset(method);
            info!("using default {:?} solver preset: {:#?}", method, cfg);
            cfg
        },
    };

    /*
     * verify requirements
     */
    let apriori_ecef = ctx.rx_ecef.ok_or(Error::UndefinedAprioriPosition)?;

    let apriori = Vector3::<f64>::new(apriori_ecef.0, apriori_ecef.1, apriori_ecef.2);
    let apriori = AprioriPosition::from_ecef(apriori);
    let rx_lat_ddeg = apriori.geodetic[0];

    if ctx.data.observation().is_none() {
        panic!("positioning requires Observation RINEX");
    }

    let nav_data = ctx
        .data
        .brdc_navigation()
        .expect("positioning requires Navigation RINEX");

    let sp3_data = ctx.data.sp3();
    if sp3_data.is_none() {
        panic!("High precision orbits (SP3) are unfortunately mandatory at the moment..");
    }

    let mut orbit = RefCell::new(OrbitInterpolator::from_ctx(
        &ctx,
        cfg.interp_order,
        apriori.clone(),
    ));
    debug!("orbit interpolator created");

    // print config to be used
    info!("Using solver {:?} method", method);

    let solver = Solver::new(
        &cfg,
        apriori,
        /* state vector interpolator */
        |t, sv, order| orbit.borrow_mut().next_at(t, sv, order),
        /* APC corrections provider */
        |_t, _sv, _freq| None,
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
