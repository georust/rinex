use crate::Error;
use csv::Writer;
use rinex::prelude::Rinex;
use std::path::Path;

pub fn write_obs_rinex<P: AsRef<Path>>(rnx: &Rinex, path: P) -> Result<(), Error> {
    let mut w = Writer::from_path(path)?;
    w.write_record(&[
        "Epoch ",
        "Flag ",
        "Clock Offset [s] ",
        "SV ",
        "RINEX Code ",
        "Value ",
        "LLI ",
        "SNR ",
    ])?;
    for ((epoch, flag), (clk, svnn)) in rnx.observation() {
        let t = epoch.to_string();
        let flag = flag.to_string();
        let clk = if let Some(clk) = clk {
            clk.to_string()
        } else {
            "None".to_string()
        };
        for (sv, obsnn) in svnn.iter() {
            let sv = sv.to_string();
            for (code, obs) in obsnn.iter() {
                let code = code.to_string();
                let value = format!("{:.3E}", obs.obs);
                let lli = if let Some(lli) = obs.lli {
                    format!("{:?}", lli)
                } else {
                    "None".to_string()
                };
                let snr = if let Some(snr) = obs.snr {
                    format!("{:?}", snr)
                } else {
                    "None".to_string()
                };
                w.write_record(&[&t, &flag, &clk, &sv, &code, &value, &lli, &snr])?;
            }
        }
    }
    Ok(())
}

pub fn write_nav_rinex<P: AsRef<Path>>(obs: &Rinex, brdc: &Rinex, path: P) -> Result<(), Error> {
    let mut w = Writer::from_path(path)?;
    w.write_record(&["Epoch", "SV", "x_ecef_km", "y_ecef_km", "z_ecef_km"])?;
    for ((t, _), (_, svnn)) in obs.observation() {
        let t_str = t.to_string();
        for (sv, _) in svnn.iter() {
            let sv_str = sv.to_string();
            if let Some((x_ecef_km, y_ecef_km, z_ecef_km)) = brdc.sv_position(*sv, *t) {
                w.write_record(&[
                    &t_str,
                    &sv_str,
                    &format!("{:.3E} ", x_ecef_km),
                    &format!("{:.3E} ", y_ecef_km),
                    &format!("{:.3E} ", z_ecef_km),
                ])?;
            }
        }
    }
    Ok(())
}
