use crate::Cli;
use std::str::FromStr;

use rinex::observation::SNR;
use rinex::prelude::RnxContext;
use rinex::preprocessing::*;
use rinex::*;
use sp3::SP3;

use itertools::Itertools;
use serde::Serialize;
use std::collections::HashMap;

use hifitime::Duration;

/*
 * Basic identification operations
 */
pub fn rinex_identification(ctx: &RnxContext, cli: &Cli) {
    let ops = cli.identification_ops();
    let pretty_json = cli.pretty_json();
    /*
     * Run identification on all contained files
     */
    if let Some(data) = ctx.obs_data() {
        info!("obs. data identification");
        identification(
            data,
            &format!(
                "{:?}",
                ctx.obs_paths()
                    .unwrap()
                    .iter()
                    .map(|p| p.to_string_lossy().to_string())
                    .collect::<Vec<String>>()
            ),
            pretty_json,
            ops.clone(),
        );
    }
    if let Some(nav) = &ctx.nav_data() {
        info!("nav. data identification");
        identification(
            nav,
            &format!(
                "{:?}",
                ctx.nav_paths()
                    .unwrap()
                    .iter()
                    .map(|p| p.to_string_lossy().to_string())
                    .collect::<Vec<String>>()
            ),
            pretty_json,
            ops.clone(),
        );
    }
    if let Some(data) = &ctx.meteo_data() {
        info!("meteo identification");
        identification(
            data,
            &format!(
                "{:?}",
                ctx.meteo_paths()
                    .unwrap()
                    .iter()
                    .map(|p| p.to_string_lossy().to_string())
                    .collect::<Vec<String>>()
            ),
            pretty_json,
            ops.clone(),
        );
    }
    if let Some(data) = &ctx.ionex_data() {
        info!("ionex identification");
        identification(
            data,
            &format!(
                "{:?}",
                ctx.ionex_paths()
                    .unwrap()
                    .iter()
                    .map(|p| p.to_string_lossy().to_string())
                    .collect::<Vec<String>>()
            ),
            pretty_json,
            ops.clone(),
        );
    }
    if let Some(sp3) = ctx.sp3_data() {
        sp3_identification(sp3);
    }
}

#[derive(Clone, Debug, Serialize)]
struct EpochReport {
    pub first: String,
    pub last: String,
}

#[derive(Clone, Debug, Serialize)]
struct SSIReport {
    pub min: Option<SNR>,
    pub max: Option<SNR>,
}

fn report_sampling_histogram(data: &Vec<(Duration, usize)>) {
    let data: HashMap<String, usize> = data
        .into_iter()
        .map(|(dt, pop)| (dt.to_string(), *pop))
        .collect();
    println!("{:#?}", data);
}

/*
 * Method dedicated to sampling "identification"
 */
fn sampling_identification(rnx: &Rinex) {
    if rnx.is_navigation_rinex() {
        /*
         * with NAV RINEX, we're interested in
         * differentiating the BRDC NAV/ION and basically all messages time frames
         */
        let data: Vec<(Duration, usize)> = rnx
            .filter(Filter::from_str("EPH").unwrap())
            .sampling_histogram()
            .collect();
        println!("BRDC ephemeris:");
        report_sampling_histogram(&data);
    } else {
        // Other RINEX types: run sampling histogram analysis
        let data: Vec<(Duration, usize)> = rnx.sampling_histogram().collect();
        report_sampling_histogram(&data);
    }
}

fn identification(rnx: &Rinex, path: &str, pretty_json: bool, ops: Vec<&str>) {
    for op in ops {
        debug!("identification: {}", op);
        if op.eq("header") {
            let content = match pretty_json {
                true => serde_json::to_string_pretty(&rnx.header).unwrap(),
                false => serde_json::to_string(&rnx.header).unwrap(),
            };
            println!("[{}]: {}", path, content);
        } else if op.eq("epochs") {
            let report = EpochReport {
                first: format!("{:?}", rnx.first_epoch()),
                last: format!("{:?}", rnx.last_epoch()),
            };
            let content = match pretty_json {
                true => serde_json::to_string_pretty(&report).unwrap(),
                false => serde_json::to_string(&report).unwrap(),
            };
            println!("[{}]: {}", path, content);
        } else if op.eq("sv") && (rnx.is_observation_rinex() || rnx.is_navigation_rinex()) {
            let mut csv = String::new();
            for (i, sv) in rnx.sv().sorted().enumerate() {
                if i == rnx.sv().count() - 1 {
                    csv.push_str(&format!("{}\n", sv));
                } else {
                    csv.push_str(&format!("{}, ", sv));
                }
            }
            println!("[{}]: {}", path, csv);
        } else if op.eq("observables") && rnx.is_observation_rinex() {
            let mut data: Vec<_> = rnx.observable().collect();
            data.sort();
            let content = match pretty_json {
                true => serde_json::to_string_pretty(&data).unwrap(),
                false => serde_json::to_string(&data).unwrap(),
            };
            println!("[{}]: {}", path, content);
        } else if op.eq("gnss") && (rnx.is_observation_rinex() || rnx.is_navigation_rinex()) {
            let mut data: Vec<_> = rnx.constellation().collect();
            data.sort();
            let content = match pretty_json {
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
            let content = match pretty_json {
                true => serde_json::to_string_pretty(&ssi).unwrap(),
                false => serde_json::to_string(&ssi).unwrap(),
            };
            println!("[{}]: {}", path, content);
        } else if op.eq("orbits") && rnx.is_navigation_rinex() {
            error!("nav::orbits not available yet");
            //let data: Vec<_> = rnx.orbit_fields();
            //let content = match pretty_json {
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
        } else if op.eq("sampling") {
            sampling_identification(rnx);
        }
    }
}

fn sp3_identification(sp3: &SP3) {
    let report = format!(
        "SP3 IDENTIFICATION
Sampling period: {:?},
NB of epochs: {},
Time frame: {:?} - {:?},
SV: {:?}
",
        sp3.epoch_interval,
        sp3.nb_epochs(),
        sp3.first_epoch(),
        sp3.last_epoch(),
        sp3.sv().map(|sv| sv.to_string()).collect::<Vec<String>>()
    );
    println!("{}", report);
}
