//! Navigation record plotting
use super::Context;
use rinex::{*,
    navigation::*,
};
use std::str::FromStr;
use plotters::prelude::*;
use std::collections::HashMap;

pub fn plot(ctx: &mut Context, record: &Record) {
    let mut e0: i64 = 0;
    let mut bias: HashMap<Sv, Vec<(f64,f64)>> = HashMap::new();
    let mut drift: HashMap<Sv, Vec<(f64,f64)>> = HashMap::new();
    let mut driftr: HashMap<Sv, Vec<(f64,f64)>> = HashMap::new();
    for (index, (epoch, classes)) in record.iter().enumerate() {
        if index == 0 {
            e0 = epoch.date.timestamp();
        }
        let t = epoch.date.timestamp() - e0;
        for (class, frames) in classes {
            if *class == FrameClass::Ephemeris {
                for frame in frames {
                    if let Some((_, sv, eph)) = frame.as_eph() {
                        if let Some(bias) = bias.get_mut(&sv) {
                            bias.push((t as f64, eph.clock_bias));
                        } else {
                            bias.insert(*sv, vec![(t as f64, eph.clock_bias)]);
                        }
                        if let Some(drift) = drift.get_mut(&sv) {
                            drift.push((t as f64, eph.clock_drift));
                        } else {
                            drift.insert(*sv, vec![(t as f64, eph.clock_drift)]);
                        }
                        if let Some(driftr) = driftr.get_mut(&sv) {
                            driftr.push((t as f64, eph.clock_drift_rate));
                        } else {
                            driftr.insert(*sv, vec![(t as f64, eph.clock_drift_rate)]);
                        }
                    }
                }
            }
        }
    }
    let plot = ctx.plots.get("clock-bias.png")
        .unwrap();
    let mut chart = ctx.charts.get("clock-bias.png")
        .unwrap()
        .clone()
        .restore(&plot);
    for (vehicule, bias) in bias {
        chart
            .draw_series(LineSeries::new(
                bias.iter().map(|point| *point),
                &BLACK,
            ))
            .expect("failed to draw clock biases")
            .label("Clock bias")
            .legend(|(x, y)| {
                //let color = ctx.colors.get(&vehicule.to_string()).unwrap();
                PathElement::new(vec![(x, y), (x + 20, y)], BLACK)
            });
    }
} 
