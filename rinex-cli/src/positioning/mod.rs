use crate::cli::{Cli, Context};
use clap::ArgMatches;
use std::cell::RefCell;
use std::fs::read_to_string;

mod buffer;
pub use buffer::Buffer;

mod eph;
use eph::EphemerisSource;

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
pub use rtk::RemoteRTKReference;

mod orbit;
use orbit::Orbits;

mod clock;
use clock::Clock;
pub use clock::ClockStateProvider;

use rinex::{
    carrier::Carrier,
    prelude::{nav::Orbit, Constellation, Rinex},
};

use rinex_qc::prelude::QcExtraPage;

use gnss_rtk::prelude::{
    BdModel, Carrier as RTKCarrier, Config, Duration, Epoch, Error as RTKError, KbModel, Method,
    NgModel, PVTSolutionType, Solver,
};

pub fn precise_positioning(
    _cli: &Cli,
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

            // CGGTTS special case
            if matches.get_flag("cggtts") {
                if cfg!(feature = "cggtts") {
                    cfg.sol_type = PVTSolutionType::TimeOnly;
                } else {
                    panic!("--cggtts solver not compiled!");
                }
            }

            info!("Using custom solver configuration: {:#?}", cfg);
            cfg
        },
        None => {
            let method = Method::default();
            let mut cfg = Config::static_ppp_preset(method);

            // CGGTTS special case
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

    // Verify requirements and print helpful comments
    assert!(
        ctx.data.observation_data().is_some(),
        "Positioning requires Observation RINEX"
    );

    if !is_rtk {
        assert!(
            ctx.data.brdc_navigation_data().is_some(),
            "Positioning requires Navigation RINEX"
        );
    }

    if let Some(obs_rinex) = ctx.data.observation_data() {
        if let Some(obs_header) = &obs_rinex.header.obs {
            if let Some(time_of_first_obs) = obs_header.timeof_first_obs {
                if let Some(clk_rinex) = ctx.data.clock_data() {
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
                } else if let Some(sp3) = ctx.data.sp3_data() {
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

    // create data providers
    let eph = RefCell::new(EphemerisSource::from_ctx(ctx));
    let clocks = Clock::new(&ctx, &eph);
    let orbits = Orbits::new(&ctx, &eph);
    let mut rtk_reference = RemoteRTKReference::from_ctx(&ctx);

    // The CGGTTS opmode (TimeOnly) is not designed
    // to support lack of apriori knowledge
    #[cfg(feature = "cggtts")]
    let apriori = if matches.get_flag("cggtts") {
        if let Some((x, y, z)) = ctx.rx_ecef {
            Some(Orbit::from_position(
                x / 1.0E3,
                y / 1.0E3,
                z / 1.0E3,
                Epoch::default(),
                ctx.data.earth_cef,
            ))
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

    let solver = Solver::new_almanac_frame(
        &cfg,
        apriori,
        orbits,
        ctx.data.almanac.clone(),
        ctx.data.earth_cef,
    );

    #[cfg(feature = "cggtts")]
    if matches.get_flag("cggtts") {
        //* CGGTTS special opmode */
        let tracks = cggtts::resolve(ctx, &eph, clocks, solver, matches)?;
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
    let solutions = ppp::resolve(ctx, &eph, clocks, &mut rtk_reference, solver);
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
