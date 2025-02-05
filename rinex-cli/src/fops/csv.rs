use crate::Error;
use csv::Writer;
use rinex::prelude::Rinex;
use std::path::Path;

use itertools::Itertools;

pub fn write_obs_rinex<P: AsRef<Path>>(rnx: &Rinex, path: P) -> Result<(), Error> {
    let mut w = Writer::from_path(path)?;
    w.write_record(&[
        "Epoch",
        "Flag",
        "Clock Offset [s]",
        "SV",
        "RINEX Code",
        "Value",
        "LLI",
        "SNR",
    ])?;

    for (k, v) in rnx.observations_iter() {
        let t = k.epoch.to_string();
        let flag = k.flag.to_string();

        let clk = if let Some(clk) = v.clock {
            format!("{:.12}E", clk.offset_s)
        } else {
            "None".to_string()
        };

        for signal in v.signals.iter() {
            let sv = signal.sv.to_string();
            let code = signal.observable.to_string();
            let value = format!("{:.12E}", signal.value);

            let lli = if let Some(lli) = signal.lli {
                format!("{:?}", lli)
            } else {
                "None".to_string()
            };

            let snr = if let Some(snr) = signal.snr {
                format!("{:?}", snr)
            } else {
                "None".to_string()
            };

            w.write_record(&[&t, &flag, &clk, &sv, &code, &value, &lli, &snr])?;
        }
    }
    Ok(())
}

pub fn write_nav_rinex(obs: &Rinex, brdc: &Rinex, path: &Path) -> Result<(), Error> {
    let mut orbit_w = Writer::from_path(path)?;
    orbit_w.write_record(&["Epoch", "SV", "x_ecef_km", "y_ecef_km", "z_ecef_km"])?;

    let parent = path.parent().unwrap();
    let stem = path.file_stem().unwrap().to_string_lossy().to_string();

    let clk_path = parent.join(&format!("{}-clock.csv", stem));
    let mut clk_w = Writer::from_path(clk_path)?;
    clk_w.write_record(&["Epoch", "SV", "correction"])?;

    for (k, v) in obs.observations_iter() {
        let t_str = k.epoch.to_string();

        for sv in v.signals.iter().map(|sig| sig.sv).unique() {
            let sv_str = sv.to_string();

            if let Some((toc, _, eph)) = brdc.nav_ephemeris_selection(sv, k.epoch) {
                if let Some(sv_orbit) = eph.kepler2position(sv, toc, k.epoch) {
                    let sv_state = sv_orbit.to_cartesian_pos_vel();
                    let (x_km, y_km, z_km) = (sv_state[0], sv_state[1], sv_state[2]);

                    orbit_w.write_record(&[
                        &t_str,
                        &sv_str,
                        &format!("{:.12E}", x_km),
                        &format!("{:.12E}", y_km),
                        &format!("{:.12E}", z_km),
                    ])?;

                    if let Some(correction) = eph.clock_correction(toc, k.epoch, sv, 8) {
                        clk_w.write_record(&[
                            &t_str,
                            &sv_str,
                            &format!("{:.12E}", correction.to_seconds()),
                        ])?;
                    }
                }
            }
        }
    }
    Ok(())
}
