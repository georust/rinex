use rinex::prelude::Rinex;
use std::path::Path;
use crate::Error;
use csv::Writer;

pub fn write_obs_rinex<P: AsRef<Path>>(rnx: &Rinex, path: P) -> Result<(), Error> {
    let mut w = Writer::from_path(path)?;
    w.write_record(&[
        "Epoch ", "Flag ", "Clock Offset [s] ", "SV ", "RINEX Code ", "Value ", "LLI ", "SNR ",
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
                w.write_record(
                    &[
                        &t,
                        &flag,
                        &clk,
                        &sv,
                        &code,
                        &value,
                        &lli,
                        &snr,
                    ]
                )?;
            }
        }
    }
    Ok(())
}
