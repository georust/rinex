use rinex::{
    prelude::*,
    navigation::MsgType,
};
use std::str::FromStr;

fn args_to_constellations (args: Vec<&str>) -> Vec<Constellation> {
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

fn args_to_space_vehicules (args: Vec<&str>) -> Vec<Sv> {
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

fn args_to_nav_message (args: Vec<&str>) -> Vec<MsgType> {
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
pub fn retain_filters(rnx: &mut Rinex, flags: Vec<&str>, ops: Vec<(&str, Vec<&str>)>) {
    for flag in flags {
        if flag.eq("retain-epoch-ok") {
            rnx.retain_epoch_ok_mut();
        } else if flag.eq("retain-epoch-nok") {
            rnx.retain_epoch_nok_mut();
        } else if flag.eq("retain-pr") {
        } else if flag.eq("retain-lnav") {
            rnx.retain_legacy_navigation_mut();
        } else if flag.eq("retain-mnav") {
            rnx.retain_modern_navigation_mut();
        } else if flag.eq("retain-nav-eph") {
            rnx.retain_navigation_ephemeris_mut();
        } else if flag.eq("retain-nav-iono") {
            rnx.retain_navigation_ionospheric_models_mut();
        } else if flag.eq("retain-phase") {
            rnx.retain_phase_observations_mut();
        } else if flag.eq("retain-pr") {
            rnx.retain_pseudorange_observations_mut();
        } else if flag.eq("retain-doppler") {
            rnx.retain_doppler_observations_mut();
        } else if flag.eq("retain-best-elev") {
            rnx.retain_best_elevation_angles_mut();
        }
    }
    for (op, args) in ops.iter() {
        if op.eq(&"retain-sv") {
            let filter = args_to_space_vehicules(args.clone());
            rnx.retain_space_vehicule_mut(filter);
        } else if op.eq(&"retain-constell") {
            let filter = args_to_constellations(args.clone());
            rnx.retain_constellation_mut(filter);
        } else if op.eq(&"retain-obs") {
            rnx.retain_observable_mut(args.clone());
        } else if op.eq(&"retain-ssi") {
        } else if op.eq(&"retain-orb") {
        } else if op.eq(&"retain-nav-msg") {
            let filter = args_to_nav_message(args.clone());
            rnx.retain_navigation_message_mut(filter);
        } else if op.eq(&"retain-elev-above") {
            if let Ok(a0) = f64::from_str(args[0].trim()) {
                rnx.orbits_elevation_angle_filter_mut(a0);
            } else {
                println!("failed to parse elevation angle. Expecting floating point value");
            }

        } else if op.eq(&"retain-elev-below") {
            if let Ok(a1) = f64::from_str(args[0].trim()) {
                rnx.orbits_elevation_angle_range_filter_mut((0.0,a1)); 
            } else {
                println!("failed to parse elevation angle. Expecting floating point value");
            }

        } else if op.eq(&"retain-elev") {
            if let Ok(a0) = f64::from_str(args[0].trim()) {
                if let Ok(a1) = f64::from_str(args[0].trim()) {
                    rnx.orbits_elevation_angle_range_filter_mut((a0,a1)); 
                } else {
                    println!("failed to parse elevation angle. Expecting floating point value");
                }
            } else {
                println!("failed to parse elevation angle. Expecting floating point value");
            }
        }
    }
}
