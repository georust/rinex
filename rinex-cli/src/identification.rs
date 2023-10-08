use crate::Cli;
use hifitime::Epoch;
use rinex::observation::Snr;
use rinex::*;
use rinex_qc::QcContext;

/*
 * Basic identification operations
 */
pub fn rinex_identification(ctx: &QcContext, cli: &Cli) {
    let pretty = cli.pretty();
    let ops = cli.identification_ops();

    identification(
        &ctx.primary_data(),
        &ctx.primary_path().to_string_lossy().to_string(),
        pretty,
        ops.clone(),
    );

    if let Some(nav) = &ctx.navigation_data() {
        identification(&nav, "Navigation Context blob", pretty, ops.clone());
    }
}

use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
struct EpochReport {
    pub first: String,
    pub last: String,
}

#[derive(Clone, Debug, Serialize)]
struct SSIReport {
    pub min: Option<Snr>,
    pub max: Option<Snr>,
}

fn identification(rnx: &Rinex, path: &str, pretty: bool, ops: Vec<&str>) {
    for op in ops {
        debug!("identification: {}", op);
        if op.eq("header") {
            let content = match pretty {
                true => serde_json::to_string_pretty(&rnx.header).unwrap(),
                false => serde_json::to_string(&rnx.header).unwrap(),
            };
            println!("[{}]: {}", path, content);
        } else if op.eq("epochs") {
            let report = EpochReport {
                first: format!("{:?}", rnx.first_epoch()),
                last: format!("{:?}", rnx.last_epoch()),
            };
            let content = match pretty {
                true => serde_json::to_string_pretty(&report).unwrap(),
                false => serde_json::to_string(&report).unwrap(),
            };
            println!("[{}]: {}", path, content);
        } else if op.eq("sv") {
            let mut csv = String::new();
            for (i, sv) in rnx.sv().enumerate() {
                if i == rnx.sv().count() - 1 {
                    csv.push_str(&format!("{}\n", sv.to_string()));
                } else {
                    csv.push_str(&format!("{}, ", sv.to_string()));
                }
            }
            println!("[{}]: {}", path, csv);
        } else if op.eq("observables") && rnx.is_observation_rinex() {
            let mut data: Vec<_> = rnx.observable().collect();
            data.sort();
            //let content = match pretty {
            //    true => serde_json::to_string_pretty(&data).unwrap(),
            //    false => serde_json::to_string(&data).unwrap(),
            //};
            println!("[{}]: {:?}", path, data);
        } else if op.eq("gnss") {
            let data: Vec<_> = rnx.constellation().collect();
            let content = match pretty {
                true => serde_json::to_string_pretty(&data).unwrap(),
                false => serde_json::to_string(&data).unwrap(),
            };
            println!("[{}]: {}", path, content);
        } else if op.eq("ssi-range") && rnx.is_observation_rinex() {
            let ssi = SSIReport {
                min: {
                    rnx.snr()
                        .min_by(|(_, _, _, snr_a), (_, _, _, snr_b)| snr_a.cmp(snr_b))
                        .map(|(_, _, _, snr)| snr)
                },
                max: {
                    rnx.snr()
                        .max_by(|(_, _, _, snr_a), (_, _, _, snr_b)| snr_a.cmp(snr_b))
                        .map(|(_, _, _, snr)| snr)
                },
            };
            let content = match pretty {
                true => serde_json::to_string_pretty(&ssi).unwrap(),
                false => serde_json::to_string(&ssi).unwrap(),
            };
            println!("[{}]: {}", path, content);
        } else if op.eq("orbits") && rnx.is_navigation_rinex() {
            error!("nav::orbits not available yet");
            //let data: Vec<_> = rnx.orbit_fields();
            //let content = match pretty {
            //    true => serde_json::to_string_pretty(&data).unwrap(),
            //    false => serde_json::to_string(&data).unwrap(),
            //};
            //println!("{}", content);
        } else if op.eq("nav-msg") && rnx.is_navigation_rinex() {
            let data: Vec<_> = rnx.nav_msg_type().collect();
            println!("{:?}", data);
        } else if op.eq("anomalies") && rnx.is_observation_rinex() {
            let data: Vec<_> = rnx.epoch_anomalies().collect();
            println!("{:#?}", data);
        }
    }
}
