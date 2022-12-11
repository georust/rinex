use crate::plot::Context;
//use itertools::Itertools;
use plotly::{
    color::NamedColor,
    common::{Marker, MarkerSymbol}, //color::Rgba},
    layout::MapboxStyle,
    //scatter_mapbox::Fill,
    ScatterMapbox,
};
use rinex::ionex::*;

pub fn plot_tec_map(ctx: &mut Context, borders: ((f64, f64), (f64, f64)), record: &Record) {
    let cmap = colorous::TURBO;
    ctx.add_world_map(MapboxStyle::OpenStreetMap, (32.5, -40.0), 1);

    let mut grid_lat: Vec<f64> = Vec::new();
    let mut grid_lon: Vec<f64> = Vec::new();
    let mut tec_max = -std::f64::INFINITY;
    for (e_index, (e, (tec, _, _))) in record.iter().enumerate() {
        for point in tec {
            if e_index == 0 {
                // grab grid definition
                grid_lat.push(point.latitude.into());
                grid_lon.push(point.longitude.into());
            }
            if point.value > tec_max {
                tec_max = point.value;
            }
        }
    }

    let grid = ScatterMapbox::new(grid_lat, grid_lon)
        .marker(
            Marker::new()
                .size(5)
                .symbol(MarkerSymbol::Circle)
                .color(NamedColor::Black)
                .opacity(0.5),
        )
        .name("TEC Grid");
    ctx.add_trace(grid);

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
