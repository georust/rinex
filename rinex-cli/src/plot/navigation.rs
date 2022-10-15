//! Navigation record plotting
use rinex::*;
use rinex::navigation::*;
use super::Context;
use std::str::FromStr;
use plotters::prelude::*;
use std::collections::HashMap;

pub fn plot(ctx: &mut Context, record: &Record) {
    //TODO
    let mut bias: Vec<f64> = Vec::new();
    let mut drift: Vec<f64> = Vec::new();
    let mut drift_r: Vec<f64> = Vec::new();
    for (epoch, classes) in record {
        for (class, frames) in classes {
            if class == FrameClass::Ephemeris {
                for frame in frames {
                    let fr = frame.as_eph();
                    bias.push(fr.clock_bias);
                    drift.push(fr.clock_drift);
                    drift_rate.push(fr.clock_drift_rate);
                }
            }
        }
    }

    ctx.charts
        .get("CK")
        .unwrap()
        .clone()
        .restore(ctx.plots.get("clock.png").unwrap())
        .draw_series(LineSeries::new(
            ctx.t_axis.iter()
                .enumerate()
                .filter_map(|(index, x)| {
                    if let Some(y) = bias.get(index) {
                        Some((*x, *y))
                    } else {
                        None
                    }
                }),
            &BLACK,
        ))
        .expect("failed to draw observations")
        .label("Clock bias")
        .legend(|(x, y)| {
            //let color = ctx.colors.get(&vehicule.to_string()).unwrap();
            PathElement::new(vec![(x, y), (x + 20, y)], BLACK)
        });
    }
} 
