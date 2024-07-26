use hifitime::Epoch;
use maud::{html, Markup, PreEscaped, Render};
use plotly::{
    common::{Font, HoverInfo, Side},
    layout::{
        Axis, Center, DragMode, Mapbox, Margin, RangeSelector, RangeSlider, SelectorButton,
        SelectorStep,
    },
    DensityMapbox, Layout, Plot as Plotly, Scatter, Scatter3D, ScatterGeo, ScatterMapbox,
    ScatterPolar, Trace,
};

use serde::Serialize;

pub use plotly::{
    color::NamedColor,
    common::{Marker, MarkerSymbol, Mode, Visible},
    layout::MapboxStyle,
};

pub struct CompassArrow {
    pub scatter: Box<ScatterPolar<f64, f64>>,
}

impl CompassArrow {
    /// Creates new [CompassArrow] to be projected in Polar.
    /// tip_base_fraction: fraction of r base as unitary fraction.
    /// tip_angle_deg: angle with base in degrees
    pub fn new(
        mode: Mode,
        rho: f64,
        theta: f64,
        hover_text: String,
        visible: bool,
        tip_base_fraction: f64,
        tip_angle_deg: f64,
    ) -> Self {
        let (tip_left_rho, tip_left_theta) =
            (rho * (1.0 - tip_base_fraction), theta + tip_angle_deg);
        let (tip_right_rho, tip_right_theta) =
            (rho * (1.0 - tip_base_fraction), theta - tip_angle_deg);
        Self {
            scatter: {
                ScatterPolar::new(
                    vec![0.0, theta, tip_left_theta, theta, tip_right_theta],
                    vec![0.0, rho, tip_left_rho, rho, tip_right_rho],
                )
                .mode(mode)
                .web_gl_mode(true)
                .hover_text_array(vec![hover_text])
                .hover_info(HoverInfo::All)
                .visible({
                    if visible {
                        Visible::True
                    } else {
                        Visible::LegendOnly
                    }
                })
                .connect_gaps(false)
            },
        }
    }
}

pub struct Plot {
    /// [Plotly]
    plotly: Plotly,
    /// html (div) id
    plot_id: String,
}

impl Render for Plot {
    fn render(&self) -> Markup {
        html! {
            div {
                (PreEscaped (self.plotly.to_inline_html(None)))
            }
        }
    }
}

impl Plot {
    /// Adds one [Trace] to self
    pub fn add_trace(&mut self, t: Box<dyn Trace>) {
        self.plotly.add_trace(t);
    }
    /// Builds new standardized 1D Time domain plot
    pub fn timedomain_plot(
        plot_id: &str,
        title: &str,
        y_axis_label: &str,
        show_legend: bool,
    ) -> Self {
        let mut buttons = Vec::<SelectorButton>::new();
        for (step, count, label) in [
            (SelectorStep::All, 1, "all"),
            (SelectorStep::Second, 10, "10s"),
            (SelectorStep::Second, 30, "30s"),
            (SelectorStep::Minute, 1, "1min"),
            (SelectorStep::Hour, 1, "1hr"),
            (SelectorStep::Hour, 4, "4hr"),
            (SelectorStep::Day, 1, "1day"),
            (SelectorStep::Month, 1, "1mon"),
        ] {
            buttons.push(
                SelectorButton::new().count(count).label(label).step(step), //.step_mode(StepMode::ToDate/Backward)
            );
        }
        let layout = Layout::new()
            .title(title)
            .x_axis(
                Axis::new()
                    .title("MJD (UTC)")
                    .zero_line(true)
                    .show_tick_labels(true)
                    .dtick(0.25)
                    .range_slider(RangeSlider::new().visible(true))
                    .range_selector(RangeSelector::new().buttons(buttons))
                    .tick_format("{:05}"),
            )
            .y_axis(Axis::new().title(y_axis_label).zero_line(true))
            .show_legend(show_legend)
            .auto_size(true);
        let mut plotly = Plotly::new();
        plotly.set_layout(layout);
        Self {
            plotly,
            plot_id: plot_id.to_string(),
        }
    }
    /// Builds new 3D plot
    pub fn plot_3d(
        plot_id: &str,
        title: &str,
        x_label: &str,
        y_label: &str,
        z_label: &str,
        show_legend: bool,
    ) -> Self {
        let layout = Layout::new()
            .title(title)
            .x_axis(
                Axis::new()
                    .title(x_label)
                    .zero_line(true)
                    .show_tick_labels(false),
            )
            .y_axis(
                Axis::new()
                    .title(y_label)
                    .zero_line(true)
                    .show_tick_labels(false),
            )
            .z_axis(
                Axis::new()
                    .title(z_label)
                    .zero_line(true)
                    .show_tick_labels(false),
            )
            .auto_size(true)
            .show_legend(show_legend);
        let mut plotly = Plotly::new();
        plotly.set_layout(layout);
        Self {
            plotly,
            plot_id: plot_id.to_string(),
        }
    }
    /// Builds new Skyplot
    pub fn sky_plot(plot_id: &str, title: &str, show_legend: bool) -> Self {
        Self::polar_plot(
            plot_id,
            title,
            "Elevation (Deg°)",
            "Azimuth (Deg°)",
            show_legend,
        )
    }
    /// Trace for a skyplot
    pub fn sky_trace<T: Default + Clone + Serialize>(
        t: Vec<Epoch>,
        rho: Vec<T>,
        theta: Vec<T>,
        visible: bool,
    ) -> Box<ScatterPolar<T, T>> {
        let txt = t.iter().map(|t| t.to_string()).collect::<Vec<_>>();
        ScatterPolar::new(theta, rho)
            .web_gl_mode(true)
            .hover_text_array(txt)
            .hover_info(HoverInfo::All)
            .visible({
                if visible {
                    Visible::True
                } else {
                    Visible::LegendOnly
                }
            })
            .connect_gaps(false)
        //TODO alpha gradient per time
    }
    /// Builds new Polar plot
    pub fn polar_plot(
        plot_id: &str,
        title: &str,
        x_label: &str,
        y_label: &str,
        show_legend: bool,
    ) -> Self {
        let mut plotly = Plotly::new();
        let layout = Layout::new()
            .title(title)
            .x_axis(Axis::new().title(x_label).zero_line(true))
            .y_axis(Axis::new().title(y_label).zero_line(true))
            .show_legend(show_legend)
            .auto_size(true);
        Self {
            plotly,
            plot_id: plot_id.to_string(),
        }
    }
    /// Builds new World Map
    pub fn world_map(
        plot_id: &str,
        title: &str,
        map_style: MapboxStyle,
        center_ddeg: (f64, f64),
        zoom: u8,
        show_legend: bool,
    ) -> Self {
        let layout = Layout::new()
            .title(title)
            .drag_mode(DragMode::Zoom)
            .margin(Margin::new().top(0).left(0).bottom(0).right(0))
            .show_legend(show_legend)
            .mapbox(
                Mapbox::new()
                    .style(map_style)
                    .center(Center::new(center_ddeg.0, center_ddeg.1))
                    .zoom(zoom),
            );
        let mut plotly = Plotly::new();
        plotly.set_layout(layout);
        Self {
            plotly,
            plot_id: plot_id.to_string(),
        }
    }
    /// Builds new Mapbox trace
    pub fn mapbox<T: Clone + Default + Serialize>(
        lat: Vec<T>,
        lon: Vec<T>,
        legend: &str,
        symbol: MarkerSymbol,
        color: NamedColor,
        opacity: f64,
        visible: bool,
    ) -> Box<ScatterMapbox<T, T>> {
        ScatterMapbox::new(lat, lon)
            .marker(
                Marker::new()
                    .size(3)
                    .symbol(symbol)
                    .color(color)
                    .opacity(opacity),
            )
            .name(legend)
            .visible({
                if visible {
                    Visible::True
                } else {
                    Visible::LegendOnly
                }
            })
    }
    /// Builds ScatterGeo
    pub fn scattergeo<T: Clone + Default + Serialize>(
        lat: Vec<T>,
        lon: Vec<T>,
        legend: &str,
    ) -> Box<ScatterGeo<T, T>> {
        ScatterGeo::new(lat, lon)
    }
    /// Builds new Density Mapbox trace
    pub fn density_mapbox<T: Clone + Default + Serialize>(
        lat: Vec<T>,
        lon: Vec<T>,
        z: Vec<T>,
        legend: &str,
        opacity: f64,
        zoom: u8,
        visible: bool,
    ) -> Box<DensityMapbox<T, T, T>> {
        DensityMapbox::new(lat, lon, z)
            .name(legend)
            .opacity(opacity)
            .zauto(true)
            .zoom(zoom)
            .visible({
                if visible {
                    Visible::True
                } else {
                    Visible::LegendOnly
                }
            })
    }
    /// Builds new 3D chart
    pub fn chart_3d<T: Clone + Default + Serialize>(
        name: &str,
        mode: Mode,
        symbol: MarkerSymbol,
        t: &Vec<Epoch>,
        x: Vec<T>,
        y: Vec<T>,
        z: Vec<T>,
    ) -> Box<Scatter3D<T, T, T>> {
        let txt = t.iter().map(|t| t.to_string()).collect::<Vec<_>>();
        Scatter3D::new(x, y, z)
            .mode(mode)
            .name(name)
            .hover_text_array(txt)
            .hover_info(HoverInfo::All)
            .marker(Marker::new().symbol(symbol))
    }
    /// Builds new Time domain chart
    pub fn timedomain_chart<Y: Clone + Default + Serialize>(
        name: &str,
        mode: Mode,
        symbol: MarkerSymbol,
        t: &Vec<Epoch>,
        y: Vec<Y>,
    ) -> Box<Scatter<f64, Y>> {
        let txt = t.iter().map(|t| t.to_string()).collect::<Vec<_>>();
        Scatter::new(t.iter().map(|t| t.to_mjd_utc_days()).collect(), y)
            .name(name)
            .mode(mode)
            .web_gl_mode(true)
            .hover_text_array(txt)
            .hover_info(HoverInfo::All)
            .marker(Marker::new().symbol(symbol))
    }
}
