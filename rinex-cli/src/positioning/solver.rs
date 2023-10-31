use crate::Cli;
use gnss::prelude::{Constellation, SNR, SV};
use rinex::carrier::Carrier;
use rinex::navigation::Ephemeris;
use rinex::prelude::{Observable, RnxContext};

use rtk::prelude::{
    AprioriPosition, Candidate, Config, Duration, Epoch, Estimate, InterpolationResult, Mode,
    PseudoRange, Solver,
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

    let apriori = AprioriPosition::from_ecef(pos.to_ecef_wgs84().into());

    // print config to be used
    info!("{:#?}", cfg);

    if ctx.sp3_data().is_none() {
        error!("--rtk does not work without SP3 at the moment");
        return Err(Error::MissingSp3Data);
    }

    let mut solver = Solver::new(mode, apriori, &cfg, interpolator, tropo_components)?;

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
    for ((t, flag), (clk, vehicles)) in obs_data.observation() {
        let mut candidates: Vec<Candidate> = Vec::new();

        if !flag.is_ok() {
            /* we only feed "OK" epochs" */
            continue;
        }

        for (sv, observations) in vehicles {
            let sv_eph = nav_data.sv_ephemeris(*sv, *t);
            if sv_eph.is_none() {
                warn!("{:?} ({}) : undetermined ephemeris", t, sv);
                continue; // can't proceed further
            }

            let (toe, sv_eph) = sv_eph.unwrap();

            let tgd = sv_eph.tgd();
            let clock_state = sv_eph.sv_clock();
            let clock_corr = Ephemeris::sv_clock_corr(*sv, clock_state, *t, toe);
            let clock_state: Vector3D = clock_state.into();
            let mut snr = SNR::default();
            let mut pseudo_range = Vec::<PseudoRange>::new();

            for (observable, data) in observations {
                if let Ok(carrier) = Carrier::from_observable(sv.constellation, observable) {
                    if observable.is_pseudorange_observable() {
                        pseudo_range.push(PseudoRange {
                            frequency: carrier.frequency(),
                            value: data.obs,
                        });
                    }
                }
            }
            if let Ok(candidate) =
                Candidate::new(*sv, *t, clock_state, clock_corr, snr, &pseudo_range.clone())
            {
                candidates.push(candidate);
            } else {
                warn!("{:?}: failed to form {} candidate", t, sv);
            }
        }
        match solver.run(*t, candidates) {
            Ok((t, estimate)) => {
                debug!("{:?} : resolved: {:?}", t, estimate);
                ret.insert(t, estimate);
            },
            Err(e) => warn!("{:?} : {}", t, e),
        }
    }
    Ok(ret)
}
