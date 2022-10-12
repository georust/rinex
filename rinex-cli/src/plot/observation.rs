//! Observation record plotting method
use rinex::*;
use rinex::observation::Record;
use super::Context;
use plotters::prelude::*;

pub fn plot(mut ctx: Context, record: &Record) {
    // extract observations in form (e, observable, sv, data)
    // for every single epoch, so we can plot easily
    let data: Vec<_> = record
        .iter()
        .enumerate()
        .map(|(index, (e, (clock_offset, vehicules)))| {
            vehicules.iter()
                .map(|(sv, observables)| {
                    observables.iter()
                        .map(|(observable, observation)| {
                            (10.0 as f64,
                            observable,
                            sv.clone(),
                            observation.obs)
                        })
                })
                .flatten()
        })
        .flatten()
        .collect();
    // actual plotting
    for (identifier, _) in ctx.y_ranges.iter() {
        // <o TODO
        //    pick a symbol per carrier signal
        // draw one serie per vehicule
        for vehicule in ctx.vehicules.clone().iter() {
            if let Some(chart) = ctx.charts.get_mut(identifier) {
                let color = ctx.colors.get(&vehicule.to_string())
                    .unwrap();
                chart.draw_series(LineSeries::new(
                    data.iter()
                        .filter_map(|(t, obs, sv, data)| {
                            let physics_matched: bool = match identifier.as_str() {
                                "PH" => is_phase_carrier_obs_code!(obs),
                                "PR" => is_pseudo_range_obs_code!(obs),
                                "SSI" => is_sig_strength_obs_code!(obs),
                                 _ => is_doppler_obs_code!(obs),
                            };
                            println!("MATCHED {}", physics_matched);
                            if physics_matched {
                                if sv == vehicule {
                                    Some((*t, *data))
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        }),
                    color.stroke_width(3)
                )).unwrap()
                .label(vehicule.to_string())
                .legend(|(x, y)| {
                    //let color = ctx.colors.get(&vehicule.to_string()).unwrap();
                    PathElement::new(vec![(x, y), (x + 20, y)], BLACK)
                 });
                 chart
                    .configure_series_labels()
                    .border_style(&BLACK)
                    .background_style(WHITE.filled())
                    .draw()
                    .unwrap();
            } else {
                println!("NONE");
            }
        }
    }
}
