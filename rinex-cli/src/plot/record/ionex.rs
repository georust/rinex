use crate::plot::PlotContext;
//use itertools::Itertools;
use geo::{
    coord, ClosestPoint, Contains, Coord, Intersects, LineString, MultiPolygon, Polygon, Simplify,
    Winding,
};
use itertools::Itertools;
use plotly::{
    color::{NamedColor, Rgba},
    common::{Marker, MarkerSymbol}, //color::Rgba},
    contour::Contour,
    layout::MapboxStyle,
    scatter_mapbox::Fill,
    ScatterMapbox,
};
use rinex::ionex::*;

enum ExtendDirection {
    West,
    South,
}

fn southern_latitude(line: &LineString) -> f64 {
    let mut lat = 80.0_f64;
    for coord in line.coords().into_iter() {
        if coord.y < lat {
            lat = coord.y;
        }
    }
    lat
}

fn western_longitude(line: &LineString) -> f64 {
    let mut lon = 180.0_f64;
    for coord in line.coords().into_iter() {
        if coord.x < lon {
            lon = coord.x;
        }
    }
    lon
}

// merges two areas toghether
//fn merge(poly: &Polygon, area: &Polygon, direction: ExtendDirection) -> Polygon {
//    match direction {
//        ExtendDirection::West => {
//        },
//        ExtendDirection::South => {
//        },
//    }
//}

// two areas are considered as neighbor is they share at least two coordinates
fn is_neighbor(area0: &Polygon, area1: &Polygon) -> bool {
    let ext0 = area0.exterior();
    let ext1 = area1.exterior();
    for coord in ext0.coords() {
        if ext1.contains(coord) {
            for rhs_coord in ext0.coords() {
                if (rhs_coord != coord) {
                    if ext1.contains(rhs_coord) {
                        //println!("EXT0 {:?}", ext0);
                        //println!("EXT1 {:?}", ext1);
                        return true;
                    }
                }
            }
        }
    }
    false
}

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
    // grid attributes
    let dlat = map_grid.lat_grid.spacing;
    let dlon = map_grid.lon_grid.spacing;
    // we use the smallest grid dimension
    // as simplify() algorithm parameters
    let simplify_dx = if dlat < dlon { dlat } else { dlon };

    // draw map grid: one small marker on every (lat, lon) coordinates
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
    let mut polygons: Vec<(f64, Polygon<f64>)> = Vec::new();
    /////////////////////////////////////////////////////////
    // Smallest area is a tiny rectangle :
    //    (x0,y0)            (x0+dx, y0)
    //          *------------*
    //          |            |
    //(x0,y0+dy)*------------*(x0+dx, y0+dy)
    //
    // with dx, dy defined by the map grid.
    //
    // for each rectangle, we try to extend neiborghing areas
    // if the TEC estimates do match
    /////////////////////////////////////////////////////////
    for (lon0, lon1) in longitudes.iter().zip(longitudes.iter().skip(1)) {
        if *lon0 > 150.0 && *lon1 < 180.0 {
            //TODO: remove this filter
            for (lat0, lat1) in latitudes.iter().zip(latitudes.iter().skip(1)) {
                if *lat1 < 10.0 && *lat0 > -10.0 {
                    // TODO: remove this filter
                    println!("LAT : {} {}, LON : {} {}", lat0, lat1, lon0, lon1);
                    // determine average tec value
                    // in the  (x0,y0), ... (x0+dx, y0+dy) square
                    let mut tec_00 = 0.0_f64;
                    let mut tec_01 = 0.0_f64;
                    let mut tec_10 = 0.0_f64;
                    let mut tec_11 = 0.0_f64;
                    for (lat, lon_maps) in z0_maps {
                        if lat == lat0 {
                            for (lon, tec) in lon_maps {
                                if lon == lon0 {
                                    tec_00 = *tec;
                                } else if lon == lon1 {
                                    tec_01 = *tec;
                                }
                            }
                        } else if lat == lat1 {
                            for (lon, tec) in lon_maps {
                                if lon == lon0 {
                                    tec_10 = *tec;
                                } else if lon == lon1 {
                                    tec_11 = *tec;
                                }
                            }
                        }
                    }
                    // average tec value
                    let tec = (tec_00 + tec_01 + tec_10 + tec_11) / 4.0;
                    // build this rectangle
                    let contour = LineString::from(vec![
                        (*lon0, *lat1),
                        (*lon1, *lat1),
                        (*lon1, *lat0),
                        (*lon0, *lat0),
                    ]);
                    let rect = Polygon::new(contour.clone(), vec![]);
                    let mut intersects = false;
                    for (_, poly) in polygons.iter_mut() {
                        let exterior = poly.exterior();
                        intersects |= exterior.intersects(&rect);
                        if intersects {
                            println!("WESTERN: {}", western_longitude(&exterior));
                            println!("SOUTHERN: {}", southern_latitude(&exterior));
                            //let mut coords: Vec<Coord<f64>> = exterior.coords().into_iter().map(|c| *c).collect();
                            //let _ = coords.pop();
                            //println!("=====================");
                            //println!("coords : {:?}", coords);
                            //coords.push(Coord{ x: *lat1, y: *lon0});
                            //coords.push(Coord{ x: *lat1, y: *lon1});
                            //coords.push(Coord{ x: *lat0, y: *lon1});
                            //coords.push(Coord{ x: *lat0, y: *lon0});
                            //println!("coords : {:?}", coords);
                            //println!("=====================");
                            //let mut linestring = LineString::from(coords);
                            //linestring.simplify(&100.0);
                            ////linestring.make_cw_winding();
                            //*poly = Polygon::new(linestring, vec![]);
                        }
                    }
                    if !intersects {
                        // add new area
                        polygons.push((tec, rect));
                    }
                }
            }
            println!("========");
        }
    }
    // draw all areas
    //for (tec, poly) in polygons {
    //    let exterior = poly.exterior();
    //    let longitudes :Vec<f64> = exterior.coords().into_iter().map(|c| c.x).collect();
    //    let latitudes :Vec<f64> = exterior.coords().into_iter().map(|c| c.y).collect();
    //    let color = cmap.eval_continuous(tec / tec_max);
    //    let (r, g, b) = (color.r, color.g, color.b);
    //    let area =
    //        ScatterMapbox::new(latitudes, longitudes)
    //            .fill(Fill::ToSelf)
    //            .connect_gaps(false)
    //            .fill_color(Rgba::new(r, g, b, 0.66));
    //    plot_ctx.add_trace(area);
    //}
    //let area0 = ScatterMapbox::new(
    //    vec![10.0, 10.0, 10.0, 25.0, 25.0, 15.0, 15.0], vec![125.0, 130.0, 135.0, 135.0, 130.0, 130.0, 125.0])
    //    .fill(Fill::ToSelf)
    //    .connect_gaps(false)
    //    .fill_color(Rgba::new(30, 30, 30, 0.66));
    //plot_ctx.add_trace(area0);
    for (tec, polygon) in &polygons {
        let ext = polygon.exterior();
        let longitudes: Vec<f64> = ext.coords().into_iter().map(|c| c.x).collect();
        let latitudes: Vec<f64> = ext.coords().into_iter().map(|c| c.y).collect();
        let area = ScatterMapbox::new(latitudes, longitudes)
            .fill(Fill::ToSelf)
            .fill_color(Rgba::new(50, 50, 50, 0.66));
        plot_ctx.add_trace(area);
    }
}
