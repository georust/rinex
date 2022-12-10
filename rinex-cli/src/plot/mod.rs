use plotly::{
    Plot, 
    Layout,
    Scatter, //ScatterPolar,
    common::{
        Mode, 
        //DashType, 
        Font,
        Title, Side, Marker, MarkerSymbol,
    },
    layout::{Axis},
};
use rand::Rng;

mod context;
pub use context::Context;

mod skyplot;
pub use skyplot::skyplot;

mod combination;
pub use combination::plot_gnss_recombination;

/*
 * Generates N marker symbols to be used
 * to differentiate data
 */
pub fn generate_markers(n: usize) -> Vec<MarkerSymbol> {
    //TODO lazy static
    let pool = vec!["Circle", "CircleOpen", "CircleDot", "CircleOpenDot", "Square", "SquareOpen", "SquareDot", "SquareOpenDot", "Diamond", "DiamondOpen", "DiamondDot", "DiamondOpenDot", "Cross", "CrossOpen", "CrossDot", "CrossOpenDot", "X", "XOpen", "XDot", "XOpenDot", "TriangleUp", "TriangleUpOpen", "TriangleUpDot", "TriangleUpOpenDot", "TriangleDown", "TriangleDownOpen", "TriangleDownDot", "TriangleDownOpenDot", "TriangleLeft", "TriangleLeftOpen", "TriangleLeftDot", "TriangleLeftOpenDot", "TriangleRight", "TriangleRightOpen", "TriangleRightDot", "TriangleRightOpenDot", "TriangleNE", "TriangleNEOpen", "TriangleNEDot", "TriangleNEOpenDot", "TriangleSE", "TriangleSEOpen", "TriangleSEDot", "TriangleSEOpenDot", "TriangleSW", "TriangleSWOpen", "TriangleSWDot", "TriangleSWOpenDot", "TriangleNW", "TriangleNWOpen", "TriangleNWDot", "TriangleNWOpenDot", "Pentagon", "PentagonOpen", "PentagonDot", "PentagonOpenDot", "Hexagon", "HexagonOpen", "HexagonDot", "HexagonOpenDot", "Hexagon2", "Hexagon2Open", "Hexagon2Dot", "Hexagon2OpenDot", "Octagon", "OctagonOpen", "OctagonDot", "OctagonOpenDot", "Star", "StarOpen", "StarDot", "StarOpenDot", "Hexagram", "HexagramOpen", "HexagramDot", "HexagramOpenDot", "StarTriangleUp", "StarTriangleUpOpen", "StarTriangleUpDot", "StarTriangleUpOpenDot", "StarTriangleDown", "StarTriangleDownOpen", "StarTriangleDownDot", "StarTriangleDownOpenDot", "StarSquare", "StarSquareOpen", "StarSquareDot", "StarSquareOpenDot", "StarDiamond", "StarDiamondOpen", "StarDiamondDot", "StarDiamondOpenDot", "DiamondTall", "DiamondTallOpen", "DiamondTallDot", "DiamondTallOpenDot", "DiamondWide", "DiamondWideOpen", "DiamondWideDot", "DiamondWideOpenDot", "Hourglass", "HourglassOpen", "BowTie", "BowTieOpen", "CircleCross", "CircleCrossOpen", "CircleX", "CircleXOpen", "SquareCross", "SquareCrossOpen", "SquareX", "SquareXOpen", "DiamondCross", "DiamondCrossOpen", "DiamondX", "DiamondXOpen", "CrossThin", "CrossThinOpen", "XThin", "XThinOpen", "Asterisk", "AsteriskOpen", "Hash", "HashOpen", "HashDot", "HashOpenDot", "YUp", "YUpOpen", "YDown", "YDownOpen", "YLeft", "YLeftOpen", "YRight", "YRightOpen", "LineEW", "LineEWOpen", "LineNS", "LineNSOpen", "LineNE", "LineNEOpen", "LineNW", "LineNWOpen", ];
    let nb_max: usize = pool.len();
    let mut rng = rand::thread_rng();
    let mut ret: Vec<MarkerSymbol> = Vec::with_capacity(n);
    for _ in 0..n {
        let symbol = pool[rng.gen_range(0..25)];
        let marker = match symbol {
            "Circle" => MarkerSymbol::Circle,
            "CircleOpen" => MarkerSymbol::CircleOpen,
            "CircleDot" => MarkerSymbol::CircleDot,
            "CircleOpenDot" => MarkerSymbol::CircleOpenDot,
            "Square" => MarkerSymbol::Square,
            "SquareDot" => MarkerSymbol::SquareDot,
            "SquareOpen" => MarkerSymbol::SquareOpen,
            "SquareOpenDot" => MarkerSymbol::SquareOpenDot,
            "Diamond" => MarkerSymbol::Diamond,
            "DiamondOpen" => MarkerSymbol::DiamondOpen,
            "DiamondDot" => MarkerSymbol::DiamondDot,
            "DiamondOpenDot" => MarkerSymbol::DiamondOpenDot,
            "Hash" => MarkerSymbol::Hash,
            "HashDot" => MarkerSymbol::HashDot,
            "HashOpen" => MarkerSymbol::HashOpen,
            "HashOpenDot" => MarkerSymbol::HashOpenDot,
            "Cross" => MarkerSymbol::Cross,
            "CrossDot" => MarkerSymbol::CrossDot,
            "CrossOpen" => MarkerSymbol::CrossOpen,
            "CrossOpenDot" => MarkerSymbol::CrossOpenDot,
            "TriangleUp" => MarkerSymbol::TriangleUp,
            "TriangleUpDot" => MarkerSymbol::TriangleUpDot,
            "TriangleUpOpen" => MarkerSymbol::TriangleUpOpen,
            "TriangleUpOpenDot" => MarkerSymbol::TriangleUpOpenDot,
            "TriangleDown" => MarkerSymbol::TriangleDown,
            "X" => MarkerSymbol::X,
            "XOpen" => MarkerSymbol::XOpen,
            "XDot" => MarkerSymbol::XDot,
            "XOpenDot" => MarkerSymbol::XOpenDot,
            "YUp" => MarkerSymbol::YUp,
            "YUpOpen" => MarkerSymbol::YUpOpen,
            "YDown" => MarkerSymbol::YDown,
            "YDownOpen" => MarkerSymbol::YDownOpen,
            _ => MarkerSymbol::Cross,
        };
        ret.push(marker);
    }
    ret
}

/*
 * builds a standard 2D plot single Y scale,
 * ready to plot data against time (`Epoch`)
 */
pub fn build_default_plot(title: &str, y_title: &str) -> Plot {
    build_plot(
        title, 
        Side::Top, 
        Font::default(),
        "Epoch (UTC)",
        y_title,
        (false, false), // zero lines
        true, // show legend
        true, // autosize
    )
}

/*
 * Builds a Plot
 */
pub fn build_plot(
    title: &str,
    title_side: Side,
    title_font: Font,
    x_axis_title: &str,
    y_axis_title: &str,
    zero_line: (bool, bool), // plots a bold line @ (x=0,y=0)
    show_legend: bool,
    auto_size: bool,
    ) -> Plot {
    let layout = Layout::new()
        .title(Title::new(title)
            .font(title_font)
        )
        .x_axis(
            Axis::new()
                .title(Title::new(x_axis_title))
                .zero_line(zero_line.0)
                .show_tick_labels(false)
        )
        .y_axis(
            Axis::new()
                .title(Title::new(y_axis_title))
                .zero_line(zero_line.0)
        )
        .show_legend(show_legend)
        .auto_size(auto_size);
    let mut p = Plot::new();
    p.set_layout(layout);
    p
}

//pub mod record;

/*
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
    chart.to_chart_state()
}

/*
 * Builds a chart with 2 Y axes and shared X axis
 */
pub fn build_twoscale_chart(
    title: &str,
    x_axis: Vec<f64>,
    y_ranges: ((f64, f64), (f64, f64)), // Y_right, Y_left
    area: &DrawingArea<BitMapBackend, Shift>,
) -> DualCoordChartState<Plot2d, Plot2d> {
    let x_axis = x_axis[0]..x_axis[x_axis.len() - 1];

    // y right range
    let (yr_range, yl_range) = y_ranges;
    let yr_axis = match yr_range.0 < 0.0 {
        true => 1.02 * yr_range.0..1.02 * yr_range.1,
        false => 0.98 * yr_range.0..1.02 * yr_range.1,
    };

    // y left range
    let yl_axis = match yl_range.0 < 0.0 {
        true => 1.02 * yl_range.0..1.02 * yl_range.1,
        false => 0.98 * yl_range.0..1.02 * yl_range.1,
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
    chart.to_chart_state()
}
*/
