use crate::plot::PlotContext;
//use itertools::Itertools;
use plotly::{
    color::{NamedColor, Rgba},
    common::{Marker, MarkerSymbol}, //color::Rgba},
    contour::Contour,
    layout::MapboxStyle,
    scatter_mapbox::Fill,
    ScatterMapbox,
};
use rinex::ionex::*;

pub fn plot_tec_map(
    plot_ctx: &mut PlotContext,
    _borders: ((f64, f64), (f64, f64)),
    record: &Record,
) {
    let _cmap = colorous::TURBO;
    let mut grid_lat: Vec<f64> = Vec::new();
    let mut grid_lon: Vec<f64> = Vec::new();
    let (mut tec_min, mut tec_max) = (f64::INFINITY, -f64::INFINITY);
    // add a world map
    plot_ctx.add_world_map(MapboxStyle::OpenStreetMap, (32.5, -40.0), 1);
    // We currently have no means to animate a plot,
    // therefore have no means to visualize several epochs (like a day course for instance)
    // So we only grab the first epoch
    let (first_epoch, map3d) = record
        .first_key_value()
        .expect("failed to grab first ionosphere map epoch");
    // We currently have no means to plot a 3D IONEX.
    // So we only grab the first altitude data set, whatever the value.
    // This will work fine 2D IONEX which is the most common type.
    let z0_maps = map3d.get(0).expect("failed to retrieve z=0 maps");
    // grab grid layer and TEC range (min, max)
    //for point in map3d {
    //    grid_lat.push(point.latitude.into());
    //    grid_lon.push(point.longitude.into());
    //    if point.value > tec_max {
    //        tec_max = point.value;
    //    }
    //    if point.value < tec_min {
    //        tec_min = point.value;
    //    }
    //}
    // determine the color map
    let cmap = colorous::TURBO;
    let cmap_max = tec_max + tec_min;
    // create the grid from the determine grid layers (list of lat,lon)
    let grid = ScatterMapbox::new(grid_lat, grid_lon)
        .marker(
            Marker::new()
                .size(4)
                .symbol(MarkerSymbol::Circle)
                .color(NamedColor::Black)
                .opacity(0.25),
        )
        .show_legend(true)
        .name("TEC Grid");
    plot_ctx.add_trace(grid);
    // iterate over the map
    // create a square
    let square = ScatterMapbox::new(
        vec![47, 47, 45, 45],     //latitudes
        vec![-74, -70, -70, -74], //longitudes
    )
    .fill(Fill::ToSelf)
    //.fill_color(Rgba::new(0, 0, 0, 0.66))
    .marker(Marker::new().color(NamedColor::Black));

    plot_ctx.add_trace(square);

    let square = ScatterMapbox::new(vec![43, 43, 45, 45], vec![-74, -70, -70, -74])
        .fill(Fill::ToSelf)
        //.fill_color(Rgba::new(0, 0, 0, 0.66))
        .marker(Marker::new().color(NamedColor::Orange));

    plot_ctx.add_trace(square);
    /*
     * Build heat map,
     * we have no means to plot several heat map (day course)
     * at the moment, we just plot the 1st epoch
     */
    /*
    for (e_index, (e, (tec, _, _))) in record.iter().enumerate() {
        if e_index == 0 {
            // form the smallest area from that grid
            for rows in
            for i in 0..tec.len() / 4 {
                println!("MAPS {}", i);
                //for i in 0..maps.len() / 2 {
                /*
                let mut row1 = maps[0]
                let mut row2 = maps[1]
                if let Some(points12) = row1.next() {
                    if let Some(points34) = row2.next() {
                        // average TEC in that area
                        let tec = (points12[0].value + points12[1].value
                            +points34[0].value + points34[1].value ) / 4.0;
                        let color = cmap.eval_continuous(tec / tec_max);
                        let area = ScatterMapbox::new(
                            vec![points12[0].latitude, points12[1].latitude, points34[0].latitude, points34[1].latitude],
                            vec![points12[0].longitude, points12[1].longitude, points34[0].longitude, points34[1].longitude],
                        )
                        .opacity(0.5)
                        .fill(Fill::ToSelf)
                        .fill_color(Rgba::new(color.r, color.g, color.b, 0.9));
                        ctx.add_trace(area);
                        println!("YES");
                    }
                }*/
            }
        } else {
            break ; // we need animations for that
        }
    }*/
}
