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

pub fn plot(dims: (u32,u32), data: &BTreeMap<Epoch, HashMap<Sv, HashMap<String, f64>>>) {
    let mut t0: i64 = 0;
    let mut taxis: Vec<f64> = Vec::new();
    let mut y_range: (f64, f64) = (0.0, 0.0);
    // one color per code diff
    let mut colors: HashMap<String, RGBAColor> = HashMap::new();
    // one symbol per vehicule
    let symbols = vec!["x","t","o"];
    // data to plot
    //  sorted by Diff op
    let mut toplot: HashMap<String, HashMap<Sv, Vec<(f64,f64)>>> = HashMap::new();
    // build a plot
    let p = build_plot("PhaseCode.png", dims);
    // determine all requirements
    for (e_index, (epoch, vehicules)) in data.iter().enumerate() {
        // xp
        if e_index == 0 {
            t0 = epoch.date.timestamp() ;
        }
        let t = (epoch.date.timestamp() - t0) as f64;
        taxis.push(t);

        for (sv, codes) in vehicules.iter() {
            for (c_index, (code, data)) in codes.iter().enumerate() {
                // one color per code 
                if colors.get(&code.to_string()).is_none() {
                    colors.insert(code.to_string(),
                        Palette99::pick(c_index) // RGB
                            .mix(0.99)); // RGBA
                }
                if data < &y_range.0 {
                    y_range.0 = *data;
                }
                if data > &y_range.1 {
                    y_range.1 = *data;
                }
                if let Some(data) = toplot.get_mut(&code.to_string()) {
                    if let Some(data) = data.get_mut(&sv) {
                        data.push((t, t));
                    } else {
                        data.insert(*sv, vec![(t, t)]);
                    }
                } else {
                    let mut map: HashMap<Sv, Vec<(f64,f64)>> = HashMap::new();
                    map.insert(*sv, vec![(t, t)]);
                    toplot.insert(code.to_string(), map);
                }
            }
        }
    }
    let x_axis = taxis[0]..taxis[taxis.len()-1]; 
    let mut chart = ChartBuilder::on(&p)
        .caption("Phase Diff", ("sans-serif", 50).into_font())
        .margin(40)
        .x_label_area_size(30)
        .y_label_area_size(40)
        .build_cartesian_2d(x_axis, 0.9*y_range.0..1.1*y_range.1) // nicer Y scale
        .unwrap();
    chart
        .configure_mesh()
        .x_desc("Timestamp [s]")
        .x_labels(30)
        .y_desc("Code Diff [n.a]")
        .y_labels(30)
        .draw()
        .unwrap();
    /*
     * Plot Data
     */
    for (c_index, (code, sv)) in toplot.iter().enumerate() {
        let color = colors.get(code).unwrap();
        for (sv, data) in sv {
            if c_index == 0 {
                chart
                    .draw_series(
                        data.iter()
                            .map(|point| {
                                Cross::new(*point, 4,
                                    Into::<ShapeStyle>::into(&color).filled())
                                .into_dyn()
                            }))
                            .expect(&format!("failed to draw {} for Sv {:?}", code, sv))
                            .label(code.to_string())
                            .legend(|(x, y)| {
                                PathElement::new(vec![(x, y), (x+20, y)], color.clone())
                            });
            } else {
                chart
                    .draw_series(
                        data.iter()
                            .map(|point| {
                                Cross::new(*point, 4,
                                    Into::<ShapeStyle>::into(&color).filled())
                                .into_dyn()
                            }))
                            .expect(&format!("failed to draw {} for Sv {:?}", code, sv));
            }
        }
    }
    chart
        .configure_series_labels()
        .border_style(&BLACK)
        .background_style(WHITE.filled())
        .draw()
        .expect("failed to plot phase diff analysis");
}
