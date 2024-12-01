use crate::Error;
use csv::Writer;
use rinex::prelude::Rinex;
use std::path::Path;

/// Dumps Observation [Rinex] to CSV
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

    for (key, observations) in rnx.observations_iter() {
        let t = key.epoch.to_string();
        let flag = key.flag.to_string();

        let clk = if let Some(clk) = observations.clock {
            format!("{:.6E}", clk.offset_s)
        } else {
            "None".to_string()
        };

        for signal in observations.signals.iter() {
            let sv = signal.sv.to_string();
            let code = signal.observable.to_string();
            let value = format!("{:.6E}", signal.value);

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

/// Dumps Navigation [Rinex] to CSV
pub fn write_nav_rinex(obs: &Rinex, brdc: &Rinex, path: &Path) -> Result<(), Error> {
    let mut orbit_w = Writer::from_path(path)?;
    orbit_w.write_record(&[
        "Epoch",
        "SV",
        "sv_x(ecef[km])",
        "sv_y(ecef[km])",
        "sv_z(ecef[km])",
    ])?;

    let parent = path.parent().unwrap();
    let stem = path.file_stem().unwrap().to_string_lossy().to_string();

    let clk_path = parent.join(&format!("{}-clock.csv", stem));
    let mut clk_w = Writer::from_path(clk_path)?;
    clk_w.write_record(&["Epoch", "SV", "correction [s]"])?;

    for (k, signal) in obs.signal_observations_iter() {
        if let Some((toc, _toe, eph)) = brdc.sv_ephemeris(signal.sv, k.epoch) {
            if let Some(sv_orbit) = eph.kepler2position(signal.sv, toc, k.epoch) {
                let sv_state = sv_orbit.to_cartesian_pos_vel();
                let (x_km, y_km, z_km) = (sv_state[0], sv_state[1], sv_state[2]);

                orbit_w.write_record(&[
                    k.epoch.to_string(),
                    signal.sv.to_string(),
                    format!("{:.3E}", x_km),
                    format!("{:.3E}", y_km),
                    format!("{:.3E}", z_km),
                ])?;

                if let Some(correction) = eph.clock_correction(toc, k.epoch, signal.sv, 8) {
                    clk_w.write_record(&[
                        k.epoch.to_string(),
                        signal.sv.to_string(),
                        format!("{:.3E}", correction.to_seconds()),
                    ])?;
                }
            }
        }
    }
    Ok(())
}
