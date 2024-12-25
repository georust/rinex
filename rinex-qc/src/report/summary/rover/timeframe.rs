use crate::{
    plot::{Button, ButtonBuilder, MarkerSymbol, NamedColor, Plot},
    prelude::{html, Markup, Render, Rinex},
    context::{QcContext, meta::MetaData},
};

use std::collections::HashMap;

use rinex::prelude::{Carrier, Epoch, SV, Constellation};

struct CurvePoint {
    pub y: f64,
    pub symbol: MarkerSymbol,
    pub color: NamedColor,
}

pub struct CurvesKey {
    pub sv: SV,
    pub carrier: Carrier,
}

struct Curve {
    pub x: Vec<f64>,
    pub y: Vec<CurvePoint>,
}

/// [QcTimeFrame] is a general summary
/// of the overal time frame represented by [QcContext]
pub struct QcTimeFrame {
    /// Context plot, per constellation
    constell_plot: HashMap<Constellation, Plot>,
}

impl QcTimeFrame {
    pub fn new(constellation: &Constellation, ctx: &QcContext, meta: &MetaData, obs_rinex: &Rinex) -> Self {

        // X range (min, max)
        let mut x_range = (Epoch::default(), Epoch::default());

        // Y points range (min, max)
        let mut y_range = (0.0, 0.0);

        let mut curves = HashMap::<CurvesKey, Curve>::new();

        let mut buttons = Vec::<Button>::new();

        // buttons.push(
        //     ButtonBuilder::new()
        //         .name("Epoch")
        //         .label(&label)
        //         .push_restyle(DensityMapbox::<f64, f64, f64>::modify_visible(
        //             (0..nb_of_maps)
        //                 .map(|i| {
        //                     if epoch_index == i {
        //                         Visible::True
        //                     } else {
        //                         Visible::False
        //                     }
        //                 })
        //                 .collect(),
        //         ))
        //         .build(),
        // );

        let mut constell_plot = HashMap::new();

        for (meta, rinex) in ctx.obs_dataset.iter() {

        }


        let mut plot = Plot::timedomain_plot("time_frame_plot", "Time Frame", "", true);

        if ctx.has_navigation_data() {
            //Self::customize_with_nav_data(ctx, &mut plot);
        }

        //plot.add_custom_controls(buttons);

        Self { constell_plot }
    }

    /// Customize plot in case we have Navigation data
    fn customize_with_nav_data(
        ctx: &QcContext,
        x_range: (&mut Epoch, &mut Epoch),
        y_range: (&mut f64, &mut f64),
        plot: &mut Plot,
    ) {
        let nav_data = ctx.nav_dataset.as_ref().unwrap();
    }
}

impl Render for QcTimeFrame {
    fn render(&self) -> Markup {
        html! {
            div class="table-container" {
                table class="table is-bordered" {
                    tbody {
                        tr {
                            th class="is-info" {
                                "Time Frame"
                            }
                            td {
                                (self.plot.render())
                            }
                        }
                    }
                }
            }
        }
    }
}
