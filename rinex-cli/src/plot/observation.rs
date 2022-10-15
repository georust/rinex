//! Observation record plotting method
use rinex::*;
use rinex::observation::Record;
use super::Context;
use std::str::FromStr;
use plotters::prelude::*;

pub fn plot(ctx: &mut Context, record: &Record) {
    // extract observations in form (observable, sv, data)
    // one per x axis point
    // so we can plot easily
    let data: Vec<_> = record
        .iter()
        .enumerate()
        .map(|(index, (e, (clock_offset, vehicules)))| {
            vehicules.iter()
                .map(|(sv, observables)| {
                    observables.iter()
                        .map(|(observable, observation)| {
                            (observable.clone(),
                            sv.clone(),
                            observation.obs)
                        })
                })
                .flatten()
        })
        .flatten()
        .collect();
    // actual plotting
    for (title, plot) in &ctx.plots {
        for (observable, chart) in &ctx.charts {
            // <o TODO
            //    pick a symbol per carrier signal
            // draw one serie per vehicule
            for (svnn, color) in &ctx.colors { 
                chart
                    .clone()
                    .restore(&plot)
                    .draw_series(LineSeries::new(
                    data.iter()
                        .zip(ctx.t_axis.iter())
                        .filter_map(|((obs, sv, data), t)| {
                            if is_phase_carrier_obs_code!(obs) && observable == "PH" {
                                let expected = Sv::from_str(&svnn)
                                    .expect("faulty plot context: unrecognized curve identifier");
                                if *sv == expected {
                                    Some((*t, *data)) // grab (x,y)
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        }),
                    color.stroke_width(3)
                ))
                .expect("failed to draw observations")
                .label(svnn.to_string())
                .legend(|(x, y)| {
                        //let color = ctx.colors.get(&vehicule.to_string()).unwrap();
                        PathElement::new(vec![(x, y), (x + 20, y)], BLACK)
                });
            }
        }
    }
}
