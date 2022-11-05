use rinex::*;
use super::{
    Plot2d,
    build_plot,
};
use plotters::prelude::*;
use std::collections::{BTreeMap, HashMap};

pub fn plot(dims: (u32,u32), data: &HashMap<String, BTreeMap<Epoch, f64>>) {
    let mut p = build_plot("phase-diff.png", dims);
    println!("{:#?}", data);
    // build a color map, one per Op
    let mut cmap: HashMap<String, RGBAColor> = HashMap::new();
    // determine (smallest, largest) ts accross all Ops
    // determine (smallest, largest) y accross all Ops (nicer scale)
    let mut y: (f64, f64) = (0.0, 0.0);
    let mut dates: (i64, i64) = (0, 0);
    for (op_index, (op, epochs)) in data.iter().enumerate() {
        if cmap.get(op).is_none() {
            cmap.insert(op.clone(),
                Palette99::pick(op_index) // RGB
                    .mix(0.99)); // RGBA
        }
        for (e_index, (epoch, data)) in epochs.iter().enumerate() {
            if e_index == 0 {
                dates.0 = epoch.date.timestamp();
            }
            if epoch.date.timestamp() > dates.1 {
                dates.1 = epoch.date.timestamp();
            }
            if *data < y.0 {
                y.0 = *data;
            }
            if *data > y.1 {
                y.1 = *data;
            }
        }
    }

    // build a chart
    let x_axis = 0.0..((dates.1-dates.0) as f64);
    let y_axis = y.0*0.9..y.1*1.1;
    let mut chart = ChartBuilder::on(&p)
        .caption("Phase Code Differential analysis", ("sans-serif", 50).into_font())
        .margin(40)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(x_axis, y_axis)
        .expect("failed to build a chart");
    chart
        .configure_mesh()
        .x_desc("Timestamp [s]")
        .x_labels(30)
        .y_desc("Phase Difference [n.a]")
        .y_labels(30)
        .draw()
        .expect("failed to draw mesh");
    /*
     * Plot all Ops
     */
    for (op, epochs) in data {
        let color = cmap.get(op).unwrap(); 
        chart.draw_series(LineSeries::new(
            epochs.iter()
                .map(|(k, v)| {
                    ((k.date.timestamp() - dates.0) as f64, *v) 
                }),
                color.clone(),
            ))
            .expect(&format!("failed to draw {} serie", op))
            .label(op.clone())
            .legend(|(x, y)| {
                PathElement::new(vec![(x, y), (x+20, y)], color.clone())
            });
        chart.draw_series(
            epochs.iter()
                .map(|(k, v)| {
                    let x = (k.date.timestamp() - dates.0) as f64;
                    Cross::new((x, *v), 4, color.clone())
                }))
                .expect(&format!("failed to draw {} serie", op));
    }
    chart
        .configure_series_labels()
        .border_style(&BLACK)
        .background_style(WHITE.filled())
        .draw()
        .expect("failed to plot phase diff analysis");
}
