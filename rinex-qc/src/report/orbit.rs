use rinex::{
    navigation::Ephemeris,
    prelude::{GroundPosition, Orbit, Rinex},
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
                        true,
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
                        true,
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
                        true,
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
    elev_plot: Plot,
    map_proj: Plot,
    // globe_proj: Plot,
    brdc_sp3_err: HashMap<Constellation, BrdcSp3Report>,
}

impl OrbitReport {
    pub fn new(ctx: &QcContext, reference: Option<GroundPosition>, force_brdc_sky: bool) -> Self {
        let (x0, y0, z0) = reference.unwrap_or_default().to_ecef_wgs84();
        let brdc_skyplot = ctx.has_brdc_navigation() && ctx.has_sp3() && force_brdc_sky;

        let max_sv_visible = if brdc_skyplot { 2 } else { 4 };

        let mut t = HashMap::<SV, Vec<Epoch>>::new();
        let mut elev = HashMap::<SV, Vec<f64>>::new();
        let mut azim = HashMap::<SV, Vec<f64>>::new();

        // calc evelation°
        if let Some(sp3) = ctx.sp3() {
            for (t_sp3, sv_sp3, pos_sp3) in sp3.sv_position() {
                let (x_sp3_m, y_sp3_m, z_sp3_m) =
                    (pos_sp3.0 * 1000.0, pos_sp3.1 * 1000.0, pos_sp3.2 * 1000.0);
                let (el, az) = Ephemeris::elevation_azimuth(
                    t_sp3,
                    &ctx.almanac,
                    (x_sp3_m, y_sp3_m, z_sp3_m),
                    (x0, y0, z0),
                );
                if let Some(t) = t.get_mut(&sv_sp3) {
                    t.push(t_sp3);
                } else {
                    t.insert(sv_sp3, vec![t_sp3]);
                }
                if let Some(e) = elev.get_mut(&sv_sp3) {
                    e.push(el);
                } else {
                    elev.insert(sv_sp3, vec![el]);
                }
                if let Some(a) = azim.get_mut(&sv_sp3) {
                    a.push(az);
                } else {
                    azim.insert(sv_sp3, vec![az]);
                }
            }
        }

        Self {
            sky_plot: {
                let mut plot = Plot::sky_plot("skyplot", "Sky plot", true);
                if let Some(sp3) = ctx.sp3() {
                    for (sv_index, sv) in sp3.sv().enumerate() {
                        let visible = sv_index < max_sv_visible;
                        if let Some(t) = t.get(&sv) {
                            let elev = elev.get(&sv).unwrap();
                            let azim = azim.get(&sv).unwrap();
                            let trace = Plot::sky_trace(
                                &sv.to_string(),
                                t.to_vec(),
                                elev.to_vec(),
                                azim.to_vec(),
                                visible,
                            );
                            plot.add_trace(trace);
                        }
                    }
                }
                plot
            },
            elev_plot: {
                let mut elev_plot =
                    Plot::timedomain_plot("elev_plot", "Elevation", "Elevation [deg°]", true);
                if let Some(sp3) = ctx.sp3() {
                    for (sv_index, sv) in sp3.sv().enumerate() {
                        if let Some(t) = t.get(&sv) {
                            let elev = elev.get(&sv).unwrap();
                            let trace = Plot::timedomain_chart(
                                &sv.to_string(),
                                Mode::Markers,
                                MarkerSymbol::Diamond,
                                &t,
                                elev.to_vec(),
                                sv_index < max_sv_visible,
                            );
                            elev_plot.add_trace(trace);
                        }
                    }
                }
                elev_plot
            },
            map_proj: {
                let mut map_proj = Plot::world_map(
                    "map_proj",
                    "Map Projection",
                    MapboxStyle::OpenStreetMap,
                    (32.0, -40.0),
                    1,
                    true,
                );
                if let Some(sp3) = ctx.sp3() {
                    for (sv_index, sv) in sp3.sv().enumerate() {
                        let t = sp3
                            .sv_position()
                            .filter_map(|(t, svnn, _)| if svnn == sv { Some(t) } else { None })
                            .collect::<Vec<_>>();
                        let orbits = sp3
                            .sv_position()
                            .filter_map(|(t, svnn, pos_km)| {
                                if svnn == sv {
                                    Some(Orbit::from_position(
                                        pos_km.0,
                                        pos_km.1,
                                        pos_km.2,
                                        t,
                                        ctx.earth_iau_ecef,
                                    ))
                                } else {
                                    None
                                }
                            })
                            .collect::<Vec<_>>();

                        let lat_ddeg = orbits
                            .iter()
                            .filter_map(|orb| {
                                if let Ok(lat) = orb.latitude_deg() {
                                    Some(lat)
                                } else {
                                    None
                                }
                            })
                            .collect::<Vec<_>>();

                        let long_ddeg = orbits
                            .iter()
                            .map(|orb| orb.longitude_deg())
                            .collect::<Vec<_>>();

                        let map = Plot::mapbox(
                            lat_ddeg,
                            long_ddeg,
                            &sv.to_string(),
                            MarkerSymbol::Circle,
                            None,
                            1.0,
                            sv_index < 5,
                        );
                        map_proj.add_trace(map);
                    }
                }
                map_proj
            },
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
                                reports.insert(
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
                    tr {
                        th class="is-info" {
                            "Map projection"
                        }
                        td {
                            (self.map_proj.render())
                        }
                    }
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
                    tr {
                        th class="is-info" {
                            "Elevation"
                        }
                        td {
                            (self.elev_plot.render())
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
