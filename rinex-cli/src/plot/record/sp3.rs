use crate::plot::{build_chart_epoch_axis, generate_markers, PlotContext};
use plotly::common::{Marker, MarkerSymbol, Mode, Visible};
use rinex::prelude::Epoch;
use rinex::quality::QcContext;
/*
 * Plots High Precision Orbit and Clock data
 * provided in the form of an SP3 file
 */
pub fn plot_sp3(ctx: &QcContext, plot_context: &mut PlotContext) {
    let sp3 = ctx.sp3_data().unwrap(); // cannot fail at this point
                                       /*
                                        * Plot SV Position
                                        *  [+] Design 1 color per SV
                                        */
    plot_context.add_cartesian2d_plot("High Precision Orbit (SP3)", "SV X Position (km)");
    for (sv_index, sv) in sp3.sv().enumerate() {
        let data_x: Vec<Epoch> = sp3
            .sv_position()
            .filter_map(
                |(epoch, svnn, (x, _, _))| {
                    if svnn == sv {
                        Some(epoch)
                    } else {
                        None
                    }
                },
            )
            .collect();
        let data_y: Vec<f64> = sp3
            .sv_position()
            .filter_map(
                |(epoch, svnn, (x, _, _))| {
                    if svnn == sv {
                        Some(x)
                    } else {
                        None
                    }
                },
            )
            .collect();
        let trace = build_chart_epoch_axis(&format!("{}", sv), Mode::LinesMarkers, data_x, data_y)
            .web_gl_mode(true)
            .visible({
                if sv_index < 4 {
                    Visible::True
                } else {
                    Visible::LegendOnly
                }
            });
        plot_context.add_trace(trace);
    }
    plot_context.add_cartesian2d_plot("High Precision Orbit (SP3)", "SV Y Position (km)");
    for (sv_index, sv) in sp3.sv().enumerate() {
        let data_x: Vec<Epoch> = sp3
            .sv_position()
            .filter_map(
                |(epoch, svnn, _)| {
                    if svnn == sv {
                        Some(epoch)
                    } else {
                        None
                    }
                },
            )
            .collect();
        let data_y: Vec<f64> = sp3
            .sv_position()
            .filter_map(
                |(epoch, svnn, (_, y, _))| {
                    if svnn == sv {
                        Some(y)
                    } else {
                        None
                    }
                },
            )
            .collect();
        let trace = build_chart_epoch_axis(&format!("{}", sv), Mode::LinesMarkers, data_x, data_y)
            .web_gl_mode(true)
            .visible({
                if sv_index < 4 {
                    Visible::True
                } else {
                    Visible::LegendOnly
                }
            });
        plot_context.add_trace(trace);
    }
    plot_context.add_cartesian2d_plot("High Precision Orbit (SP3)", "SV Z Position (km)");
    for (sv_index, sv) in sp3.sv().enumerate() {
        let data_x: Vec<Epoch> = sp3
            .sv_position()
            .filter_map(
                |(epoch, svnn, _)| {
                    if svnn == sv {
                        Some(epoch)
                    } else {
                        None
                    }
                },
            )
            .collect();
        let data_y: Vec<f64> = sp3
            .sv_position()
            .filter_map(
                |(epoch, svnn, (_, _, z))| {
                    if svnn == sv {
                        Some(z)
                    } else {
                        None
                    }
                },
            )
            .collect();
        let trace = build_chart_epoch_axis(&format!("{}", sv), Mode::LinesMarkers, data_x, data_y)
            .web_gl_mode(true)
            .visible({
                if sv_index < 4 {
                    Visible::True
                } else {
                    Visible::LegendOnly
                }
            });
        plot_context.add_trace(trace);
    }
    trace!("sp3 orbit visualization");
}
