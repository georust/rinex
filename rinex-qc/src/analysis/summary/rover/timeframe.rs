use crate::{
    context::{meta::MetaData, QcContext},
    plot::{Button, ButtonBuilder, MarkerSymbol, NamedColor, Plot},
    prelude::{html, Markup, Render, Rinex},
};

use std::collections::HashMap;

use rinex::prelude::{Carrier, Constellation, Epoch, SV};

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
    plot: Plot,
}

impl QcTimeFrame {
    pub fn new(
        meta: &MetaData,
        obs_rinex: &Rinex,
        nav_set: &Option<Rinex>,
        constellation: &Constellation,
    ) -> Self {

        let html_id = format!("{:x}-time_frame", constellation);
        let mut plot = Plot::timedomain_plot(html_id, "Time Frame", "", true);


        // determine total amount of signals for this constellation
        let mut signals = Vec::new();
        let mut prn = Vec::new();
        let (mut t_min_obs, mut t_max_obs) = (Epoch::default(), Epoch::default());

        for (nth, (k, v)) in obs_rinex.signals_observation_iter().enumerate() {
            if nth == 0 {
                t_min_obs = k.epoch;
            }
            if v.sv.constellation == constellation {
                if !signals.contains(v.observable) {
                    signals.push(v.observable);
                }
                if !prn.contains(v.sv.prn) {
                    prn.push(v.sv.prn);
                }
            }
            t_max_obs = k.epoch;
        }
        
        let t_range_obs = vec![t_min_obs, t_max_obs];

        let signals_dy = 0.8 / signals.len() as f64;
            
        // draw thick line @ each prn
        for (nth, prn) in prn.iter().enumerate() {

            let curve = timedomain_chart(
                &format!("{:x}-time_frame-prn{:02}", constellation, prn),
                Mode::LineCurves,
                MarkerSymbol::Dash,
                &t_range_obs,
                vec![(prn as f64, prn as f64)],
                nth == 0,
            );
                
            // design signals curve
            for (nth_sig, sig) in signals.iter().enumerate() {
                let mut y_sig = prn as f64 + 0.1;
                y_sig += nth_sig as f64 * signals_dy; 

                for (k, v) in obs_rinex.signals_observation_iter() {
                    if v.sv.constellation == constellation {
                        if v.sv.prn == prn && v.signal == sig {

                        }
                    }
                }
            }

        }

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

        for (meta, rinex) in ctx.obs_dataset.iter() {}

        if let Some(nav) = &nav_set {
            s = s.augment_with_nav_set(&nav);
        }

        s
    }

    /// Customize plot in case we have Navigation data
    fn augment_with_nav_set(&self, nav_set: &Rinex) -> Self {
        let mut s = self.clone();

        // plot the ephemeris range
        // plot updates of ionosphere model
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

pub struct QcTimeFrames {
    pub pages: HashMap<Constellation, QcTimeFrame>,
}

impl QcTimeFrames {
    pub fn new(meta: &MetaData, obs: &Rinex, nav_set: &Rinex) -> Self {
        Self {
            pages: {
                let mut pages = HashMap::new();
                for constellation in obs.constellation() {
                    pages.insert(QcTimeFrame::new(meta, obs, nav_set, constellation));
                }
                pages
            }
        }
    }
}
