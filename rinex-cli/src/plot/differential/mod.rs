use rinex::*;
use super::{
    Plot2d,
    build_plot, build_chart,
};
use std::collections::{BTreeMap, HashMap};
use plotters::{
    prelude::*,
    coord::Shift,
    chart::ChartState,
    coord::types::RangedCoordf64,
};

/// Plot Context for differential analysis
pub struct Context<'a> {
    /// Plot area
    pub plot: DrawingArea<BitMapBackend<'a>, Shift>,
    /// Plot chart
    pub chart: ChartState<Plot2d>,
    /// Differential analysis is against time
    pub t_axis: Vec<f64>,
    /// Colors: one per differentiation
    pub colors: HashMap<String, RGBAColor>,
}

impl<'a> Context<'a> {
    /// Builds new plot context
    pub fn new (dims: (u32,u32), rnx: &Rinex) -> Self {
        let plot = build_plot("Differential Analysis", dims);
        let mut t_axis: Vec<f64> = Vec::with_capacity(16384);
        let mut colors: HashMap<String, RGBAColor> = HashMap::new();
        let mut e0: i64 = 0;
        if let Some(record) = rnx.record.as_obs(){
            for (e_index, (e, (_,_))) in record.iter().enumerate() {
                if e_index == 0 {
                    e0 = e.date.timestamp();
                } else {
                    let t = e.date.timestamp() - e0;
                    t_axis.push(t as f64);
                }
            }
        }
        let chart = build_chart("DIFF", t_axis.clone(), (0.0, 0.0), &plot);
        Self {
            plot,
            chart,
            t_axis,
            colors,
        }
    }
}

pub fn plot(ctx: &mut Context, data: BTreeMap<Epoch, HashMap<Sv, HashMap<String, f64>>>) {
    // to differentiate code diffs
    let symbols = vec!["x","t","o"];
    let chart = ctx.chart.restore(&ctx.plot);
}
