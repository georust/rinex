//! Observation record plotting
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
    let mut e0: i64 = 0;
    let mut clock_offsets: Vec<(f64, f64)> = Vec::new();
    let mut pr: HashMap<Sv, Vec<(f64, f64)>> = HashMap::new();
    let mut ssi: HashMap<Sv, Vec<(f64,f64)>> = HashMap::new();
    let mut phase: HashMap<Sv, Vec<(f64,f64)>> = HashMap::new();
    let mut doppler: HashMap<Sv, Vec<(f64,f64)>> = HashMap::new();
    for (index, (epoch, (clock_offset, vehicules))) in record.iter().enumerate() {
        if index == 0 {
            e0 = epoch.date.timestamp()
        }
        let e = epoch.date.timestamp();
        if let Some(value) = clock_offset {
            clock_offsets.push(((e-e0) as f64, *value));
        }
        for (vehicule, observations) in vehicules {
            for (observation, data) in observations {
                if is_phase_carrier_obs_code!(observation) {
                    if let Some(phases) = phase.get_mut(vehicule) {
                        phases.push(((e-e0) as f64, data.obs));
                    } else {
                        phase.insert(*vehicule, vec![((e-e0) as f64, data.obs)]);
                    }
                } else if is_doppler_obs_code!(observation) {
                    if let Some(d) = doppler.get_mut(vehicule) {
                        d.push(((e-e0) as f64, data.obs));
                    } else {
                        doppler.insert(*vehicule, vec![((e-e0) as f64, data.obs)]);
                    }
                } else if is_pseudo_range_obs_code!(observation) {
                    if let Some(pr) = pr.get_mut(vehicule) {
                        pr.push(((e-e0) as f64, data.obs));
                    } else {
                        pr.insert(*vehicule, vec![((e-e0) as f64, data.obs)]);
                    }
                } else if is_sig_strength_obs_code!(observation) {
                    if let Some(ssi) = phase.get_mut(vehicule) {
                        ssi.push(((e-e0) as f64, data.obs));
                    } else {
                        ssi.insert(*vehicule, vec![((e-e0) as f64, data.obs)]);
                    }
                }
            }
        }
    }

    if clock_offsets.len() > 0 { // got clock offsets
        let plot = ctx.plots.get("clock-offset.png")
            .expect("faulty plot context, missing clock offset plot");
        ctx.charts
            .get("CK")
            .expect("faulty plot context, missing clock offset chart")
            .clone()
            .restore(plot)
            .draw_series(LineSeries::new(
                clock_offsets.iter()
                    .map(|(x, y)| (*x, *y)),
                &BLACK,
            ))
            .expect("failed to display receiver clock offsets")
            .label("Offset")
            .legend(|(x, y)| {
                //let color = ctx.colors.get(&vehicule.to_string()).unwrap();
                PathElement::new(vec![(x, y), (x + 20, y)], BLACK)
            });
    }
    for (sv, data) in phase {
        let color = ctx.colors.get(&sv.to_string())
            .expect(&format!("no colors to identify \"{}\"", sv));
        let plot = ctx.plots.get("phase.png")
            .expect("missing phase data plot");
        ctx.charts
            .get("PH")
            .expect("missing phase data chart")
            .clone()
            .restore(plot)
            .draw_series(LineSeries::new(
                data.iter()
                    .map(|(x, y)| (*x, *y)),
                color.stroke_width(3)
            ))
            .expect("failed to draw phase observations")
            .label(sv.to_string())
            .legend(|(x, y)| {
                //let color = ctx.colors.get(&vehicule.to_string()).unwrap();
                PathElement::new(vec![(x, y), (x + 20, y)], BLACK)
            });
    }
    for (sv, data) in pr {
        let color = ctx.colors.get(&sv.to_string())
            .expect(&format!("no colors to identify \"{}\"", sv));
        let plot = ctx.plots.get("pseudo-range.png")
            .expect("missing pseudo range data plot");
        ctx.charts
            .get("PR")
            .expect("missing pseudo range data chart")
            .clone()
            .restore(plot)
            .draw_series(LineSeries::new(
                data.iter()
                    .map(|(x, y)| (*x, *y)),
                color.stroke_width(3)
            ))
            .expect("failed to draw pseudo range observations")
            .label(sv.to_string())
            .legend(|(x, y)| {
                //let color = ctx.colors.get(&vehicule.to_string()).unwrap();
                PathElement::new(vec![(x, y), (x + 20, y)], BLACK)
            });
    }
    for (sv, data) in ssi {
        let color = ctx.colors.get(&sv.to_string())
            .expect(&format!("no colors to identify \"{}\"", sv));
        let plot = ctx.plots.get("phase.png")
            .expect("missing ssi data plot");
        ctx.charts
            .get("SSI")
            .expect("missing ssi data chart")
            .clone()
            .restore(plot)
            .draw_series(LineSeries::new(
                data.iter()
                    .map(|(x, y)| (*x, *y)),
                color.stroke_width(3)
            ))
            .expect("failed to draw ssi observations")
            .label(sv.to_string())
            .legend(|(x, y)| {
                //let color = ctx.colors.get(&vehicule.to_string()).unwrap();
                PathElement::new(vec![(x, y), (x + 20, y)], BLACK)
            });
    }
    // draw labels
    if let Some(plot) = ctx.plots.get("phase.png") {
        if let Some(chart) = ctx.charts.get("PH") {
            chart
                .clone()
                .restore(&plot)
                .configure_series_labels()
                .border_style(&BLACK)
                .background_style(WHITE.filled())
                .draw()
                .expect("failed to draw labels on phase chart");
        }
    }
    if let Some(plot) = ctx.plots.get("doppler.png") {
        if let Some(chart) = ctx.charts.get("DOP") {
            chart
                .clone()
                .restore(&plot)
                .configure_series_labels()
                .border_style(&BLACK)
                .background_style(WHITE.filled())
                .draw()
                .expect("failed to draw labels on doppler chart");
        }
    }
    if let Some(plot) = ctx.plots.get("ssi.png") {
        if let Some(chart) = ctx.charts.get("SSI") {
            chart
                .clone()
                .restore(&plot)
                .configure_series_labels()
                .border_style(&BLACK)
                .background_style(WHITE.filled())
                .draw()
                .expect("failed to draw labels on ssi chart");
        }
    }
    if let Some(plot) = ctx.plots.get("pseudo-range.png") {
        if let Some(chart) = ctx.charts.get("PR") {
            chart
                .clone()
                .restore(&plot)
                .configure_series_labels()
                .border_style(&BLACK)
                .background_style(WHITE.filled())
                .draw()
                .expect("failed to draw labels on pseudo range chart");
        }
    }
} 
