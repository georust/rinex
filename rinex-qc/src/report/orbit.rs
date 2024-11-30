use rinex::{
    navigation::Ephemeris,
    prelude::{nav::Orbit, Constellation, Epoch, GroundPosition, Rinex, SV},
};
use std::collections::{BTreeMap, HashMap};

use qc_traits::{Filter, Preprocessing};

use crate::{
    plot::{MapboxStyle, MarkerSymbol, Mode},
    prelude::{html, Markup, Plot, QcContext, Render},
};

#[cfg(feature = "sp3")]
use sp3::prelude::SP3;

#[cfg(feature = "sp3")]
struct BrdcSp3Report {
    x_err_plot: Plot,
    y_err_plot: Plot,
    z_err_plot: Plot,
}

#[cfg(feature = "sp3")]
impl BrdcSp3Report {
    fn new(sp3: &SP3, brdc: &Rinex) -> Self {
        let mut errors = BTreeMap::<SV, Vec<(Epoch, f64, f64, f64)>>::new();
        for (t_sp3, sv_sp3, (sp3_x, sp3_y, sp3_z)) in sp3.sv_position() {
            if let Some(brdc_orb) = brdc.sv_orbit(sv_sp3, t_sp3) {
                let brdc_state = brdc_orb.to_cartesian_pos_vel();
                let (brdc_x, brdc_y, brdc_z) = (brdc_state[0], brdc_state[1], brdc_state[2]);
                let (err_x_m, err_y_m, err_z_m) = (
                    (brdc_x - sp3_x) * 1000.0,
                    (brdc_y - sp3_y) * 1000.0,
                    (brdc_z - sp3_z) * 1000.0,
                );
                if let Some(errors) = errors.get_mut(&sv_sp3) {
                    errors.push((t_sp3, err_x_m, err_y_m, err_z_m));
                } else {
                    errors.insert(sv_sp3, vec![(t_sp3, err_x_m, err_y_m, err_z_m)]);
                }
            }
        }
        Self {
            x_err_plot: {
                let mut plot = Plot::timedomain_plot(
                    "sp3_brdc_x_err",
                    "(BRDC - SP3) Position Errors",
                    "Error [m]",
                    true,
                );
                for (sv_index, (sv, errors)) in errors.iter().enumerate() {
                    let error_t = errors.iter().map(|(t, _, _, _)| *t).collect::<Vec<_>>();
                    let error_x = errors.iter().map(|(_, x, _, _)| *x).collect::<Vec<_>>();
                    let trace = Plot::timedomain_chart(
                        &sv.to_string(),
                        Mode::Markers,
                        MarkerSymbol::Diamond,
                        &error_t,
                        error_x,
                        sv_index < 4,
                    );
                    plot.add_trace(trace);
                }
                plot
            },
            y_err_plot: {
                let mut plot = Plot::timedomain_plot(
                    "sp3_brdc_y_err",
                    "(BRDC - SP3) Position Errors",
                    "Error [m]",
                    true,
                );
                for (sv_index, (sv, errors)) in errors.iter().enumerate() {
                    let error_t = errors.iter().map(|(t, _, _, _)| *t).collect::<Vec<_>>();
                    let error_y = errors.iter().map(|(_, _, y, _)| *y).collect::<Vec<_>>();
                    let trace = Plot::timedomain_chart(
                        &sv.to_string(),
                        Mode::Markers,
                        MarkerSymbol::Diamond,
                        &error_t,
                        error_y,
                        sv_index < 4,
                    );
                    plot.add_trace(trace);
                }
                plot
            },
            z_err_plot: {
                let mut plot = Plot::timedomain_plot(
                    "sp3_brdc_z_err",
                    "(BRDC - SP3) Position Errors",
                    "Error [m]",
                    true,
                );
                for (sv_index, (sv, errors)) in errors.iter().enumerate() {
                    let error_t = errors.iter().map(|(t, _, _, _)| *t).collect::<Vec<_>>();
                    let error_z = errors.iter().map(|(_, _, _, z)| *z).collect::<Vec<_>>();
                    let trace = Plot::timedomain_chart(
                        &sv.to_string(),
                        Mode::Markers,
                        MarkerSymbol::Diamond,
                        &error_t,
                        error_z,
                        sv_index < 4,
                    );
                    plot.add_trace(trace);
                }
                plot
            },
        }
    }
}

#[cfg(feature = "sp3")]
impl Render for BrdcSp3Report {
    fn render(&self) -> Markup {
        html! {
            div class="table-container" {
                table class="table is-bordered" {
                    tr {
                        th class="is-info" {
                            "X errors"
                        }
                        td {
                            (self.x_err_plot.render())
                        }
                    }
                    tr {
                        th class="is-info" {
                            "Y errors"
                        }
                        td {
                            (self.y_err_plot.render())
                        }
                    }
                    tr {
                        th class="is-info" {
                            "Z errors"
                        }
                        td {
                            (self.z_err_plot.render())
                        }
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
    #[cfg(feature = "sp3")]
    brdc_sp3_err: HashMap<Constellation, BrdcSp3Report>,
}

impl OrbitReport {
    pub fn new(ctx: &QcContext, reference: Option<GroundPosition>, force_brdc_sky: bool) -> Self {
        let (x0, y0, z0) = reference.unwrap_or_default().to_ecef_wgs84();
        let (x0_km, y0_km, z0_km) = (x0 / 1000.0, y0 / 1000.0, z0 / 1000.0);

        // TODO: brdc needs a timeserie..
        #[cfg(feature = "sp3")]
        let brdc_skyplot = ctx.has_brdc_navigation() && ctx.has_sp3() && force_brdc_sky;
        #[cfg(not(feature = "sp3"))]
        let brdc_skyplot = ctx.has_brdc_navigation();

        let max_sv_visible = if brdc_skyplot { 2 } else { 4 };

        let mut t_sp3 = BTreeMap::<SV, Vec<Epoch>>::new();
        let mut elev_sp3 = BTreeMap::<SV, Vec<f64>>::new();
        let mut azim_sp3 = BTreeMap::<SV, Vec<f64>>::new();

        let mut t_brdc = BTreeMap::<SV, Vec<Epoch>>::new();
        let mut elev_brdc = BTreeMap::<SV, Vec<f64>>::new();
        let mut azim_brdc = BTreeMap::<SV, Vec<f64>>::new();

        #[cfg(feature = "sp3")]
        if let Some(sp3) = ctx.sp3_data() {
            for (t, sv_sp3, pos_sp3) in sp3.sv_position() {
                let rx_orbit = Orbit::from_position(x0_km, y0_km, z0_km, t, ctx.earth_cef);

                let (x_sp3_km, y_sp3_km, z_sp3_km) = (pos_sp3.0, pos_sp3.1, pos_sp3.2);
                if let Ok(el_az_range) = Ephemeris::elevation_azimuth_range(
                    t,
                    &ctx.almanac,
                    ctx.earth_cef,
                    (x_sp3_km, y_sp3_km, z_sp3_km),
                    (x0_km, y0_km, z0_km),
                ) {
                    let (el_deg, az_deg) = (el_az_range.elevation_deg, el_az_range.azimuth_deg);
                    if el_deg < 0.0 {
                        continue;
                    }
                    if let Some(t_sp3) = t_sp3.get_mut(&sv_sp3) {
                        t_sp3.push(t);
                    } else {
                        t_sp3.insert(sv_sp3, vec![t]);
                    }
                    if let Some(e) = elev_sp3.get_mut(&sv_sp3) {
                        e.push(el_deg);
                    } else {
                        elev_sp3.insert(sv_sp3, vec![el_deg]);
                    }
                    if let Some(a) = azim_sp3.get_mut(&sv_sp3) {
                        a.push(az_deg);
                    } else {
                        azim_sp3.insert(sv_sp3, vec![az_deg]);
                    }
                    if brdc_skyplot {
                        let brdc = ctx.brdc_navigation_data().unwrap();
                        if let Some(el_az_range) =
                            brdc.sv_azimuth_elevation_range(sv_sp3, t, rx_orbit, &ctx.almanac)
                        {
                            let (el_deg, az_deg) =
                                (el_az_range.elevation_deg, el_az_range.azimuth_deg);
                            if let Some(t_brdc) = t_brdc.get_mut(&sv_sp3) {
                                t_brdc.push(t);
                            } else {
                                t_brdc.insert(sv_sp3, vec![t]);
                            }
                            if let Some(e) = elev_brdc.get_mut(&sv_sp3) {
                                e.push(el_deg);
                            } else {
                                elev_brdc.insert(sv_sp3, vec![el_deg]);
                            }
                            if let Some(a) = azim_brdc.get_mut(&sv_sp3) {
                                a.push(az_deg);
                            } else {
                                azim_brdc.insert(sv_sp3, vec![az_deg]);
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
                    if let Some(elev_brdc) = elev_brdc.get(&sv) {
                        let t_brdc = t_brdc.get(&sv).unwrap();
                        let azim_brdc = azim_brdc.get(&sv).unwrap();
                        let trace = Plot::sky_trace(
                            &format!("{}_brdc", sv),
                            t_brdc,
                            elev_brdc.to_vec(),
                            azim_brdc.to_vec(),
                            sv_index < max_sv_visible,
                        );
                        plot.add_trace(trace);
                    }
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
                    if let Some(elev_brdc) = elev_brdc.get(&sv) {
                        let t_brdc = t_brdc.get(&sv).unwrap();
                        let trace = Plot::timedomain_chart(
                            &format!("{}_brdc", sv),
                            Mode::Markers,
                            MarkerSymbol::Diamond,
                            t_brdc,
                            elev_brdc.to_vec(),
                            sv_index < max_sv_visible,
                        );
                        elev_plot.add_trace(trace);
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
                #[cfg(feature = "sp3")]
                if let Some(sp3) = ctx.sp3_data() {
                    for (_sv_index, sv) in sp3.sv().enumerate() {
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
                            5,
                            MarkerSymbol::Circle,
                            None,
                            1.0,
                            true,
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
            #[cfg(feature = "sp3")]
            brdc_sp3_err: {
                let mut reports = HashMap::<Constellation, BrdcSp3Report>::new();
                if let Some(sp3) = ctx.sp3_data() {
                    if let Some(nav) = ctx.brdc_navigation_data() {
                        for constellation in sp3.constellation() {
                            if let Some(constellation) = nav
                                .constellations_iter()
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

#[cfg(feature = "sp3")]
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

#[cfg(not(feature = "sp3"))]
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
                }
            }
        }
    }
}
