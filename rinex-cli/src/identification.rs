use clap::ArgMatches;

use rinex::{
    observation::SNR,
    prelude::{Constellation, Epoch, Observable, Rinex},
    preprocessing::*,
};

use rinex_qc::prelude::{DataContext, ProductType};

use std::str::FromStr;

use itertools::Itertools;
use serde::Serialize;
use std::collections::HashMap;

use map_3d::{ecef2geodetic, Ellipsoid};

/*
 * Dataset identification operations
 */
pub fn dataset_identification(ctx: &DataContext, matches: &ArgMatches) {
    /*
     * Browse all possible types of data, and apply relevant ID operation
     */
    if let Some(files) = ctx.files(ProductType::Observation) {
        let files = files
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
            .collect::<Vec<_>>();
        println!("\n%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%");
        println!("%%%%%%%%%%%% Observation Data %%%%%%%%%");
        println!("%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%");
        println!("{:?}", files);
    }
    if let Some(data) = ctx.observation() {
        if matches.get_flag("all") || matches.get_flag("epochs") {
            println!("{:#?}", EpochReport::from_data(data));
        }
        if matches.get_flag("all") || matches.get_flag("gnss") {
            let constel = data
                .constellation()
                .sorted()
                .map(|c| format!("{:X}", c))
                .collect::<Vec<_>>();
            println!("Constellations: {:?}", constel);
        }
        if matches.get_flag("all") || matches.get_flag("sv") {
            let sv = data
                .sv()
                .sorted()
                .map(|sv| format!("{:X}", sv))
                .collect::<Vec<_>>();
            println!("SV: {:?}", sv);
        }
        if matches.get_flag("all") || matches.get_flag("observables") {
            let observables = data
                .observable()
                .sorted()
                .map(|obs| obs.to_string())
                .collect::<Vec<_>>();
            println!("Observables: {:?}", observables);
        }
        if matches.get_flag("all") || matches.get_flag("snr") {
            let report = SNRReport::from_data(data);
            println!("SNR: {:#?}", report);
        }
        if matches.get_flag("all") || matches.get_flag("anomalies") {
            let anomalies = data.epoch_anomalies().collect::<Vec<_>>();
            if anomalies.is_empty() {
                println!("No anomalies reported.");
            } else {
                println!("Anomalies: {:#?}", anomalies);
            }
        }
    }

    if let Some(files) = ctx.files(ProductType::MeteoObservation) {
        let files = files
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
            .collect::<Vec<_>>();
        println!("\n%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%");
        println!("%%%%%%%%%%%% Meteo Data %%%%%%%%%");
        println!("%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%");
        println!("{:?}", files);
    }
    if let Some(data) = ctx.meteo() {
        if matches.get_flag("all") || matches.get_flag("epochs") {
            println!("{:#?}", EpochReport::from_data(data));
        }
        if matches.get_flag("all") || matches.get_flag("observables") {
            let observables = data
                .observable()
                .sorted()
                .map(|obs| obs.to_string())
                .collect::<Vec<_>>();
            println!("Observables: {:?}", observables);
        }
        if let Some(header) = &data.header.meteo {
            for sensor in &header.sensors {
                println!("{} sensor: ", sensor.observable);
                if let Some(model) = &sensor.model {
                    println!("model: \"{}\"", model);
                }
                if let Some(sensor_type) = &sensor.sensor_type {
                    println!("type: \"{}\"", sensor_type);
                }
                if let Some(ecef) = &sensor.position {
                    let (lat, lon, alt) = ecef2geodetic(ecef.0, ecef.1, ecef.2, Ellipsoid::WGS84);
                    if !lat.is_nan() && !lon.is_nan() {
                        println!("coordinates: lat={}°, lon={}°", lat, lon);
                    }
                    if alt.is_nan() {
                        println!("altitude above sea: {}m", ecef.3);
                    } else {
                        println!("altitude above sea: {}m", alt + ecef.3);
                    }
                }
            }
        }
    }

    if let Some(files) = ctx.files(ProductType::BroadcastNavigation) {
        let files = files
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
            .collect::<Vec<_>>();
        println!("\n%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%");
        println!("%%%%%%%%%%%% Navigation Data (BRDC) %%%%%%%%%");
        println!("%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%");
        println!("{:?}", files);
    }
    if let Some(data) = ctx.brdc_navigation() {
        if matches.get_flag("all") || matches.get_flag("nav-msg") {
            let msg = data.nav_msg_type().collect::<Vec<_>>();
            println!("BRDC NAV Messages: {:?}", msg);
        }
        println!("BRDC Ephemerides: ");
        let ephemerides = data.filter(Filter::from_str("EPH").unwrap());
        if matches.get_flag("all") || matches.get_flag("epochs") {
            println!("{:#?}", EpochReport::from_data(data));
        }
        if matches.get_flag("all") || matches.get_flag("gnss") {
            let constel = ephemerides
                .constellation()
                .sorted()
                .map(|c| format!("{:X}", c))
                .collect::<Vec<_>>();
            println!("Constellations: {:?}", constel);
        }
        if matches.get_flag("all") || matches.get_flag("sv") {
            let sv = ephemerides
                .sv()
                .sorted()
                .map(|sv| format!("{:X}", sv))
                .collect::<Vec<_>>();
            println!("SV: {:?}", sv);
        }
    }

    if let Some(files) = ctx.files(ProductType::HighPrecisionOrbit) {
        let files = files
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
            .collect::<Vec<_>>();
        println!("\n%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%");
        println!("%%%%%%%%%%%% Precise Orbits (SP3) %%%%%%%%%");
        println!("%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%%");
        println!("{:?}", files);
    }
    if let Some(data) = ctx.sp3() {
        println!("SP3 orbits: ");
        if matches.get_flag("all") || matches.get_flag("epochs") {
            let report = EpochReport {
                first: match data.first_epoch() {
                    Some(first) => first.to_string(),
                    None => "Undefined".to_string(),
                },
                last: match data.last_epoch() {
                    Some(last) => last.to_string(),
                    None => "Undefined".to_string(),
                },
                sampling: {
                    [(
                        format!("dt={}s", data.epoch_interval.to_seconds()),
                        data.nb_epochs(),
                    )]
                    .into()
                },
                system: {
                    if let Some(system) = data.constellation.timescale() {
                        system.to_string()
                    } else {
                        "Undefined".to_string()
                    }
                },
            };
            println!("{:#?}", report);
        }
        if matches.get_flag("all") || matches.get_flag("sv") {
            let sv = data
                .sv()
                .sorted()
                .map(|sv| format!("{:X}", sv))
                .collect::<Vec<_>>();
            println!("SV: {:?}", sv);
        }
    }
}

#[derive(Clone, Debug, Serialize)]
struct EpochReport {
    pub first: String,
    pub last: String,
    pub system: String,
    pub sampling: HashMap<String, usize>,
}

impl EpochReport {
    fn from_data(data: &Rinex) -> Self {
        let first_epoch = data.first_epoch();
        Self {
            first: {
                if let Some(first) = first_epoch {
                    first.to_string()
                } else {
                    "NONE".to_string()
                }
            },
            last: {
                if let Some(last) = data.last_epoch() {
                    last.to_string()
                } else {
                    "NONE".to_string()
                }
            },
            sampling: {
                data.sampling_histogram()
                    .map(|(dt, pop)| (format!("dt={}s", dt.to_seconds()), pop))
                    .collect()
            },
            system: {
                if data.is_observation_rinex() || data.is_meteo_rinex() {
                    if let Some(first) = first_epoch {
                        first.time_scale.to_string()
                    } else {
                        "Undefined".to_string()
                    }
                } else if data.is_navigation_rinex() {
                    match data.header.constellation {
                        Some(Constellation::Mixed) => "Mixed".to_string(),
                        Some(c) => c.timescale().unwrap().to_string(),
                        None => "Undefined".to_string(),
                    }
                } else {
                    "Undefined".to_string()
                }
            },
        }
    }
}

#[derive(Clone, Debug, Serialize)]
struct SNRReport {
    pub worst: Option<(Epoch, String, Observable, SNR)>,
    pub best: Option<(Epoch, String, Observable, SNR)>,
}

impl SNRReport {
    fn from_data(data: &Rinex) -> Self {
        Self {
            worst: {
                data.snr()
                    .min_by(|(_, _, _, snr_a), (_, _, _, snr_b)| snr_a.cmp(snr_b))
                    .map(|((t, _), sv, obs, snr)| (t, sv.to_string(), obs.clone(), snr))
            },
            best: {
                data.snr()
                    .max_by(|(_, _, _, snr_a), (_, _, _, snr_b)| snr_a.cmp(snr_b))
                    .map(|((t, _), sv, obs, snr)| (t, sv.to_string(), obs.clone(), snr))
            },
        }
    }
}
