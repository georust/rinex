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
    plot_ctx.add_world_map(MapboxStyle::StamenTerrain, (32.5, -40.0), 1);

    let header = ctx.primary_data().header.ionex.as_ref().unwrap();
    let record = ctx.primary_data().record.as_ionex().unwrap(); // cannot fail

    let first_epoch = ctx
        .primary_data()
        .first_epoch()
        .expect("failed to determine first epoch");

    let first_epoch_content: Vec<_> = ctx
        .primary_data()
        .tec()
        .filter_map(|(t, lat, lon, h, tec)| {
            if t == first_epoch {
                Some((lat, lon, h, tec))
            } else {
                None
            }
        })
        .collect();

    let mut lat: Vec<f64> = Vec::new();
    let mut lon: Vec<f64> = Vec::new();
    let mut z: Vec<f64> = Vec::new();
    for (tec_lat, tec_lon, _, tec) in first_epoch_content {
        lat.push(tec_lat);
        lon.push(tec_lon);
        z.push(tec);
        // println!("({}, {}): {}", tec_lat, tec_lon, tec);
    }

    let grid = ScatterMapbox::new(lat.clone(), lon.clone())
        .marker(
            Marker::new()
                .size(3)
                .symbol(MarkerSymbol::Circle)
                .color(NamedColor::Black)
                .opacity(0.5),
        )
        .name("TEC Grid");

    plot_ctx.add_trace(grid);

    let map = DensityMapbox::new(lat.clone(), lon.clone(), z)
        .opacity(0.66)
        .zauto(true)
        .zoom(3);
    //.color_continuous_scale(
    //    vec![
    //        (0, NamedColor::Black),
    //        (100, NamedColor::White),
    //    ].into_iter()
    //        .collect()
    //)
    //.radius(5);
    plot_ctx.add_trace(map);

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
