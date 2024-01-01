use crate::{cli::Context, Error};
use clap::ArgMatches;
use rinex::observation::{Combination, Combine, Dcb};

use plotly::{
    common::{
        AxisSide,
        //DashType,
        Font,
        HoverInfo,
        Marker,
        MarkerSymbol,
        Mode,
        Side,
        Title,
    },
    layout::{Axis, Center, DragMode, Mapbox, MapboxStyle, Margin},
    Layout, Plot, Scatter, Scatter3D,
};

use rand::Rng;
use serde::Serialize;

use rinex::prelude::*;

mod record;
use record::{
    plot_atmosphere_conditions, plot_residual_ephemeris, plot_sv_nav_clock, plot_sv_nav_orbits,
};

mod context;
pub use context::PlotContext;

mod skyplot;
use skyplot::skyplot;

mod naviplot;

mod combination;
use combination::{plot_gnss_code_mp, plot_gnss_combination, plot_gnss_dcb};

mod csv; // export to CSV instead of plotting
pub use csv::csv_export_timedomain;

/*
 * Generates N marker symbols to be used
 * to differentiate data
 */
pub fn generate_markers(n: usize) -> Vec<MarkerSymbol> {
    //TODO lazy static
    let pool = vec![
        "Circle",
        "CircleOpen",
        "CircleDot",
        "CircleOpenDot",
        "Square",
        "SquareOpen",
        "SquareDot",
        "SquareOpenDot",
        "Diamond",
        "DiamondOpen",
        "DiamondDot",
        "DiamondOpenDot",
        "Cross",
        "CrossOpen",
        "CrossDot",
        "CrossOpenDot",
        "X",
        "XOpen",
        "XDot",
        "XOpenDot",
        "TriangleUp",
        "TriangleUpOpen",
        "TriangleUpDot",
        "TriangleUpOpenDot",
        "TriangleDown",
        "TriangleDownOpen",
        "TriangleDownDot",
        "TriangleDownOpenDot",
        "TriangleLeft",
        "TriangleLeftOpen",
        "TriangleLeftDot",
        "TriangleLeftOpenDot",
        "TriangleRight",
        "TriangleRightOpen",
        "TriangleRightDot",
        "TriangleRightOpenDot",
        "TriangleNE",
        "TriangleNEOpen",
        "TriangleNEDot",
        "TriangleNEOpenDot",
        "TriangleSE",
        "TriangleSEOpen",
        "TriangleSEDot",
        "TriangleSEOpenDot",
        "TriangleSW",
        "TriangleSWOpen",
        "TriangleSWDot",
        "TriangleSWOpenDot",
        "TriangleNW",
        "TriangleNWOpen",
        "TriangleNWDot",
        "TriangleNWOpenDot",
        "Pentagon",
        "PentagonOpen",
        "PentagonDot",
        "PentagonOpenDot",
        "Hexagon",
        "HexagonOpen",
        "HexagonDot",
        "HexagonOpenDot",
        "Hexagon2",
        "Hexagon2Open",
        "Hexagon2Dot",
        "Hexagon2OpenDot",
        "Octagon",
        "OctagonOpen",
        "OctagonDot",
        "OctagonOpenDot",
        "Star",
        "StarOpen",
        "StarDot",
        "StarOpenDot",
        "Hexagram",
        "HexagramOpen",
        "HexagramDot",
        "HexagramOpenDot",
        "StarTriangleUp",
        "StarTriangleUpOpen",
        "StarTriangleUpDot",
        "StarTriangleUpOpenDot",
        "StarTriangleDown",
        "StarTriangleDownOpen",
        "StarTriangleDownDot",
        "StarTriangleDownOpenDot",
        "StarSquare",
        "StarSquareOpen",
        "StarSquareDot",
        "StarSquareOpenDot",
        "StarDiamond",
        "StarDiamondOpen",
        "StarDiamondDot",
        "StarDiamondOpenDot",
        "DiamondTall",
        "DiamondTallOpen",
        "DiamondTallDot",
        "DiamondTallOpenDot",
        "DiamondWide",
        "DiamondWideOpen",
        "DiamondWideDot",
        "DiamondWideOpenDot",
        "Hourglass",
        "HourglassOpen",
        "BowTie",
        "BowTieOpen",
        "CircleCross",
        "CircleCrossOpen",
        "CircleX",
        "CircleXOpen",
        "SquareCross",
        "SquareCrossOpen",
        "SquareX",
        "SquareXOpen",
        "DiamondCross",
        "DiamondCrossOpen",
        "DiamondX",
        "DiamondXOpen",
        "CrossThin",
        "CrossThinOpen",
        "XThin",
        "XThinOpen",
        "Asterisk",
        "AsteriskOpen",
        "Hash",
        "HashOpen",
        "HashDot",
        "HashOpenDot",
        "YUp",
        "YUpOpen",
        "YDown",
        "YDownOpen",
        "YLeft",
        "YLeftOpen",
        "YRight",
        "YRightOpen",
        "LineEW",
        "LineEWOpen",
        "LineNS",
        "LineNSOpen",
        "LineNE",
        "LineNEOpen",
        "LineNW",
        "LineNWOpen",
    ];
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
pub fn build_timedomain_plot(title: &str, y_title: &str) -> Plot {
    build_plot(
        title,
        Side::Top,
        Font::default(),
        "MJD",
        y_title,
        (true, true), // y=0 lines
        true,         // show legend
        true,         // autosize
        true,         // show tick labels
        0.25,         // ticks dx
        "{:05}",      // ticks fmt
    )
}

/*
 * builds a standard 3D plot
 */
pub fn build_default_3d_plot(title: &str, x_title: &str, y_title: &str, z_title: &str) -> Plot {
    build_3d_plot(
        title,
        Side::Top,
        Font::default(),
        x_title,
        y_title,
        z_title,
        (true, true, true), // x=0,y=0,z=0 bold lines
        true,               // show legend
        true,               // autosize
    )
}

/*
 * build a standard 2D plot dual Y axes,
 * to plot against `Epochs`
 */
pub fn build_timedomain_2y_plot(title: &str, y1_title: &str, y2_title: &str) -> Plot {
    build_plot_2y(
        title,
        Side::Top,
        Font::default(),
        "MJD",
        y1_title,
        y2_title,
        (false, false), // y=0 lines
        true,           // show legend
        true,           // autosize
        true,           // show x tick label
        0.25,           // dx tick
        "{:05}",        // x tick fmt
    )
}

/*
 * Builds a default Polar2D plot
 */
pub fn build_default_polar_plot(title: &str) -> Plot {
    let layout = Layout::new()
        .title(Title::new(title))
        .x_axis(
            Axis::new()
                .title(Title::new("Latitude [°]").side(Side::Top))
                .zero_line(true), //.show_tick_labels(show_tick_labels)
                                  //.dtick(dx_tick)
                                  //.tick_format(tick_fmt)
        )
        .y_axis(
            Axis::new()
                .title(Title::new("Longitude [°]"))
                .zero_line(true),
        )
        .show_legend(true)
        .auto_size(true);
    let mut p = Plot::new();
    p.set_layout(layout);
    p
}

/*
 * Builds a world map,
 * centered on given locations, in decimal degrees,
 * zoom factor
 */
pub fn build_world_map(
    title: &str,
    show_legend: bool,
    map_style: MapboxStyle,
    center: (f64, f64),
    zoom: u8,
) -> Plot {
    let mut p = Plot::new();
    let layout = Layout::new()
        .title(Title::new(title).font(Font::default()))
        .drag_mode(DragMode::Zoom)
        .margin(Margin::new().top(0).left(0).bottom(0).right(0))
        .show_legend(show_legend)
        .mapbox(
            Mapbox::new()
                .style(map_style)
                .center(Center::new(center.0, center.1))
                .zoom(zoom),
        );
    p.set_layout(layout);
    p
}

/*
 * Builds a Plot
 */
fn build_plot(
    title: &str,
    title_side: Side,
    title_font: Font,
    x_axis_title: &str,
    y_axis_title: &str,
    zero_line: (bool, bool), // plots a bold line @ (x=0,y=0)
    show_legend: bool,
    auto_size: bool,
    show_xtick_labels: bool,
    dx_tick: f64,
    x_tick_fmt: &str,
) -> Plot {
    let layout = Layout::new()
        .title(Title::new(title).font(title_font))
        .x_axis(
            Axis::new()
                .title(Title::new(x_axis_title).side(title_side))
                .zero_line(zero_line.0)
                .show_tick_labels(show_xtick_labels)
                .dtick(dx_tick)
                .tick_format(x_tick_fmt),
        )
        .y_axis(
            Axis::new()
                .title(Title::new(y_axis_title))
                .zero_line(zero_line.0),
        )
        .show_legend(show_legend)
        .auto_size(auto_size);
    let mut p = Plot::new();
    p.set_layout(layout);
    p
}

fn build_plot_2y(
    title: &str,
    title_side: Side,
    title_font: Font,
    x_title: &str,
    y1_title: &str,
    y2_title: &str,
    zero_line: (bool, bool), // plots a bold line @ (x=0,y=0)
    show_legend: bool,
    auto_size: bool,
    show_xtick_labels: bool,
    dx_tick: f64,
    xtick_fmt: &str,
) -> Plot {
    let layout = Layout::new()
        .title(Title::new(title).font(title_font))
        .x_axis(
            Axis::new()
                .title(Title::new(x_title).side(title_side))
                .zero_line(zero_line.0)
                .show_tick_labels(show_xtick_labels)
                .dtick(dx_tick)
                .tick_format(xtick_fmt),
        )
        .y_axis(
            Axis::new()
                .title(Title::new(y1_title))
                .zero_line(zero_line.1),
        )
        .y_axis2(
            Axis::new()
                .title(Title::new(y2_title))
                .overlaying("y")
                .side(AxisSide::Right)
                .zero_line(zero_line.1),
        )
        .show_legend(show_legend)
        .auto_size(auto_size);
    let mut p = Plot::new();
    p.set_layout(layout);
    p
}

fn build_3d_plot(
    title: &str,
    title_side: Side,
    title_font: Font,
    x_title: &str,
    y_title: &str,
    z_title: &str,
    zero_line: (bool, bool, bool), // plots a bold line @ (x=0,y=0,z=0)
    show_legend: bool,
    auto_size: bool,
) -> Plot {
    let layout = Layout::new()
        .title(Title::new(title).font(title_font))
        .x_axis(
            Axis::new()
                .title(Title::new(x_title).side(title_side))
                .zero_line(zero_line.0)
                .show_tick_labels(false),
        )
        .y_axis(
            Axis::new()
                .title(Title::new(y_title))
                .zero_line(zero_line.1),
        )
        .z_axis(
            Axis::new()
                .title(Title::new(z_title))
                .zero_line(zero_line.2),
        )
        .show_legend(show_legend)
        .auto_size(auto_size);
    let mut p = Plot::new();
    p.set_layout(layout);
    p
}

/*
 * Builds a default chart, 2D, X = time axis
 */
pub fn build_chart_epoch_axis<T: Clone + Default + Serialize>(
    name: &str,
    mode: Mode,
    epochs: Vec<Epoch>,
    data_y: Vec<T>,
) -> Box<Scatter<f64, T>> {
    let txt: Vec<String> = epochs.iter().map(|e| e.to_string()).collect();
    Scatter::new(epochs.iter().map(|e| e.to_mjd_utc_days()).collect(), data_y)
        .mode(mode)
        //.web_gl_mode(true)
        .name(name)
        .hover_text_array(txt)
        .hover_info(HoverInfo::All)
}

/*
 * Builds a default 3D chart
 */
pub fn build_3d_chart_epoch_label<T: Clone + Default + Serialize>(
    name: &str,
    mode: Mode,
    epochs: Vec<Epoch>,
    x: Vec<T>,
    y: Vec<T>,
    z: Vec<T>,
) -> Box<Scatter3D<T, T, T>> {
    let txt: Vec<String> = epochs.iter().map(|e| e.to_string()).collect();
    Scatter3D::new(x, y, z)
        .mode(mode)
        //.web_gl_mode(true)
        .name(name)
        .hover_text_array(txt)
        .hover_info(HoverInfo::All)
}

/* Returns True if GNSS combination is to be plotted */
fn gnss_combination_plot(matches: &ArgMatches) -> bool {
    matches.get_flag("if")
        || matches.get_flag("gf")
        || matches.get_flag("wl")
        || matches.get_flag("nl")
        || matches.get_flag("mw")
}

/* Returns True if Navigation plot is to be generated */
fn navigation_plot(matches: &ArgMatches) -> bool {
    matches.get_flag("skyplot") || matches.get_flag("sp3-res") || matches.get_flag("sv-clock")
}

/* Returns True if Atmosphere conditions is to be generated */
fn atmosphere_plot(matches: &ArgMatches) -> bool {
    matches.get_flag("tropo") || matches.get_flag("tec") || matches.get_flag("ionod")
}

pub fn graph_opmode(ctx: &Context, matches: &ArgMatches) -> Result<(), Error> {
    /*
     * Prepare session:
     *  + HTML: (default) in this session directly
     *  + CSV: (option): generate a subdir
     */
    let csv_export = matches.get_flag("csv");
    if csv_export {
        ctx.create_subdir("CSV");
    }
    /*
     * Observations graphs
     */
    if matches.get_flag("obs") {
        let mut plot_ctx = PlotContext::new();
        if ctx.data.has_observation_data() {
            record::plot_observations(ctx, &mut plot_ctx, csv_export);
        }
        if let Some(data) = ctx.data.meteo_data() {
            record::plot_meteo_observations(data, &mut plot_ctx);
        }

        /* save observations */
        ctx.render_html("OBSERVATIONS.html", plot_ctx.to_html());
    }
    /*
     * GNSS combinations graphs
     */
    if gnss_combination_plot(matches) {
        let data = ctx.data.obs_data().ok_or(Error::MissingObservationRinex)?;

        let mut plot_ctx = PlotContext::new();
        if matches.get_flag("if") {
            let combination = data.combine(Combination::IonosphereFree);
            plot_gnss_combination(
                &combination,
                &mut plot_ctx,
                "Ionosphere Free combination",
                "Meters of delay",
            );
        }
        if matches.get_flag("gf") {
            let combination = data.combine(Combination::GeometryFree);
            plot_gnss_combination(
                &combination,
                &mut plot_ctx,
                "Ionosphere Free combination",
                "Meters of delay",
            );
        }
        if matches.get_flag("wl") {
            let combination = data.combine(Combination::WideLane);
            plot_gnss_combination(
                &combination,
                &mut plot_ctx,
                "Ionosphere Free combination",
                "Meters of delay",
            );
        }
        if matches.get_flag("nl") {
            let combination = data.combine(Combination::NarrowLane);
            plot_gnss_combination(
                &combination,
                &mut plot_ctx,
                "Ionosphere Free combination",
                "Meters of delay",
            );
        }
        if matches.get_flag("mw") {
            let combination = data.combine(Combination::MelbourneWubbena);
            plot_gnss_combination(
                &combination,
                &mut plot_ctx,
                "Ionosphere Free combination",
                "Meters of delay",
            );
        }

        /* save combinations */
        ctx.render_html("COMBINATIONS.html", plot_ctx.to_html());
    }
    /*
     * DCB visualization
     */
    if matches.get_flag("dcb") {
        let data = ctx.data.obs_data().ok_or(Error::MissingObservationRinex)?;

        let mut plot_ctx = PlotContext::new();
        let data = data.dcb();
        plot_gnss_dcb(
            &data,
            &mut plot_ctx,
            "Differential Code Bias",
            "Differential Code Bias [s]",
        );

        /* save DCB */
        ctx.render_html("DCB.html", plot_ctx.to_html());
    }
    if matches.get_flag("mp") {
        let data = ctx.data.obs_data().ok_or(Error::MissingObservationRinex)?;

        let mut plot_ctx = PlotContext::new();
        let data = data.code_multipath();
        plot_gnss_code_mp(&data, &mut plot_ctx, "Code Multipath", "Meters of delay");

        /* save MP */
        ctx.render_html("MULTIPATH.html", plot_ctx.to_html());
    }
    if navigation_plot(matches) {
        let mut plot_ctx = PlotContext::new();

        if matches.get_flag("skyplot") {
            let rx_ecef = ctx
                .rx_ecef
                .expect("skyplot requires the receiver location to be defined.");
            if ctx.data.sp3_data().is_none() && ctx.data.nav_data().is_none() {
                panic!("skyplot requires either BRDC or SP3.");
            }
            skyplot(&ctx.data, rx_ecef, &mut plot_ctx);
        }
        if matches.get_flag("orbits") {
            plot_sv_nav_orbits(&ctx.data, &mut plot_ctx);
        }
        if matches.get_flag("sp3-res") {
            if ctx.data.sp3_data().is_none() || ctx.data.nav_data().is_none() {
                panic!("skyplot requires both BRDC or SP3.");
            }
            plot_residual_ephemeris(&ctx.data, &mut plot_ctx);
        }
        /* save NAV */
        ctx.render_html("NAVIGATION.html", plot_ctx.to_html());
    }
    if matches.get_flag("sv-clock") {
        let mut plot_ctx = PlotContext::new();
        plot_sv_nav_clock(&ctx.data, &mut plot_ctx);

        /* save CLK */
        ctx.render_html("CLOCKS.html", plot_ctx.to_html());
    }
    if atmosphere_plot(matches) {
        let mut plot_ctx = PlotContext::new();
        plot_atmosphere_conditions(ctx, &mut plot_ctx, matches);

        /* save ATMOSPHERE */
        ctx.render_html("ATMOSPHERE.html", plot_ctx.to_html());
    }
    Ok(())
}
