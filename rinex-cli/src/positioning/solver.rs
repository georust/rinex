use crate::Cli;
use gnss::prelude::{Constellation, SV};
use rinex::prelude::{Observable, RnxContext};
use rtk::prelude::{
    Candidate, Config, Duration, Epoch, Estimate, InterpolationResult, Mode, Solver,
};

use rtk::{model::TropoComponents, Vector3D};

use map_3d::{ecef2geodetic, Ellipsoid};
use thiserror::Error;

use std::collections::HashMap;

#[derive(Debug, Error)]
pub enum Error {
    #[error("solver error")]
    SolverError(#[from] rtk::Error),
    #[error("missing observations")]
    MissingObservationData,
    #[error("missing brdc navigation")]
    MissingBroadcastNavigationData,
    #[error("positioning requires overlapped SP3 data at the moment")]
    MissingSp3Data,
    #[error("undefined apriori position")]
    UndefinedAprioriPosition,
}

fn interpolator(t: Epoch, sv: SV, order: usize) -> Option<InterpolationResult> {
    None
}

fn tropo_components(t: Epoch, lat_ddeg: f64, h_sea: f64) -> Option<TropoComponents> {
    None
}

//fn tropo_model_components(ctx: &RnxContext, t: &Epoch) -> Option<TropoComponents> {
//    const max_latitude_delta: f64 = 15.0_f64;
//    let max_dt = Duration::from_hours(24.0);
//    let rnx = ctx.meteo_data()?;
//    let meteo = meteo.header.meteo.as_ref().unwrap();
//
//    let delays: Vec<(Observable, f64)> = meteo
//        .sensors
//        .iter()
//        .filter_map(|s| {
//            match s.observable {
//                Observable::ZenithDryDelay => {
//                    let (x, y, z, _) = s.position?;
//                    let (lat, _, _) = ecef2geodetic(x, y, z, Ellipsoid::WGS84);
//                    if (lat - lat_ddeg).abs() < max_latitude_delta {
//                        let value = rnx.zenith_dry_delay()
//                            .filter(|(t_sens, value)| (*t_sens - t).abs() < max_dt)
//                            .min_by_key(|(t_sens, _)| (*t_sens - t).abs());
//                        let (_, value) = value?;
//                        debug!("(meteo) zdd @ {}: {}", lat_ddeg, value);
//                        Some((s.observable.clone(), value))
//                    } else {
//                        None /* not within latitude tolerance */
//                    }
//                },
//                Observable::ZenithWetDelay => {
//                    let (x, y, z, _) = s.position?;
//                    let (lat, _, _) = ecef2geodetic(x, y, z, Ellipsoid::WGS84);
//                    if (lat - lat_ddeg).abs() < max_latitude_delta {
//                        let value = rnx.zenith_wet_delay()
//                            .filter(|(t_sens, value)| (*t_sens - t).abs() < max_dt)
//                            .min_by_key(|(t_sens, _)| (*t_sens - t).abs());
//                        let (_, value) = value?;
//                        debug!("(meteo) zwd @ {}: {}", lat_ddeg, value);
//                        Some((s.observable.clone(), value))
//                    } else {
//                        None /* not within latitude tolerance */
//                    }
//                },
//                _ => None,
//            }
//        })
//        .collect();
//    if delays.len() < 2 {
//        trace!("meteo data out of tolerance");
//        None
//    } else {
//        Some(TropoComponents {
//            zdd: {
//                delays
//                    .iter()
//                    .filter_map(|(obs, value)| {
//                        if obs == &Observable::ZenithDryDelay {
//                            Some(value)
//                        } else {
//                            None
//                        }
//                    })
//                    .reduce(|k, _| k)
//            },
//            zwd: {
//                delays
//                    .iter()
//                    .filter_map(|(obs, value)| {
//                        if obs == &Observable::ZenithWetDelay {
//                            Some(value)
//                        } else {
//                            None
//                        }
//                    })
//                    .reduce(|k, _| k)
//            },
//        })
//    }
//}

pub fn solver(ctx: &mut RnxContext, cli: &Cli) -> Result<HashMap<Epoch, Estimate>, Error> {
    // parse custom config, if any
    let cfg = match cli.rtk_config() {
        Some(cfg) => cfg,
        None => Config::default(Mode::SPP),
    };

    let mode = match cli.spp() {
        true => {
            info!("spp position solver");
            Mode::SPP
        },
        false => Mode::SPP, //TODO
    };

    let pos = match cli.manual_position() {
        Some(pos) => pos,
        None => ctx
            .ground_position()
            .ok_or(Error::UndefinedAprioriPosition)?,
    };

    let ecef = pos.to_ecef_wgs84();

    let apriori = Vector3D {
        x: ecef.0,
        y: ecef.1,
        z: ecef.2,
    };

    // print config to be used
    info!("{:#?}", cfg);

    if ctx.sp3_data().is_none() {
        error!("--rtk does not work without SP3 at the moment");
        return Err(Error::MissingSp3Data);
    }

    let mut solver = Solver::new(mode, apriori, &cfg, interpolator, tropo_components);

    let obs_data = match ctx.obs_data() {
        Some(obs_data) => obs_data,
        None => {
            return Err(Error::MissingObservationData);
        },
    };

    let nav_data = match ctx.nav_data() {
        Some(nav_data) => nav_data,
        None => {
            return Err(Error::MissingBroadcastNavigationData);
        },
    };

    let mut ret: HashMap<Epoch, Estimate> = HashMap::new();
    for ((t, flag), (clk, observations)) in obs_data.observation() {
        let mut candidates: Vec<Candidate> = Vec::new();
        //for sv in svs {
        //    let ephemeris = nav_data.ephemeris()?;
        //    let clock_state = None;
        //    let clock_corr = None;
        //    candidates.push(Candidate::new(sv, t, clock_state, clock_corr, snr, pseudo_range));
        //}
        //if let Ok((t, estimate)) = solver.run(epoch, candidates) {
        //    ret.insert(t, estimate);
        //}
    }
    Ok(ret)
}
