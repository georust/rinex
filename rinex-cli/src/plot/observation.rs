//! Observation record plotting method
use rinex::*;
use rinex::observation::Record;
use super::Context;
use std::str::FromStr;
use plotters::prelude::*;

pub fn plot(ctx: &mut Context, record: &Record) {
    //TODO
    // emphasize LLI/SSI somehow ?

    // extract observations in form (observable, sv, data)
    // one per x axis point
    // so we can plot easily
    let data: Vec<_> = record
        .iter()
        .map(|(e, (clock_offset, vehicules))| {
            vehicules.iter()
                .map(|(sv, observables)| {
                    observables.iter()
                        .map(|(observable, observation)| {
                            (observable,
                            sv.clone(),
                            observation.obs)
                        })
                })
                .flatten()
        })
        .flatten()
        .collect();
    // for each plot
    for (_, plot) in &ctx.plots {
        // for each chart
        for (chart_id, chart) in &ctx.charts {
            // <o TODO
            //    pick a symbol per carrier signal
            // for each curve 
            for (svnn, color) in &ctx.colors { 
                chart
                    .clone()
                    .restore(&plot)
                    .draw_series(LineSeries::new(
                        data.iter()
                            .zip(ctx.t_axis.iter())
                            .filter_map(|((obs, sv, data), t)| {
                                // match chart <=> physics
                                let mut physics_matched = is_phase_carrier_obs_code!(obs) && chart_id.eq("PH");
                                physics_matched |= is_doppler_obs_code!(obs) && chart_id.eq("DOP");
                                physics_matched |= is_pseudo_range_obs_code!(obs) && chart_id.eq("PR");
                                physics_matched |= is_sig_strength_obs_code!(obs) && chart_id.eq("SSI");
                                if physics_matched {
                                    // match curve/sv
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
