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

pub fn plot_tec_map(plot_ctx: &mut PlotContext, map_grid: &Grid, record: &Record) {
    // add a world map
    plot_ctx.add_world_map(MapboxStyle::OpenStreetMap, (32.5, -40.0), 4);
    // We currently have no means to animate a plot,
    // therefore have no means to visualize several epochs (like a day course for instance)
    // So we only grab the first epoch
    let (first_epoch, map3d) = record
        .first_key_value()
        .expect("failed to grab first ionosphere map epoch");
    // We currently have no means to plot a 3D IONEX.
    // So we only grab the first altitude data set, whatever the value.
    // This will work fine 2D IONEX which is the most common type.
    let (_z0, z0_maps) = map3d.get(0).expect("failed to retrieve z=0 maps");
    // draw map grid:
    // one small marker on every (lat, lon) coordinates to form a grid
    let latitudes = record.latitudes();
    let longitudes = record.longitudes();
    let mut lat_grid: Vec<f64> = Vec::with_capacity(latitudes.len() * longitudes.len());
    let mut lon_grid: Vec<f64> = Vec::with_capacity(latitudes.len() * longitudes.len());
    for lat in &latitudes {
        for lon in &longitudes {
            lat_grid.push(*lat);
            lon_grid.push(*lon);
        }
    }
    let grid = ScatterMapbox::new(lat_grid, lon_grid)
        .marker(
            Marker::new()
                .size(2)
                .symbol(MarkerSymbol::Circle)
                .color(NamedColor::Black)
                .opacity(0.5),
        )
        .show_legend(true)
        .name("Grid");
    plot_ctx.add_trace(grid);
    // determine max|TEC| and min|TEC|
    // so we can define a color map
    let (mut tec_min, mut tec_max) = (f64::INFINITY, -f64::INFINITY);
    for (_lat, lon_maps) in z0_maps {
        for (_lon, tec) in lon_maps {
            if *tec < tec_min {
                tec_min = *tec;
            }
            if *tec > tec_max {
                tec_max = *tec;
            }
        }
    }
    // determine the color map
    let cmap = colorous::TURBO;
    // browse by smallest area, defined as a rectangle
    //    (x0,y0)            (x0+dx, y0)
    //          *------------*
    //          |            |
    //(x0,y0+dy)*------------*(x0+dx, y0+dx)
    //where dy and dx are defined by the map grid.
    // determine number of total rectangles to draw
    for (lon0, lon1) in longitudes.iter().zip(longitudes.iter().skip(1)) {
        if *lon1 < 20.0 && *lon0 > -20.0 {
            println!("LON : {} {}", lon0, lon1);
            for (lat0, lat1) in latitudes.iter().zip(latitudes.iter().skip(1)) {
                if *lat1 < 30.0 && *lat0 > -30.0 {
                    println!("{} {}", lat0, lat1);
                    let rect = ScatterMapbox::new(
                        vec![*lat1, *lat1, *lat0, *lat0],
                        vec![*lon0, *lon1, *lon1, *lon0],
                    )
                    .fill(Fill::ToSelf)
                    .fill_color(Rgba::new(0, 0, 0, 0.66));
                    //.marker(Marker::new().color(NamedColor::Black));
                    plot_ctx.add_trace(rect);
                }
            }
        }
    }

    //for ((lat1, lon1_map), (lat2, lon2_map)) in z0_maps
    //    .iter()
    //        .zip(z0_maps.iter().skip(1))
    //{
    //    println!("lat 1 : {}, lat 2: {}", lat1, lat2);
    //    for ((lon00, tec00), (lon01, tec01)) in lon1_map
    //        .iter()
    //            .zip(lon1_map.iter().skip(1))
    //    {
    //        for ((lon10, tec10), (lon11, tec11)) in lon2_map
    //            .iter()
    //                .zip(lon2_map.iter().skip(1))
    //            {
    //                println!("lon00: {} lon01: {} lon11: {}, lon10: {}", lon00, lon01, lon11, lon10);
    //            }
    //    }
    //}

    //let rect
    //    = ScatterMapbox::new(
    //        vec![43, 43, 45, 45],
    //        vec![-74, -70, -70, -74])
    //    .fill(Fill::ToSelf)
    //    .fill_color(Rgba::new(0, 0, 0, 0.66))
    //    .marker(Marker::new().color(NamedColor::Black));
    //for ((lat0, lon0_maps), (lat1, lon1_maps)) in z0_maps
    //    .iter()
    //    .zip(z0_maps.iter().skip(1))
    //{
    //    for ((lon0, data0), (lon1, data1)) in lon0_maps
    //        .iter()
    //        .zip(lon0_maps.iter().skip(1))
    //    {
    //        let rect = ScatterMapbox::new(
    //            vec![*lat0, *lat0, *lat1, *lat1],
    //            vec![*lon0, *lon1, *lon1, *lon0],
    //        ).fill(Fill::ToSelf)
    //        .fill_color(Rgba::new(0, 0, 0, 0.66))
    //        .marker(Marker::new().color(NamedColor::Black));
    //        plot_ctx.add_trace(rect);
    //    }
    //}

    //let square = ScatterMapbox::new(vec![43, 43, 45, 45], vec![-74, -70, -70, -74])
    //    .fill(Fill::ToSelf)
    //    //.fill_color(Rgba::new(0, 0, 0, 0.66))
    //    .marker(Marker::new().color(NamedColor::Orange));

    //plot_ctx.add_trace(square);
}
