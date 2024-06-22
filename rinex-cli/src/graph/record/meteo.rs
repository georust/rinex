use crate::cli::Context;
use crate::graph::{build_chart_epoch_axis, csv::CSV, PlotContext}; //generate_markers};
use plotly::common::{Marker, MarkerSymbol, Mode};
use plotly::ScatterPolar;
use rinex::prelude::Observable;
use statrs::statistics::Statistics;

/*
 * Plots Meteo observations
 */
pub fn plot_meteo_observations(ctx: &Context, plot_context: &mut PlotContext, csv_export: bool) {
    let rnx = ctx.data.meteo().unwrap(); // infaillible
                                         /*
                                          * 1 plot per physics
                                          */
    for observable in rnx.observable() {
        //if csv_export {
        //    let filename = format!("{}.csv", observable);
        //    let title = format!("{} observations", observable);
        //    let labels = format!("Epoch, {} [{}]", observable, unit);
        //    let mut csv = CSV::new(&ctx.workspace, &filename, &title, &labels)
        //        .expect("failed to render data as CSV");
        //    csv.export_timedomain(&data_x, &data_y)
        //        .expect("failed to render data as CSV");
        //}
    }
    /*
     * Plot Wind Direction
     */
    let wind_speed = rnx.wind_speed().map(|(_, speed)| speed).collect::<Vec<_>>();
    let wind_speed_max = wind_speed.max();

    let theta = rnx
        .wind_direction()
        .map(|(_, angle)| angle)
        .collect::<Vec<_>>();

    let has_wind_direction = !theta.is_empty();

    let rho = rnx
        .wind_direction()
        .map(|(t, _)| {
            if let Some(speed) = rnx
                .wind_speed()
                .find(|(ts, _)| *ts == t)
                .map(|(_, speed)| speed)
            {
                speed / wind_speed_max
            } else {
                1.0_f64
            }
        })
        .collect::<Vec<_>>();

    let trace = ScatterPolar::new(theta, rho)
        .marker(Marker::new().symbol(MarkerSymbol::TriangleUp))
        .connect_gaps(false)
        .name("Wind direction [°]");
    if has_wind_direction {
        plot_context.add_polar2d_plot("Wind direction (r= normalized speed)");
        plot_context.add_trace(trace);
    }
}
