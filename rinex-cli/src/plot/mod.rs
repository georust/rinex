use rinex::prelude::*;
use plotters::{
    prelude::*,
    coord::Shift,
    chart::{
        ChartState,
        DualCoordChartState,
    },
    coord::types::RangedCoordf64,
};
use std::collections::{BTreeMap, HashMap};

pub mod record;
pub type Plot2d = Cartesian2d<RangedCoordf64, RangedCoordf64>;

/// Builds plot area
pub fn build_plot(file: &str, dims: (u32, u32)) -> DrawingArea<BitMapBackend, Shift> {
    let area = BitMapBackend::new(file, dims).into_drawing_area();
    area.fill(&WHITE)
        .expect("failed to create background image");
    area
}

/// Builds a chart
pub fn build_chart(
    title: &str,
    x_axis: Vec<f64>,
    y_range: (f64, f64),
    area: &DrawingArea<BitMapBackend, Shift>,
) -> ChartState<Plot2d> {
    let x_axis = x_axis[0]..x_axis[x_axis.len() - 1];
    // y axis is scaled for better rendering
    let y_axis = match y_range.0 < 0.0 {
        true => 1.02 * y_range.0..1.02 * y_range.1,
        false => 0.98 * y_range.0..1.02 * y_range.1,
    };
    let mut chart = ChartBuilder::on(area)
        .caption(title, ("sans-serif", 50).into_font())
        .margin(40)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(x_axis, y_axis)
        .expect(&format!("failed to build {} chart", title));
    chart
        .configure_mesh()
        .x_desc("Timestamp [s]") //TODO not for special records
        .x_labels(30)
        .y_desc(title)
        .y_labels(30)
        .y_label_formatter(&|y| format!("{:e}", y)) //nicer f64 rendering
        .draw()
        .expect(&format!("failed to draw {} mesh", title));
    chart
        .to_chart_state()
}

/*
 * Builds a chart with 2 Y axes and shared X axis
 */
pub fn build_twoscale_chart(
    title: &str, 
    x_axis: Vec<f64>,
    y_ranges: ((f64, f64), (f64,f64)), // Y_right, Y_left
    area: &DrawingArea<BitMapBackend, Shift>) 
    -> DualCoordChartState<Plot2d, Plot2d> 
{
    let x_axis = x_axis[0]..x_axis[x_axis.len()-1]; 
    
    // y right range
    let (yr_range, yl_range) = y_ranges;
    let yr_axis = match yr_range.0 < 0.0 {
        true => 1.02*yr_range.0..1.02*yr_range.1,
        false => 0.98*yr_range.0..1.02*yr_range.1,
    };

    // y left range
    let yl_axis = match yl_range.0 < 0.0 {
        true => 1.02*yl_range.0..1.02*yl_range.1,
        false => 0.98*yl_range.0..1.02*yl_range.1,
    };

    let mut chart = ChartBuilder::on(area)
        .caption(title, ("sans-serif", 50).into_font())
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .right_y_label_area_size(50)
        .build_cartesian_2d(x_axis.clone(), yr_axis)
        .expect(&format!("failed to build {} chart", title))
        .set_secondary_coord(x_axis.clone(), yl_axis); // shared X
    chart
        .configure_mesh()
        .x_desc("Timestamp [s]")
        .x_labels(30)
        .y_desc(title)
        .y_labels(30)
        .y_label_formatter(&|y| format!("{:e}", y)) //nicer f64 rendering
        .draw()
        .expect(&format!("failed to draw {} mesh", title));
    chart
        .configure_secondary_axes()
        .y_desc("Evelation angle [°]") // TODO: might require some improvement, 
            // in case we have other use cases
        .draw()
        .expect(&format!("failed to draw {} secondary axis", title));
    chart
        .to_chart_state()
}

/*
 * Plots (any kind of) recombined GNSS dataset
 */
pub fn plot_gnss_recombination(
    dims: (u32, u32),
    file: &str,
    caption: &str,
    y_desc: &str,
    data: &HashMap<String, HashMap<Sv, BTreeMap<(Epoch, EpochFlag), f64>>>,
) {
    let p = build_plot(file, dims);
    // color map: one per sv
    let cmap = colorous::TURBO;
    let mut cmap_max_index = 0_u8;
    // one symbol per op
    let symbols = vec!["x", "t", "o"];
    // determine (smallest, largest) ts accross all Ops
    // determine (smallest, largest) y accross all Ops (nicer scale)
    let mut y: (f64, f64) = (0.0, 0.0);
    let mut dates: (f64, f64) = (0.0, 0.0);
    for (_op_index, (_op, vehicules)) in data.iter().enumerate() {
        for (sv, epochs) in vehicules.iter() {
            if sv.prn > cmap_max_index {
                cmap_max_index = sv.prn;
            }
            for (e_index, ((epoch, _flag), data)) in epochs.iter().enumerate() {
                if e_index == 0 {
                    dates.0 = epoch.to_utc_seconds();
                }
                if epoch.to_utc_seconds() > dates.1 {
                    dates.1 = epoch.to_utc_seconds();
                }
                let yp = data; // * 1.546;
                if *yp < y.0 {
                    y.0 = *yp;
                }
                if *yp > y.1 {
                    y.1 = *yp;
                }
            }
        }
    }

    // build a chart
    let x_axis = 0.0..((dates.1 - dates.0) as f64);
    // y axis is scaled for better rendering
    let y_axis = match y.0 < 0.0 {
        true => y.0 * 1.1..y.1 * 1.1,
        false => y.0 * 0.9..y.1 * 1.1,
    };
    let mut chart = ChartBuilder::on(&p)
        .caption(caption, ("sans-serif", 50).into_font())
        .margin(10)
        .x_label_area_size(30)
        .y_label_area_size(80)
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
            let color = cmap.eval_rational(sv.prn.into(), cmap_max_index.into());
            let color = RGBColor {
                0: color.r,
                1: color.g,
                2: color.b,
            };
            /*chart.draw_series(LineSeries::new(
            epochs.iter()
                .map(|((k, flag), v)| (k.to_utc_seconds() - dates.0, *v)),
                color.clone(),
            ))
            .expect(&format!("failed to draw {} serie", op));*/
            chart
                .draw_series(epochs.iter().map(|((k, _flag), v)| {
                    let x = k.to_utc_seconds() - dates.0;
                    match symbol {
                        "x" => Cross::new((x, *v), 4, Into::<ShapeStyle>::into(&color).filled())
                            .into_dyn(),
                        "o" => Circle::new((x, *v), 4, Into::<ShapeStyle>::into(&color).filled())
                            .into_dyn(),
                        _ => TriangleMarker::new(
                            (x, *v),
                            4,
                            Into::<ShapeStyle>::into(&color).filled(),
                        )
                        .into_dyn(),
                    }
                }))
                .expect(&format!("failed to draw {} serie", op))
                .label(&format!("{}({})", op, sv))
                .legend(move |point| match symbol {
                    "x" => {
                        Cross::new(point, 4, Into::<ShapeStyle>::into(&color).filled()).into_dyn()
                    },
                    "o" => {
                        Circle::new(point, 4, Into::<ShapeStyle>::into(&color).filled()).into_dyn()
                    },
                    _ => TriangleMarker::new(point, 4, Into::<ShapeStyle>::into(&color).filled())
                        .into_dyn(),
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
