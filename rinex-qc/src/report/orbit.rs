use rinex::{navigation::Ephemeris, prelude::GroundPosition};
use sp3::prelude::{Constellation, SP3, SV};

use crate::{
    plot::MapboxStyle,
    prelude::{html, Markup, Plot, QcContext, Render},
};

pub struct OrbitReport {
    map_proj: Plot,
    globe_proj: Plot,
    sky_plot: Plot,
}

impl OrbitReport {
    pub fn new(ctx: &QcContext, reference: Option<GroundPosition>, force_brdc_sky: bool) -> Self {
        let (x0, y0, z0) = reference.unwrap_or_default().to_ecef_wgs84();
        let brdc_skyplot = ctx.has_brdc_navigation() && (!ctx.has_sp3() || force_brdc_sky);

        let max_sv_visible = if brdc_skyplot { 2 } else { 4 };
        Self {
            sky_plot: {
                let mut plot = Plot::sky_plot("skyplot", "Sky plot", true);
                if let Some(sp3) = ctx.sp3() {
                    for (sv_index, sv) in sp3.sv().enumerate() {
                        let t = sp3
                            .sv_position()
                            .filter_map(|(t, svnn, _)| if svnn == sv { Some(t) } else { None })
                            .collect::<Vec<_>>();
                        let rho = sp3
                            .sv_position()
                            .filter_map(|(t, svnn, sv_pos)| {
                                if svnn == sv {
                                    let el = Ephemeris::elevation_azimuth(sv_pos, (x0, y0, z0)).0;
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
                        let trace = Plot::sky_trace(t, rho, theta, sv_index < max_sv_visible)
                            .name(format!("{:X}", sv));
                        plot.add_trace(trace);
                    }
                }
                if brdc_skyplot {}
                plot
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
                map_proj
            },
            globe_proj: {
                let mut map_proj = Plot::world_map(
                    "map_proj",
                    "Map Projection",
                    MapboxStyle::OpenStreetMap,
                    (32.0, -40.0),
                    1,
                    true,
                );
                map_proj
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
                    tr {
                        th class="is-info" {
                            "Globe projection"
                        }
                        td {
                            (self.globe_proj.render())
                        }
                    }
                    tr {
                        th class="is-info" {
                            "Sky plot"
                        }
                        td {
                            (self.sky_plot.render())
                        }
                    }
                }
            }
        }
    }
}
