//use itertools::Itertools;
use crate::plot::PlotContext;
use plotly::{
    color::NamedColor,
    common::{Marker, MarkerSymbol}, //color::Rgba},
    layout::MapboxStyle,
    DensityMapbox,
    //scatter_mapbox::Fill,
    ScatterMapbox,
};
use rinex_qc::QcContext;

pub fn plot_tec_map(
    ctx: &QcContext,
    _borders: ((f64, f64), (f64, f64)),
    plot_ctx: &mut PlotContext,
) {
    let _cmap = colorous::TURBO;
    /*
     * TEC map visualization
     * plotly-rs has no means to animate plots at the moment
     * therefore.. we create one plot for all existing epochs
     */
    for (index, epoch) in ctx.primary_data().epoch().enumerate() {
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

        let mut lat: Vec<f64> = Vec::new();
        let mut lon: Vec<f64> = Vec::new();
        let mut z: Vec<f64> = Vec::new();
        for (tec_lat, tec_lon, _, tec) in content {
            lat.push(tec_lat);
            lon.push(tec_lon);
            z.push(tec);
        }

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

        let map = DensityMapbox::new(lat.clone(), lon.clone(), z)
            .opacity(0.66)
            .hover_text
            .zauto(true)
            .zoom(3);
        plot_ctx.add_trace(map);
    }
}
