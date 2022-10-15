//! Observation record plotting method
use rinex::*;
use rinex::observation::Record;
use super::Context;
use std::str::FromStr;
use plotters::prelude::*;
use std::collections::HashMap;

pub fn plot(ctx: &mut Context, record: &Record) {
    //TODO
    // emphasize LLI/SSI somehow ?
    // Grab datapoints to draw for each vehicule, for each chart
    let mut clock_offsets: Vec<f64> = Vec::new();
    let mut pr: HashMap<Sv, Vec<f64>> = HashMap::new();
    let mut ssi: HashMap<Sv, Vec<f64>> = HashMap::new();
    let mut phase: HashMap<Sv, Vec<f64>> = HashMap::new();
    let mut doppler: HashMap<Sv, Vec<f64>> = HashMap::new();
    for (epoch, (clock_offset, vehicules)) in record {
        if let Some(offset) = clock_offset {
            clock_offsets.push(*offset);
        }
        for (vehicule, observations) in vehicules {
            for (observation, data) in observations {
                if is_phase_carrier_obs_code!(observation) {
                    if let Some(phases) = phase.get_mut(vehicule) {
                        phases.push(data.obs);
                    } else {
                        phase.insert(*vehicule, vec![data.obs]);
                    }
                } else if is_doppler_obs_code!(observation) {
                    if let Some(d) = doppler.get_mut(vehicule) {
                        d.push(data.obs);
                    } else {
                        doppler.insert(*vehicule, vec![data.obs]);
                    }
                } else if is_pseudo_range_obs_code!(observation) {
                    if let Some(pr) = pr.get_mut(vehicule) {
                        pr.push(data.obs);
                    } else {
                        pr.insert(*vehicule, vec![data.obs]);
                    }
                } else if is_sig_strength_obs_code!(observation) {
                    if let Some(ssi) = phase.get_mut(vehicule) {
                        ssi.push(data.obs);
                    } else {
                        ssi.insert(*vehicule, vec![data.obs]);
                    }
                }
            }
        }
    }

    if clock_offsets.len() > 0 { // got clock offsets
        ctx.charts
            .get("CK")
            .unwrap()
            .clone()
            .restore(ctx.plots.get("clock-offset.png").unwrap())
            .draw_series(LineSeries::new(
                ctx.t_axis.iter()
                    .enumerate()
                    .filter_map(|(index, x)| {
                        if let Some(y) = clock_offsets.get(index) {
                            Some((*x, *y))
                        } else {
                            None
                        }
                    }),
                &BLACK,
            ))
            .expect("failed to draw observations")
            .label("Offset")
            .legend(|(x, y)| {
                //let color = ctx.colors.get(&vehicule.to_string()).unwrap();
                PathElement::new(vec![(x, y), (x + 20, y)], BLACK)
            });
    }
    for (sv, data) in phase {
        let color = ctx.colors.get(&sv.to_string())
            .unwrap();
        println!("Plotting phase data [phase.png]");
        ctx.charts
            .get("PH")
            .unwrap()
            .clone()
            .restore(ctx.plots.get("phase.png").unwrap())
            .draw_series(LineSeries::new(
                ctx.t_axis.iter()
                    .enumerate()
                    .filter_map(|(index, x)| {
                        if let Some(y) = data.get(index) { // handle missing epochs
                            Some((*x, *y))
                        } else {
                            None
                        }
                    }),
                color.stroke_width(3)
            ))
            .expect("failed to draw observations")
            .label(sv.to_string())
            .legend(|(x, y)| {
                //let color = ctx.colors.get(&vehicule.to_string()).unwrap();
                PathElement::new(vec![(x, y), (x + 20, y)], BLACK)
            });
    }
    for (sv, data) in ssi {
        println!("Plotting ssi [ssi.png]");
        let color = ctx.colors.get(&sv.to_string())
            .unwrap();
        ctx.charts
            .get("SSI")
            .unwrap()
            .clone()
            .restore(ctx.plots.get("ssi.png").unwrap())
            .draw_series(LineSeries::new(
                ctx.t_axis.iter()
                    .enumerate()
                    .filter_map(|(index, x)| {
                        if let Some(y) = data.get(index) {
                            Some((*x, *y))
                        } else {
                            None
                        }
                    }),
                color.stroke_width(3)
            ))
            .expect("failed to draw observations")
            .label(sv.to_string())
            .legend(|(x, y)| {
                //let color = ctx.colors.get(&vehicule.to_string()).unwrap();
                PathElement::new(vec![(x, y), (x + 20, y)], BLACK)
            });
    }
} 
