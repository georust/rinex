//use hifitime::{Epoch, TimeScale};
//use plotly::common::{Marker, MarkerSymbol, Mode, Visible};
use crate::plot::PlotContext;

//use rinex::carrier::Carrier;
//use rinex::navigation::KbModel;
use rinex::prelude::RnxContext;

pub fn plot_ionospheric_delay(_ctx: &RnxContext, _plot_ctx: &mut PlotContext) {
    //let ref_pos = ctx.ground_position().unwrap_or_default();

    //let ref_geo = ref_pos.to_geodetic();
    //let lat_lon_ddeg = (ref_geo.0, ref_geo.1);
    //let ref_ecef_wgs84 = ref_pos.to_ecef_wgs84();

    //if let Some(obs) = ctx.obs_data() {
    //     if let Some(nav) = ctx.nav_data() {
    //        for (sv_index, sv) in obs.sv().enumerate() {
    //            if sv_index == 0 {
    //                plot_ctx.add_timedomain_plot("Ionospheric Delay", "meters of delay");
    //                trace!("ionod corr plot");
    //            }
    //            let codes = obs.observable()
    //                .filter(|obs| obs.is_pseudorange_observable())
    //                .collect::<Vec<_>>();
    //            /*
    //             * Plot the ionod corr for each code measurement, at every Epoch
    //             */
    //            for (code_index, code) in codes.iter().enumerate() {
    //                let x = obs.pseudo_range()
    //                    .filter_map(|((t, t_flag), svnn, observable, _)| {
    //                        if t_flag.is_ok() && svnn == sv && &observable == code {
    //                            Some(t)
    //                        } else {
    //                            None
    //                        }
    //                    })
    //                    .collect::<Vec<_>>();
    //
    //                let y = x.iter().filter_map(|t| {
    //                    let (elev, azim) =
    //                    let (lat, lon) = lat_lon_ddeg;
    //                    let freq = Carrier::from_observable(
    //                        sv.constellation,
    //                        code).ok()?;
    //                    let ionod_corr = nav.ionod_correction(*t, elev, azim, lat, lon, freq)?;
    //                    Some(ionod_corr)
    //                }).collect::<Vec<_>>();
    //                let chart = build_chart_epoch_axis(
    //                    &format!("{:X}({})", sv, code),
    //                    Mode::Markers,
    //                    x,
    //                    y
    //                )
    //                .visible({
    //                    if sv_index < 2 && code_index < 2 {
    //                        Visible::True
    //                    } else {
    //                        Visible::LegendOnly
    //                    }
    //                });
    //                plot_ctx.add_trace(chart);
    //            }
    //         }
    //     }
    //}
}
