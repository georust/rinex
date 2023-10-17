use crate::RTKConfig;
use hifitime::{Duration, Epoch};
use log::{debug, trace};
use map_3d::{deg2rad, ecef2geodetic, Ellipsoid};
use rinex::prelude::{Observable, RnxContext, SV};
use std::collections::HashMap;

use std::f64::consts::PI;

fn meteorological_tropo_delay(
    t: Epoch,
    lat_ddeg: f64,
    ctx: &RnxContext,
) -> (Option<f64>, Option<f64>) {
    // maximal tolerated latitude difference
    const max_latitude_delta: f64 = 15.0_f64;
    // retain data sampled on that day only
    // const max_dt: Duration = Duration {
    //     centuries: 0,
    //     nanoseconds: 3_600_000_000_000,
    // };
    let max_dt = Duration::from_hours(24.0);

    let rnx = ctx.meteo_data().unwrap(); // infaillible
    let meteo = rnx.header.meteo.as_ref().unwrap(); // infaillible
    let delays: Vec<(Observable, f64)> = meteo
        .sensors
        .iter()
        .filter_map(|s| {
            match s.observable {
                Observable::ZenithDryDelay => {
                    let (x, y, z, _) = s.position?;
                    let (lat, _, _) = ecef2geodetic(x, y, z, Ellipsoid::WGS84);
                    if (lat - lat_ddeg).abs() < max_latitude_delta {
                        let value = rnx
                            .zenith_dry_delay()
                            .filter(|(t_sens, value)| (*t_sens - t).abs() < max_dt)
                            .min_by_key(|(t_sens, _)| (*t_sens - t).abs());
                        let (_, value) = value?;
                        debug!("(meteo) zdd @ {}: {}", lat_ddeg, value);
                        Some((s.observable.clone(), value))
                    } else {
                        None /* not within latitude tolerance */
                    }
                },
                Observable::ZenithWetDelay => {
                    let (x, y, z, _) = s.position?;
                    let (lat, _, _) = ecef2geodetic(x, y, z, Ellipsoid::WGS84);
                    if (lat - lat_ddeg).abs() < max_latitude_delta {
                        let value = rnx
                            .zenith_wet_delay()
                            .filter(|(t_sens, value)| (*t_sens - t).abs() < max_dt)
                            .min_by_key(|(t_sens, _)| (*t_sens - t).abs());
                        let (_, value) = value?;
                        debug!("(meteo) zwd @ {}: {}", lat_ddeg, value);
                        Some((s.observable.clone(), value))
                    } else {
                        None /* not within latitude tolerance */
                    }
                },
                _ => None,
            }
        })
        .collect();
    if delays.len() > 1 {
        /* both components were identified, return their values */
        (Some(delays[0].1), Some(delays[1].1))
    } else {
        trace!("meteo data out of tolerance");
        (None, None)
    }
}

#[derive(Copy, Clone, Debug)]
enum UNB3Param {
    // pressure in mBar
    Pressure = 0,
    // temperature in Kelvin
    Temperature = 1,
    // water vapour pressure in mBar
    WaterVapourPressure = 2,
    // beta is temperature lapse rate (Kelvin/m)
    Beta = 3,
    // lambda is wvp height factor (N/A)
    Lambda = 4,
}

fn unb3_look_up_table(lut: [(f64, [f64; 5]); 5], prm: UNB3Param, lat_ddeg: f64) -> f64 {
    let prm = (prm as u8) as usize;
    if lat_ddeg <= 15.0 {
        lut[0].1[prm]
    } else if lat_ddeg >= 75.0 {
        lut[4].1[prm]
    } else {
        let mut nearest: usize = 0;
        let mut min_delta = 180.0_f64;
        for i in 0..5 {
            let lat = lut[i].0;
            let delta = (lat - lat_ddeg).abs();
            if delta < min_delta {
                min_delta = delta;
                nearest = i;
            }
        }
        lut[nearest].1[prm] + lut[nearest + 1].1[prm]
            - lut[nearest].1[prm] / 15.0_f64 * (lat_ddeg - lut[nearest].0)
    }
}

fn unb3_annual_average(prm: UNB3Param, lat_ddeg: f64) -> f64 {
    const lut: [(f64, [f64; 5]); 5] = [
        (15.0, [1013.25, 299.65, 26.31, 6.30E-3, 2.77]),
        (30.0, [1017.25, 294.15, 21.79, 6.05E-3, 3.15]),
        (45.0, [1015.75, 283.15, 11.66, 5.58E-3, 2.57]),
        (60.0, [1011.75, 272.15, 6.78, 5.39E-3, 1.81]),
        (75.0, [1013.00, 263.65, 4.11, 4.53E-3, 1.55]),
    ];
    unb3_look_up_table(lut, prm, lat_ddeg)
}

fn unb3_average_amplitude(prm: UNB3Param, lat_ddeg: f64) -> f64 {
    const lut: [(f64, [f64; 5]); 5] = [
        (15.0, [0.0, 0.0, 0.0, 0.0, 0.0]),
        (30.0, [-3.75, 7.0, 8.85, 0.25E-3, 0.33]),
        (45.0, [-2.25, 11.0, 7.24, 0.32E-3, 0.46]),
        (60.0, [-1.75, 15.0, 5.36, 0.81E-3, 0.74]),
        (75.0, [-0.50, 14.5, 3.39, 0.62E-3, 0.3]),
    ];
    unb3_look_up_table(lut, prm, lat_ddeg)
}

fn unb3_parameter(prm: UNB3Param, lat_ddeg: f64, day_of_year: f64) -> f64 {
    let dmin = match lat_ddeg.is_sign_positive() {
        true => 28.0_f64,
        false => 211.0_f64,
    };
    let annual = unb3_annual_average(prm, lat_ddeg);
    let amplitude = unb3_average_amplitude(prm, lat_ddeg);
    annual - amplitude * ((day_of_year - dmin) * 2.0_f64 * PI / 365.25_f64).cos()
}

/*
 * Evaluate ZWD and ZDD at given Epoch "t" and given latitude
 * This method is infaillible and will work at any Epoch, for any latitude
 */
fn unb3_delay_components(t: Epoch, lat_ddeg: f64, alt_above_sea_m: f64) -> (f64, f64) {
    // estimate zenith delay
    const r: f64 = 287.054;
    const k_1: f64 = 77.064;
    const k_2: f64 = 382000.0_f64;
    const r_d: f64 = 287.054;

    const g: f64 = 9.80665_f64;
    const g_m: f64 = 9.784_f64;
    //TODO
    //let g_m = 9.784 * (1.0_f64 - 2.66E-3 * (2 * lat_ddeg).cos() - 2.8E-7 * h);

    let day_of_year = t.day_of_year();

    //TODO
    // let h = h is the orthometric height in m;
    let beta = unb3_parameter(UNB3Param::Beta, lat_ddeg, day_of_year);
    let p = unb3_parameter(UNB3Param::Pressure, lat_ddeg, day_of_year);
    let t = unb3_parameter(UNB3Param::Temperature, lat_ddeg, day_of_year);
    let e = unb3_parameter(UNB3Param::WaterVapourPressure, lat_ddeg, day_of_year);
    let lambda = unb3_parameter(UNB3Param::Lambda, lat_ddeg, day_of_year);

    let z0_zdd = 10.0E-6 * k_1 * r_d * p / g_m;
    let denom = (lambda + 1.0_f64) * g_m - beta * r_d;
    let z0_zwd = 10.0E-6 * k_2 * r_d * e / t / denom;

    let value = 1.0_f64 - beta * alt_above_sea_m / t;

    let zdd = (value).powf(g / r_d / beta) * z0_zdd;
    let zwd = (value).powf((lambda + 1.0_f64) * g / r_d / beta - 1.0_f64) * z0_zwd;

    debug!(
        "{:?}: unb3 - zdd(h=0) {} zwd(h=0) {} zdd(h={}) {} zwd(h={}) {}",
        t, z0_zdd, z0_zwd, alt_above_sea_m, zdd, alt_above_sea_m, zwd
    );
    (zdd, zwd)
}

pub(crate) fn tropo_delay(
    t: Epoch,
    lat_ddeg: f64,
    altitude: f64,
    elev: f64,
    ctx: &RnxContext,
) -> f64 {
    let (zdd, zwd): (f64, f64) = match ctx.has_meteo_data() {
        true => {
            if let (Some(zdd), Some(zwd)) = meteorological_tropo_delay(t, lat_ddeg, ctx) {
                (zdd, zwd)
            } else {
                unb3_delay_components(t, lat_ddeg, altitude)
            }
        },
        false => unb3_delay_components(t, lat_ddeg, altitude),
    };
    1.001_f64 / (0.002001_f64 + elev.sin().powi(2)).sqrt() * (zdd + zwd)
}
