//use itertools::Itertools;
use crate::plot::PlotContext;
use plotly::color::NamedColor;
use plotly::common::{Marker, MarkerSymbol};
use plotly::layout::MapboxStyle;
use plotly::DensityMapbox;
use plotly::ScatterMapbox;
use rinex::prelude::Rinex;

pub fn plot_tec_map(data: &Rinex, _borders: ((f64, f64), (f64, f64)), plot_ctx: &mut PlotContext) {
    let _cmap = colorous::TURBO;
    //TODO
    //let hover_text: Vec<String> = ctx.primary_data().epoch().map(|e| e.to_string()).collect();
    /*
     * TEC map visualization
     * plotly-rs has no means to animate plots at the moment
     * therefore.. we create one plot for all remaining Epochs
     */
    for epoch in data.epoch() {
        let lat: Vec<_> = data
            .tec()
            .filter_map(
                |(t, lat, _, _, _)| {
                    if t == epoch {
                        Some(lat)
                    } else {
                        None
                    }
                },
            )
            .collect();
        let lon: Vec<_> = data
            .tec()
            .filter_map(
                |(t, _, lon, _, _)| {
                    if t == epoch {
                        Some(lon)
                    } else {
                        None
                    }
                },
            )
            .collect();
        let tec: Vec<_> = data
            .tec()
            .filter_map(
                |(t, _, _, _, tec)| {
                    if t == epoch {
                        Some(tec)
                    } else {
                        None
                    }
                },
            )
            .collect();

        plot_ctx.add_world_map(
            &epoch.to_string(),
            true,
            MapboxStyle::StamenTerrain,
            (32.5, -40.0),
            1,
        );

        /* plot the map grid */
        let grid = ScatterMapbox::new(lat.clone(), lon.clone())
            .marker(
                Marker::new()
                    .size(3)
                    .symbol(MarkerSymbol::Circle)
                    .color(NamedColor::Black)
                    .opacity(0.5),
            )
            .name("grid");
        plot_ctx.add_trace(grid);

        //let map = AnimatedDensityMapbox::new(lat.clone(), lon.clone(), z)
        let map = DensityMapbox::new(lat.clone(), lon.clone(), tec.clone())
            //.title("TEST")
            .name(epoch.to_string())
            .opacity(0.66)
            //.hover_text_array(hover_text.clone())
            .zauto(true)
            //.animation_frame("test")
            .zoom(3);
        plot_ctx.add_trace(map);
    }
}
