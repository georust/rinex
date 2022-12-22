/*
use crate::{Cli, Context};
use rinex::{navigation::MsgType, prelude::*};
use std::str::FromStr;

fn args_to_constellations(args: Vec<&str>) -> Vec<Constellation> {
    args.iter()
        .filter_map(|x| {
            if let Ok(c) = Constellation::from_str(x) {
                Some(c)
            } else {
                println!("non recognized constellation \"{}\"", x);
                None
            }
        })
        .collect()
}

fn args_to_space_vehicules(args: Vec<&str>) -> Vec<Sv> {
    args.iter()
        .filter_map(|x| {
            if let Ok(c) = Sv::from_str(x) {
                Some(c)
            } else {
                println!("non recognized vehicule description \"{}\"", x);
                None
            }
        })
        .collect()
}

fn args_to_nav_message(args: Vec<&str>) -> Vec<MsgType> {
    args.iter()
        .filter_map(|x| {
            if let Ok(msg) = MsgType::from_str(x) {
                Some(msg)
            } else {
                println!("unknown navigation message type \"{}\"", x);
                None
            }
        })
        .collect()
}

/// Efficient RINEX content filter
pub fn retain_filters(ctx: &mut Context, cli: &Cli) {
    let flags = cli.retain_flags();
    let ops = cli.retain_ops();

    for flag in flags {
        if flag.eq("retain-epoch-ok") {
            ctx.primary_rinex.retain_epoch_ok_mut();
        } else if flag.eq("retain-epoch-nok") {
            ctx.primary_rinex.retain_epoch_nok_mut();
        } else if flag.eq("retain-lnav") {
            ctx.primary_rinex.retain_legacy_navigation_mut();
            if let Some(ref mut nav) = ctx.nav_rinex {
                nav.retain_legacy_navigation_mut();
            }
        } else if flag.eq("retain-mnav") {
            ctx.primary_rinex.retain_modern_navigation_mut();
            if let Some(ref mut nav) = ctx.nav_rinex {
                nav.retain_modern_navigation_mut();
            }
        } else if flag.eq("retain-nav-eph") {
            ctx.primary_rinex.retain_navigation_ephemeris_mut();
            if let Some(ref mut nav) = ctx.nav_rinex {
                nav.retain_modern_navigation_mut();
            }
        } else if flag.eq("retain-nav-iono") {
            ctx.primary_rinex.retain_navigation_ionospheric_models_mut();
            if let Some(ref mut nav) = ctx.nav_rinex {
                nav.retain_navigation_ionospheric_models_mut();
            }
        } else if flag.eq("retain-phase") {
            ctx.primary_rinex.retain_phase_observations_mut();
        } else if flag.eq("retain-pr") {
            ctx.primary_rinex.retain_pseudorange_observations_mut();
        } else if flag.eq("retain-doppler") {
            ctx.primary_rinex.retain_doppler_observations_mut();
        }
    }
    for (op, args) in ops.iter() {
        if op.eq(&"retain-sv") {
            let filter = args_to_space_vehicules(args.clone());
            ctx.primary_rinex.retain_space_vehicule_mut(filter.clone());
            if let Some(ref mut nav) = ctx.nav_rinex {
                nav.retain_space_vehicule_mut(filter.clone());
            }
        } else if op.eq(&"retain-constell") {
            let filter = args_to_constellations(args.clone());
            ctx.primary_rinex.retain_constellation_mut(filter.clone());
            if let Some(ref mut nav) = ctx.nav_rinex {
                nav.retain_constellation_mut(filter.clone());
            }
        } else if op.eq(&"retain-obs") {
            ctx.primary_rinex.retain_observable_mut(args.clone());
        } else if op.eq(&"retain-ssi") {
        } else if op.eq(&"retain-orb") {
        } else if op.eq(&"retain-nav-msg") {
            let filter = args_to_nav_message(args.clone());
            ctx.primary_rinex.retain_navigation_message_mut(&filter);
            if let Some(ref mut nav) = ctx.nav_rinex {
                nav.retain_navigation_message_mut(&filter);
            }
        }
    }
    */
}
