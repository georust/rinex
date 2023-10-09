//use itertools::Itertools;
use crate::plot::PlotContext;
use plotly::layout::MapboxStyle;
use rinex::prelude::Epoch;
use rinex::prelude::RnxContext;
use std::collections::HashMap;

pub fn plot_tec_map(
    ctx: &RnxContext,
    _borders: ((f64, f64), (f64, f64)),
    plot_ctx: &mut PlotContext,
) {
    let _cmap = colorous::TURBO;
    //let hover_text: Vec<String> = ctx.primary_data().epoch().map(|e| e.to_string()).collect();
    /*
     * TEC map visualization
     * plotly-rs has no means to animate plots at the moment
     * therefore.. we create one plot for all existing epochs
     */
    for (_index, epoch) in ctx.primary_data().epoch().enumerate() {
        let content: Vec<_> = ctx
            .primary_data()
            .tec()
            .filter_map(|(t, lat, lon, h, tec)| {
                if t == epoch {
                    Some((lat, lon, h, tec))
                } else {
                    None
                }
            })
            .collect();

        plot_ctx.add_world_map(
            &epoch.to_string(),
            true,
            MapboxStyle::StamenTerrain,
            (32.5, -40.0),
            1,
        );

        let mut lat: HashMap<String, f64> = HashMap::new();
        let mut lon: HashMap<String, f64> = HashMap::new();
        let mut z: HashMap<String, f64> = HashMap::new();
        for (tec_lat, tec_lon, _, tec) in content {
            lat.insert(Epoch::default().to_string(), tec_lat);
            lon.insert(Epoch::default().to_string(), tec_lon);
            z.insert(Epoch::default().to_string(), tec);
        }

        /* plot the map grid */
        //let grid = ScatterMapbox::new(lat.clone(), lon.clone())
        //    .marker(
        //        Marker::new()
        //            .size(3)
        //            .symbol(MarkerSymbol::Circle)
        //            .color(NamedColor::Black)
        //            .opacity(0.5),
        //    )
        //    .name("grid");
        //plot_ctx.add_trace(grid);

        //let map = AnimatedDensityMapbox::new(lat.clone(), lon.clone(), z)
        //    .title("TEST")
        //    .name(epoch.to_string())
        //    .opacity(0.66)
        //    .hover_text_array(hover_text.clone())
        //    .zauto(true)
        //    //.animation_frame("test")
        //    .zoom(3);
        //plot_ctx.add_trace(map);
    }
}
