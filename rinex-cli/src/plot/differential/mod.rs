use super::build_plot;
use rinex::prelude::*;
use plotters::prelude::*;
use std::collections::{BTreeMap, HashMap};

pub fn plot(
    dims: (u32,u32), 
    file: &str, 
    caption: &str,
    y_desc: &str,
    data: &HashMap<String, HashMap<Sv, BTreeMap<Epoch, f64>>>) 
{
    let p = build_plot(file, dims);
    // one symbol per op
    let symbols = vec!["x","t","o"];
    // colormap: one per sv 
    let mut cmap: HashMap<Sv, RGBAColor> = HashMap::new();
    // determine (smallest, largest) ts accross all Ops
    // determine (smallest, largest) y accross all Ops (nicer scale)
    let mut y: (f64, f64) = (0.0, 0.0);
    let mut dates: (i64, i64) = (0, 0);
    for (op_index, (op, vehicules)) in data.iter().enumerate() {
        for (sv, epochs) in vehicules {
            if cmap.get(sv).is_none() {
                cmap.insert(*sv,
                    Palette9999::pick((sv.prn/2+5).into()) // RGB
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
    }

    // build a chart
    let x_axis = 0.0..((dates.1-dates.0) as f64);
    let y_axis = y.0..y.1*1.1;
    let mut chart = ChartBuilder::on(&p)
        .caption(caption, ("sans-serif", 50).into_font())
        .margin(40)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(x_axis, y_axis)
        .expect("failed to build a chart");
    chart
        .configure_mesh()
        .x_desc("Timestamp [s]")
        .x_labels(30)
        .y_desc(y_desc)
        .y_labels(30)
        .draw()
        .expect("failed to draw mesh");
    /*
     * Plot all ops
     */
    for (op_index, (op, vehicules)) in data.iter().enumerate() {
        let symbol = symbols[op_index % symbols.len()];
        for (sv, epochs) in vehicules {
            let color = cmap.get(sv).unwrap(); 
            chart.draw_series(LineSeries::new(
                epochs.iter()
                    .map(|(k, v)| {
                        ((k.date.timestamp() - dates.0) as f64, *v) 
                    }),
                    color.clone(),
                ))
                .expect(&format!("failed to draw {} serie", op));
            chart.draw_series(
                epochs.iter()
                    .map(|(k, v)| {
                        let x = (k.date.timestamp() - dates.0) as f64;
                        match symbol {
                            "x" => {
                                Cross::new((x, *v), 4,
                                    Into::<ShapeStyle>::into(&color).filled())
                                    .into_dyn()
                            },
                            "o" => {
                                Circle::new((x, *v), 4,
                                    Into::<ShapeStyle>::into(&color).filled())
                                    .into_dyn()
                            },
                            _ => {
                                TriangleMarker::new((x, *v), 4,
                                    Into::<ShapeStyle>::into(&color).filled())
                                    .into_dyn()
                            }
                        }
                    }))
                    .expect(&format!("failed to draw {} serie", op))
                    .label(&format!("{}({})", op, sv))
                    .legend(move |point| {
                        match symbol {
                            "x" => {
                                Cross::new(point, 4,
                                    Into::<ShapeStyle>::into(&color).filled())
                                    .into_dyn()
                            },
                            "o" => {
                                Circle::new(point, 4,
                                    Into::<ShapeStyle>::into(&color).filled())
                                    .into_dyn()
                            },
                            _ => {
                                TriangleMarker::new(point, 4,
                                    Into::<ShapeStyle>::into(&color).filled())
                                    .into_dyn()
                            },
                        }
                    });
        }
    }
    chart
        .configure_series_labels()
        .border_style(&BLACK)
        .background_style(WHITE.filled())
        .draw()
        .expect("failed to draw chart");
}
