use thiserror::Error;
use rinex::prelude::RnxContext;
use rtk::prelude::{
    Config, 
    Estimate, Mode, InterpolationResult,
    model::TropoComponents,
};

#[derive(Debug, Error)]
pub enum Error {
    #[error("rtk error")]
    RTKError(#[from] rtk::Error),
    #[error("missing observations")]
    MissingObservationData,
    #[error("missing brdc navigation")]
    MissingBroadcastNavigationData,
}

fn interpolator(sv: SV, t: Epoch, order: usize) -> Option<InterpolationResult> {
    None
}

fn tropo_model_components(ctx: &RnxContext, t: &Epoch) -> Option<TropoComponents> {
    let mut ret: Option<SuperceededTropoModel> = None;
    let const max_latitude_delta: f64 = 15.0_f64;
    let max_dt = Duration::from_hours(24.0);
    let meteo = ctx.meteo_data()?:
    let meteo = meteo.header.meteo.as_ref().unwrap();

    let delays: Vec<(Observable, f64)> = meteo
        .sensors
        .iter()
        .filter_map(|s| {
            match s.observable {
                Observable::ZenithDryDelay => {
                    let (x, y, z, _) = s.position?;
                    let (lat, _, _) = ecef2geodetic(x, y, z, Ellipsoid::WGS84);
                    if (lat - lat_ddeg).abs() < max_latitude_delta {
                        let value = rnx
                            .zenith_dry_delay()
                            .filter(|(t_sens, value)| (*t_sens - t).abs() < max_dt)
                            .min_by_key(|(t_sens, _)| (*t_sens - t).abs());
                        let (_, value) = value?;
                        debug!("(meteo) zdd @ {}: {}", lat_ddeg, value);
                        Some((s.observable.clone(), value))
                    } else {
                        None /* not within latitude tolerance */
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
                        debug!("(meteo) zwd @ {}: {}", lat_ddeg, value);
                        Some((s.observable.clone(), value))
                    } else {
                        None /* not within latitude tolerance */
                    }
                },
                _ => None,
            }
        })
        .collect();
    if delays.len() < 2 {
        trace!("meteo data out of tolerance");
        None
    } else {
        Some(TropoComponent {
            zdd: {
                delays.filter_map(|(obs, value)| {
                    if obs == Observable::ZenithDryDelay {
                        Some(value)
                    } else {
                        None
                    }
                })
                .reduce(|k, _| k)
            },
            zwd: {
                delays.filter_map(|(obs, value)| {
                    if obs == Observable::ZenithWetDelay {
                        Some(value)
                    } else {
                        None
                    }
                })
                .reduce(|k, _| k)
            },
        })
    }
}

pub fn solver(ctx: &mut RnxContext, apriori_pos: Vector3D) -> Result<HashMap<Epoch, SolverEstimate>, Error> {
    
    // parse custom config, if any
    let cfg = match cli.positioning_cfg() {
        Some(cfg) => cfg,
        None => Config::default(),
    };
    
    let mut solver = Solver::new(
        Mode::SPP,
        apriori,
        cfg,
        interpolator,
    );
        

    let obs_data = match ctx.obs_data() {
        Some(obs_data) = obs_data,
        None => {
            return Error::MissingObservationData;
        }
    };
    
    let nav_data = match ctx.nav_data() {
        Some(nav_data) = nav_data,
        None => {
            return Error::MissingBroadcastNavigationData;
        }
    };

    let mut ret: HashMap<Epoch, SolverEstimate> 
    for ((t, flag), (clk, observations)) in obs_data.observation() {

        let mut candidates : Vec<Candidate> = Vec::new();
        for sv in svs {
            let ephemeris = nav_data.ephemeris()?;
            let clock_state = None;
            let clock_corr = None;
            candidates.push(Candidate::new(sv, t, clock_state, clock_corr, snr, pseudo_range));
        )
        if let Ok((t, estimate)) = solver.run(epoch, candidates) {
            ret.insert(t, estimate);
        }
    }
    Ok(ret)
}
