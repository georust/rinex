use rinex::{
    navigation::Ephemeris,
    prelude::{GroundPosition, Rinex},
};
use sp3::prelude::{Constellation, Epoch, SP3, SV};
use std::collections::HashMap;

use qc_traits::processing::{Filter, Preprocessing};

use map_3d::{ecef2geodetic, Ellipsoid};

use crate::{
    plot::{MapboxStyle, MarkerSymbol, Mode, NamedColor},
    prelude::{html, Markup, Plot, QcContext, Render},
};

struct BrdcSp3Report {
    err_x: Plot,
    err_y: Plot,
    err_z: Plot,
}

impl BrdcSp3Report {
    fn new(sp3: &SP3, brdc: &Rinex) -> Self {
        let mut t = HashMap::<SV, Vec<Epoch>>::new();
        let mut error_x = HashMap::<SV, Vec<f64>>::new();
        let mut error_y = HashMap::<SV, Vec<f64>>::new();
        let mut error_z = HashMap::<SV, Vec<f64>>::new();
        for (t_sp3, sv_sp3, pos_sp3) in sp3.sv_position() {
            if let Some((brdc_x, brdc_y, brdc_z)) = brdc.sv_position(sv_sp3, t_sp3) {
                if let Some(t) = t.get_mut(&sv_sp3) {
                    t.push(t_sp3);
                } else {
                    t.insert(sv_sp3, vec![t_sp3]);
                }
                if let Some(x) = error_x.get_mut(&sv_sp3) {
                    x.push(brdc_x - pos_sp3.0);
                } else {
                    error_x.insert(sv_sp3, vec![brdc_x - pos_sp3.0]);
                }
                if let Some(y) = error_y.get_mut(&sv_sp3) {
                    y.push(brdc_y - pos_sp3.1);
                } else {
                    error_y.insert(sv_sp3, vec![brdc_y - pos_sp3.1]);
                }
                if let Some(z) = error_z.get_mut(&sv_sp3) {
                    z.push(brdc_z - pos_sp3.2);
                } else {
                    error_z.insert(sv_sp3, vec![brdc_z - pos_sp3.2]);
                }
            }
        }
        Self {
            err_x: {
                let mut plot = Plot::timedomain_plot(
                    "sp3_brdc_err_x",
                    "X(ecef) coordinates error",
                    "Error [m]",
                    true,
                );
                for (sv, t) in t.iter() {
                    let error_x = error_x.get(&sv).unwrap();
                    let trace = Plot::timedomain_chart(
                        &sv.to_string(),
                        Mode::Markers,
                        MarkerSymbol::Diamond,
                        &t,
                        error_x.to_vec(),
                    );
                    plot.add_trace(trace);
                }
                plot
            },
            err_y: {
                let mut plot = Plot::timedomain_plot(
                    "sp3_brdc_err_y",
                    "Y(ecef) coordinates error",
                    "Error [m]",
                    true,
                );
                for (sv, t) in t.iter() {
                    let error_y = error_y.get(&sv).unwrap();
                    let trace = Plot::timedomain_chart(
                        &sv.to_string(),
                        Mode::Markers,
                        MarkerSymbol::Diamond,
                        &t,
                        error_y.to_vec(),
                    );
                    plot.add_trace(trace);
                }
                plot
            },
            err_z: {
                let mut plot = Plot::timedomain_plot(
                    "sp3_brdc_err_z",
                    "Z(ecef) coordinates error",
                    "Error [m]",
                    true,
                );
                for (sv, t) in t.iter() {
                    let error_z = error_z.get(&sv).unwrap();
                    let trace = Plot::timedomain_chart(
                        &sv.to_string(),
                        Mode::Markers,
                        MarkerSymbol::Diamond,
                        &t,
                        error_z.to_vec(),
                    );
                    plot.add_trace(trace);
                }
                plot
            },
        }
    }
}

impl Render for BrdcSp3Report {
    fn render(&self) -> Markup {
        html! {
            div class="table-container" {
                table class="table is-bordered" {
                    tr {
                        (self.err_x.render())
                    }
                    tr {
                        (self.err_y.render())
                    }
                    tr {
                        (self.err_z.render())
                    }
                }
            }
        }
    }
}

pub struct OrbitReport {
    sky_plot: Plot,
    // map_proj: Plot,
    // globe_proj: Plot,
    brdc_sp3_err: HashMap<Constellation, BrdcSp3Report>,
}

impl OrbitReport {
    pub fn new(ctx: &QcContext, reference: Option<GroundPosition>, force_brdc_sky: bool) -> Self {
        let (x0, y0, z0) = reference.unwrap_or_default().to_ecef_wgs84();
        let brdc_skyplot = ctx.has_brdc_navigation() && ctx.has_sp3() && force_brdc_sky;

        let max_sv_visible = if brdc_skyplot { 2 } else { 4 };
        Self {
            sky_plot: {
                let mut plot = Plot::sky_plot("skyplot", "Sky plot", true);
                if let Some(sp3) = ctx.sp3() {
                    for (sv_index, sv) in sp3.sv().enumerate() {
                        let visible = sv_index < max_sv_visible;
                        let visible = true;
                        let t = sp3
                            .sv_position()
                            .filter_map(|(t, svnn, _)| if svnn == sv { Some(t) } else { None })
                            .collect::<Vec<_>>();
                        let rho = sp3
                            .sv_position()
                            .filter_map(|(t, svnn, sv_pos)| {
                                if svnn == sv {
                                    let el =
                                        Ephemeris::elevation_azimuth(sv_pos, (x0, y0, z0)).0.abs();
                                    Some(el)
                                } else {
                                    None
                                }
                            })
                            .collect::<Vec<_>>();
                        let theta = sp3
                            .sv_position()
                            .filter_map(|(t, svnn, sv_pos)| {
                                if svnn == sv {
                                    let az = Ephemeris::elevation_azimuth(sv_pos, (x0, y0, z0)).1;
                                    Some(az)
                                } else {
                                    None
                                }
                            })
                            .collect::<Vec<_>>();
                        let trace =
                            Plot::sky_trace(t, rho, theta, visible).name(format!("{:X}", sv));
                        plot.add_trace(trace);
                    }
                }
                plot
            },
            //map_proj: {
            //    let mut map_proj = Plot::world_map(
            //        "map_proj",
            //        "Map Projection",
            //        MapboxStyle::OpenStreetMap,
            //        (32.0, -40.0),
            //        1,
            //        true,
            //    );
            //    if let Some(sp3) = ctx.sp3() {
            //        for (sv_index, sv) in sp3.sv().enumerate() {
            //            let t = sp3.sv_position().filter_map(|(t, svnn, _)| {
            //                if svnn == sv {
            //                    Some(t)
            //                } else {
            //                    None
            //                }
            //            }).collect::<Vec<_>>();
            //            let lat_ddeg = sp3.sv_position().filter_map(|(t, svnn, pos)| {
            //                if svnn == sv {
            //                    let lat_rad = ecef2geodetic(pos.0, pos.1, pos.2, Ellipsoid::WGS84).0;
            //                    Some(lat_rad.to_degrees())
            //                } else {
            //                    None
            //                }
            //            }).collect::<Vec<_>>();
            //            let lon_ddeg = sp3.sv_position().filter_map(|(t, svnn, pos)| {
            //                if svnn == sv {
            //                    let long_rad = ecef2geodetic(pos.0, pos.1, pos.2, Ellipsoid::WGS84).1;
            //                    Some(long_rad.to_degrees())
            //                } else {
            //                    None
            //                }
            //            }).collect::<Vec<_>>();
            //            let map = Plot::mapbox(
            //                lat_ddeg,
            //                lon_ddeg,
            //                &sv.to_string(),
            //                MarkerSymbol::Diamond,
            //                NamedColor::Red,
            //                1.0,
            //                sv_index == 1);
            //            map_proj.add_trace(map);
            //        }
            //    }
            //    map_proj
            //},
            //globe_proj: {
            //    let mut map_proj = Plot::world_map(
            //        "map_proj",
            //        "Map Projection",
            //        MapboxStyle::OpenStreetMap,
            //        (32.0, -40.0),
            //        1,
            //        true,
            //    );
            //    map_proj
            //},
            brdc_sp3_err: {
                let mut reports = HashMap::<Constellation, BrdcSp3Report>::new();
                if let Some(sp3) = ctx.sp3() {
                    if let Some(nav) = ctx.brdc_navigation() {
                        for constellation in sp3.constellation() {
                            if let Some(constellation) = nav
                                .constellation()
                                .filter(|c| *c == constellation)
                                .reduce(|k, _| k)
                            {
                                let filter = Filter::equals(&constellation.to_string()).unwrap();
                                let focused_sp3 = sp3.filter(&filter);
                                let focused_nav = nav.filter(&filter);
                                let page = reports.insert(
                                    constellation,
                                    BrdcSp3Report::new(&focused_sp3, &focused_nav),
                                );
                            }
                        }
                    }
                }
                reports
            },
        }
    }
    pub fn html_inline_menu_bar(&self) -> Markup {
        html! {
            a id="menu:orbit" {
                span class="icon" {
                    i class="fa-solid fa-globe" {}
                }
                "Orbital projections"
            }
        }
    }
}

impl Render for OrbitReport {
    fn render(&self) -> Markup {
        html! {
            div class="table-container" {
                table class="table is-bordered" {
                    //tr {
                    //    th class="is-info" {
                    //        "Map projection"
                    //    }
                    //    td {
                    //        (self.map_proj.render())
                    //    }
                    //}
                    //tr {
                    //    th class="is-info" {
                    //        "Globe projection"
                    //    }
                    //    td {
                    //        (self.globe_proj.render())
                    //    }
                    //}
                    tr {
                        th class="is-info" {
                            "Sky plot"
                        }
                        td {
                            (self.sky_plot.render())
                        }
                    }
                    @if self.brdc_sp3_err.len() > 0 {
                        @for (constell, page) in self.brdc_sp3_err.iter() {
                            tr {
                                th class="is-info" {
                                    (format!("{} SP3/BRDC", constell))
                                }
                                td {
                                    (page.render())
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
