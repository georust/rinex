use crate::Cli;
use statrs::statistics::Statistics;

use gnss::prelude::{Constellation, SV};
use rinex::carrier::Carrier;
use rinex::navigation::Ephemeris;
use rinex::observation::SNR;
use rinex::prelude::{Observable, Rinex, RnxContext};

use rtk::{
    model::TropoComponents,
    prelude::{
        AprioriPosition, Candidate, Config, Duration, Epoch, InterpolationResult, Mode,
        PVTSolution, PseudoRange, Solver,
    },
    Vector3D,
};

use map_3d::{ecef2geodetic, Ellipsoid};
use std::collections::HashMap;
use thiserror::Error;

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

fn tropo_components(meteo: Option<&Rinex>, t: Epoch, lat_ddeg: f64) -> Option<TropoComponents> {
    const max_latitude_delta: f64 = 15.0;
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
                if (lat - lat_ddeg).abs() < max_latitude_delta {
                    let value = rnx
                        .zenith_dry_delay()
                        .filter(|(t_sens, value)| (*t_sens - t).abs() < max_dt)
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
                let (lat, _, _) = ecef2geodetic(x, y, z, Ellipsoid::WGS84);
                if (lat - lat_ddeg).abs() < max_latitude_delta {
                    let value = rnx
                        .zenith_wet_delay()
                        .filter(|(t_sens, value)| (*t_sens - t).abs() < max_dt)
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
        Some(TropoComponents {
            zdd: {
                delays
                    .iter()
                    .filter_map(|(obs, value)| {
                        if obs == &Observable::ZenithDryDelay {
                            Some(*value)
                        } else {
                            None
                        }
                    })
                    .reduce(|k, _| k)
                    .unwrap()
            },
            zwd: {
                delays
                    .iter()
                    .filter_map(|(obs, value)| {
                        if obs == &Observable::ZenithWetDelay {
                            Some(*value)
                        } else {
                            None
                        }
                    })
                    .reduce(|k, _| k)
                    .unwrap()
            },
        })
    }
}

pub fn solver(ctx: &mut RnxContext, cli: &Cli) -> Result<HashMap<Epoch, PVTSolution>, Error> {
    // parse custom config, if any
    let cfg = match cli.config() {
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

    let apriori_ecef_wgs84 = pos.to_ecef_wgs84();
    let apriori = AprioriPosition::from_ecef(apriori_ecef_wgs84.into());
    let lat_ddeg = apriori.geodetic.x;

    // print config to be used
    info!("{:#?}", cfg);

    let obs_data = match ctx.obs_data() {
        Some(data) => data,
        None => {
            return Err(Error::MissingObservationData);
        },
    };

    let nav_data = match ctx.nav_data() {
        Some(data) => data,
        None => {
            return Err(Error::MissingBroadcastNavigationData);
        },
    };

    let sp3_data = ctx.sp3_data();
    let meteo_data = ctx.meteo_data();

    let mut solver = Solver::new(
        mode,
        apriori,
        &cfg,
        /* state vector interpolator */
        |t, sv, order| {
            /* SP3 source is prefered */
            if let Some(sp3) = sp3_data {
                if let Some(vec3d) = sp3.sv_position_interpolate(sv, t, order) {
                    let (elev, azi) = Ephemeris::elevation_azimuth(vec3d, apriori_ecef_wgs84);
                    Some(InterpolationResult {
                        sky_pos: vec3d.into(),
                        elevation: Some(elev),
                        azimuth: Some(azi),
                    })
                } else {
                    // debug!("{:?} ({}): sp3 interpolation failed", t, sv);
                    if let Some(vec3d) = nav_data.sv_position_interpolate(sv, t, order) {
                        let (elev, azi) = Ephemeris::elevation_azimuth(vec3d, apriori_ecef_wgs84);
                        Some(InterpolationResult {
                            sky_pos: vec3d.into(),
                            elevation: Some(elev),
                            azimuth: Some(azi),
                        })
                    } else {
                        // debug!("{:?} ({}): nav interpolation failed", t, sv);
                        None
                    }
                }
            } else {
                if let Some(vec3d) = nav_data.sv_position_interpolate(sv, t, order) {
                    let (elev, azi) = Ephemeris::elevation_azimuth(vec3d, apriori_ecef_wgs84);
                    Some(InterpolationResult {
                        sky_pos: vec3d.into(),
                        elevation: Some(elev),
                        azimuth: Some(azi),
                    })
                } else {
                    // debug!("{:?} ({}): nav interpolation failed", t, sv);
                    None
                }
            }
        },
    )?;

    let mut ret: HashMap<Epoch, PVTSolution> = HashMap::new();
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
            let mut snr = Vec::<f64>::new();
            let mut pseudo_range = Vec::<PseudoRange>::new();

            for (observable, data) in observations {
                if let Ok(carrier) = Carrier::from_observable(sv.constellation, observable) {
                    if observable.is_pseudorange_observable() {
                        pseudo_range.push(PseudoRange {
                            frequency: carrier.frequency(),
                            value: data.obs,
                        });
                        if let Some(obs_snr) = data.snr {
                            snr.push(obs_snr.into());
                        }
                    }
                }
            }

            /* worst SNR over all used observations */
            let snr: Option<f64> = match snr.is_empty() {
                true => None,
                false => {
                    let snr = snr.min();
                    debug!("{:?} ({}): SNR : {:?}", t, sv, snr);
                    Some(snr)
                },
            };

            if let Ok(candidate) =
                Candidate::new(*sv, *t, clock_state, clock_corr, snr, pseudo_range.clone())
            {
                candidates.push(candidate);
            } else {
                warn!("{:?}: failed to form {} candidate", t, sv);
            }
        }
        let tropo_components = tropo_components(meteo_data, *t, lat_ddeg);

        match solver.run(*t, candidates, tropo_components) {
            Ok((t, estimate)) => {
                debug!("{:?} : {:?}", t, estimate);
                ret.insert(t, estimate);
            },
            Err(e) => warn!("{:?} : {}", t, e),
        }
    }
    Ok(ret)
}
