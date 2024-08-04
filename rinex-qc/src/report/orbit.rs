use rinex::{
    navigation::Ephemeris,
    prelude::{GroundPosition, Orbit, Rinex},
};
use sp3::prelude::{Constellation, Epoch, SP3, SV};
use std::collections::HashMap;

use qc_traits::processing::{Filter, Preprocessing};

use crate::{
    plot::{MapboxStyle, MarkerSymbol, Mode},
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
                let (err_x_m, err_y_m, err_z_m) = (
                    (brdc_x - pos_sp3.0) * 1000.0,
                    (brdc_y - pos_sp3.1) * 1000.0,
                    (brdc_z - pos_sp3.2) * 1000.0,
                );
                if let Some(t) = t.get_mut(&sv_sp3) {
                    t.push(t_sp3);
                } else {
                    t.insert(sv_sp3, vec![t_sp3]);
                }
                if let Some(x) = error_x.get_mut(&sv_sp3) {
                    x.push(err_x_m);
                } else {
                    error_x.insert(sv_sp3, vec![err_x_m]);
                }
                if let Some(y) = error_y.get_mut(&sv_sp3) {
                    y.push(err_y_m);
                } else {
                    error_y.insert(sv_sp3, vec![err_y_m]);
                }
                if let Some(z) = error_z.get_mut(&sv_sp3) {
                    z.push(err_z_m);
                } else {
                    error_z.insert(sv_sp3, vec![err_z_m]);
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
        let (x0_km, y0_km, z0_km) = (x0 / 1000.0, y0 / 1000.0, z0 / 1000.0);
        // TODO: brdc needs a timeserie..
        let brdc_skyplot = ctx.has_brdc_navigation() && ctx.has_sp3() && force_brdc_sky;

        let max_sv_visible = if brdc_skyplot { 2 } else { 4 };

        let mut t_sp3 = HashMap::<SV, Vec<Epoch>>::new();
        let mut elev_sp3 = HashMap::<SV, Vec<f64>>::new();
        let mut azim_sp3 = HashMap::<SV, Vec<f64>>::new();

        let mut t_brdc = HashMap::<SV, Vec<Epoch>>::new();
        let mut elev_brdc = HashMap::<SV, Vec<f64>>::new();
        let mut azim_brdc = HashMap::<SV, Vec<f64>>::new();

        // calc evelation°
        if let Some(sp3) = ctx.sp3() {
            for (t, sv_sp3, pos_sp3) in sp3.sv_position() {
                let (x_sp3_km, y_sp3_km, z_sp3_km) = (pos_sp3.0, pos_sp3.1, pos_sp3.2);
                if let Ok(el_az_range) = Ephemeris::elevation_azimuth_range(
                    t,
                    &ctx.almanac,
                    ctx.earth_cef,
                    (x_sp3_km, y_sp3_km, z_sp3_km),
                    (x0_km, y0_km, z0_km),
                ) {
                    if let Some(t_sp3) = t_sp3.get_mut(&sv_sp3) {
                        t_sp3.push(t);
                    } else {
                        t_sp3.insert(sv_sp3, vec![t]);
                    }
                    if let Some(e) = elev_sp3.get_mut(&sv_sp3) {
                        e.push(el_az_range.elevation_deg);
                    } else {
                        elev_sp3.insert(sv_sp3, vec![el_az_range.elevation_deg]);
                    }
                    if let Some(a) = azim_sp3.get_mut(&sv_sp3) {
                        a.push(el_az_range.azimuth_deg);
                    } else {
                        azim_sp3.insert(sv_sp3, vec![el_az_range.azimuth_deg]);
                    }
                }
                if brdc_skyplot {
                    let brdc = ctx.brdc_navigation().unwrap();
                    if let Some((x_km, y_km, z_km)) = brdc.sv_position(sv_sp3, t) {
                        if let Ok(el_az_range) = Ephemeris::elevation_azimuth_range(
                            t,
                            &ctx.almanac,
                            ctx.earth_cef,
                            (x_km, y_km, z_km),
                            (x0_km, y0_km, z0_km),
                        ) {
                            if let Some(t_brdc) = t_brdc.get_mut(&sv_sp3) {
                                t_brdc.push(t);
                            } else {
                                t_brdc.insert(sv_sp3, vec![t]);
                            }
                            if let Some(e) = elev_brdc.get_mut(&sv_sp3) {
                                e.push(el_az_range.elevation_deg);
                            } else {
                                elev_brdc.insert(sv_sp3, vec![el_az_range.elevation_deg]);
                            }
                            if let Some(a) = azim_brdc.get_mut(&sv_sp3) {
                                a.push(el_az_range.azimuth_deg);
                            } else {
                                azim_brdc.insert(sv_sp3, vec![el_az_range.azimuth_deg]);
                            }
                        }
                    }
                }
            }
        }

        Self {
            sky_plot: {
                let mut plot = Plot::sky_plot("skyplot", "Sky plot", true);
                for (sv_index, (sv, epochs)) in t_sp3.iter().enumerate() {
                    let visible = sv_index < max_sv_visible;
                    let elev_sp3 = elev_sp3.get(&sv).unwrap();
                    let azim_sp3 = azim_sp3.get(&sv).unwrap();
                    let trace = Plot::sky_trace(
                        &sv.to_string(),
                        epochs,
                        elev_sp3.to_vec(),
                        azim_sp3.to_vec(),
                        visible,
                    );
                    plot.add_trace(trace);
                }
                for (sv_index, (sv, epochs)) in t_brdc.iter().enumerate() {
                    let visible = sv_index < max_sv_visible;
                    let elev_brdc = elev_brdc.get(&sv).unwrap();
                    let azim_brdc = azim_brdc.get(&sv).unwrap();
                    let trace = Plot::sky_trace(
                        &format!("{}_brdc", sv),
                        epochs,
                        elev_brdc.to_vec(),
                        azim_brdc.to_vec(),
                        visible,
                    );
                    plot.add_trace(trace);
                }
                plot
            },
            elev_plot: {
                let mut elev_plot =
                    Plot::timedomain_plot("elev_plot", "Elevation", "Elevation [deg°]", true);
                for (sv_index, (sv, epochs)) in t_sp3.iter().enumerate() {
                    let elev = elev_sp3.get(&sv).unwrap();
                    let trace = Plot::timedomain_chart(
                        &sv.to_string(),
                        Mode::Markers,
                        MarkerSymbol::Diamond,
                        epochs,
                        elev.to_vec(),
                        sv_index < max_sv_visible,
                    );
                    elev_plot.add_trace(trace);
                }
                for (sv_index, (sv, epochs)) in t_brdc.iter().enumerate() {
                    let elev = elev_brdc.get(&sv).unwrap();
                    let trace = Plot::timedomain_chart(
                        &format!("{}_brdc", sv),
                        Mode::Markers,
                        MarkerSymbol::Diamond,
                        epochs,
                        elev.to_vec(),
                        sv_index < max_sv_visible,
                    );
                    elev_plot.add_trace(trace);
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
                        let orbits = sp3
                            .sv_position()
                            .filter_map(|(t, svnn, pos_km)| {
                                if svnn == sv {
                                    Some(Orbit::from_position(
                                        pos_km.0,
                                        pos_km.1,
                                        pos_km.2,
                                        t,
                                        ctx.earth_cef,
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
