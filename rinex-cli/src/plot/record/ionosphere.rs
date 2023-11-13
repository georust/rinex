use crate::plot::{build_chart_epoch_axis, PlotContext};
use hifitime::{Epoch, TimeScale};
use plotly::common::Mode;
use rinex::navigation::{IonMessage, KbModel};
use rinex::prelude::{Constellation, GroundPosition, RnxContext, SV};
use std::f64::consts::PI;

fn klob_ionospheric_delay(
    t: Epoch,
    sv: SV,
    model: KbModel,
    elev_azim_deg: (f64, f64),
    lat_lon_ddeg: (f64, f64),
) -> f64 {
    let t = t.to_duration_in_time_scale(TimeScale::GPST).to_seconds();

    let alpha = model.alpha;
    let beta = model.beta;
    let re = 6371.0E3_f64;
    let phi_p = map_3d::deg2rad(78.3);
    let lambda_p = map_3d::deg2rad(291.0);
    let phi_u = map_3d::deg2rad(lat_lon_ddeg.0);
    let lambda_u = map_3d::deg2rad(lat_lon_ddeg.1);
    let e = map_3d::deg2rad(elev_azim_deg.0);
    let a = map_3d::deg2rad(elev_azim_deg.1);

    let h = match sv.constellation {
        Constellation::BeiDou => 375.0E3,
        _ => 350.0E3,
    };

    let psi = PI / 2.0_f64 - e - (re * e.cos() / (re + h)).asin();

    let phi_ipp = phi_u.sin() * psi.cos() + phi_u.cos() * psi.sin() * a.cos();
    let lambda_i = lambda_u + psi * a.sin() / phi_ipp.cos();
    let phi_m =
        phi_ipp.sin() * phi_p.sin() + phi_ipp.cos() * phi_p.cos() * (lambda_i - lambda_p).cos();

    let mut t_ipp = 43.200E3 * lambda_i / PI + t;
    if t_ipp > 86400.0 {
        t_ipp -= 86400.0;
    } else if t_ipp < 0.0 {
        t_ipp += 86400.0;
    }

    let mut a_i = alpha.0
        + alpha.1 * (phi_m / PI).powi(1)
        + alpha.2 * (phi_m / PI).powi(2)
        + alpha.3 * (phi_m / PI).powi(3);

    if a_i < 0.0 {
        a_i = 0.0;
    }

    let mut p_i = beta.0
        + beta.1 * (phi_m / PI).powi(1)
        + beta.2 * (phi_m / PI).powi(2)
        + beta.3 * (phi_m / PI).powi(3);

    if p_i < 72000.0 {
        p_i = 72000.0;
    }

    let x_i = 2.0_f64 * PI * (t_ipp - 50.400E3) / p_i;

    let f = 1.0 / (1.0 - (re * e.cos() / (re + h)).powi(2)).sqrt();

    if x_i.abs() < PI / 2.0 {
        (5.0E-9 + a_i * x_i.cos()) * f
    } else {
        5.0E-9 * f
    }
}

pub fn plot_ionospheric_delay(ctx: &RnxContext, plot_ctx: &mut PlotContext) {
    let ref_pos = ctx.ground_position().unwrap_or(GroundPosition::default());

    let ref_geo = ref_pos.to_geodetic();
    let lat_lon_ddeg = (ref_geo.0, ref_geo.1);

    // if let Some(nav) = ctx.nav_data() {
    //     let mut kb_delay: Vec<(Epoch, f64)> = Vec::new();
    //     for (_index, (t, svnn)) in nav.sv_epoch().enumerate() {
    //         for sv in svnn {
    //             if let Some(t, (_, _, model)) = nav.ionosphere_models(t) {
    //                 match model {
    //                     IonMessage::KlobucharModel(model) => {
    //                         let sv_elev_azim = nav
    //                             .sv_elevation_azimuth(Some(ref_pos))
    //                             .find(|(epoch, svnn, _)| *epoch == t && *svnn == sv);
    //                         if let Some(elev_azim) = sv_elev_azim {
    //                             kb_delay.push((
    //                                 t,
    //                                 klob_ionospheric_delay(t, sv, model, elev_azim.2, lat_lon_ddeg),
    //                             ));
    //                         }
    //                     },
    //                     _ => {},
    //                 }
    //             }
    //         }
    //     }
    //     if !kb_delay.is_empty() {
    //         trace!("klobuchar ionospheric model");
    //         plot_ctx.add_cartesian2d_plot("Ionospheric Delay", "Delay [s]");
    //         let trace = build_chart_epoch_axis(
    //             "kb",
    //             Mode::LinesMarkers,
    //             kb_delay.iter().map(|(t, _)| *t).collect(),
    //             kb_delay.iter().map(|(_, dly)| *dly).collect(),
    //         );
    //         plot_ctx.add_trace(trace);
    //     }
    // }
}
