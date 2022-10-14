//! Observation record plotting method
use rinex::*;
use rinex::observation::Record;
use super::Context;
use plotters::prelude::*;

pub fn plot(mut ctx: Context, record: &Record) {
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
                            (observable,
                            sv.clone(),
                            observation.obs)
                        })
                })
                .flatten()
        })
        .flatten()
        .collect();
    // actual plotting
    for (observable, chart) in ctx.charts.iter_mut() {
        // <o TODO
        //    pick a symbol per carrier signal
        // draw one serie per vehicule
        for (svnn, curve) in ctx.curves {
            let color = ctx.colors.get(svnn)
                .expect("faulty plot context: one color per sat vehicule should have been assigned");
            chart.draw_series(LineSeries::new(
                data.iter()
                    .filter_map(|(obs, sv, data)| {
                        if obs == observable { // physics matched for this chart
                            let expected = Sv::from_str(svnn)
                                .expect("faulty plot context: unrecognized curve identifier");
                            if sv == expected {
                                Some((*t, *data)) // grab (x,y)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }),
                color.stroke_width(3)
            )).unwrap()
            .label(vehicule.to_string());
            //.legend(|(x, y)| {
            //        //let color = ctx.colors.get(&vehicule.to_string()).unwrap();
            //        PathElement::new(vec![(x, y), (x + 20, y)], BLACK)
            //    ))

        }
    }
}
