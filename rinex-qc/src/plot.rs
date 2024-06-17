use hifitime::Epoch;
use plotly::{
    common::{Font, HoverInfo, Marker, Side, Title},
    layout::{Axis, Center, DragMode, Mapbox, MapboxStyle, Margin},
    Layout, Plot as Plotly, Scatter, Trace,
};
use qc_traits::html::*;

pub use plotly::common::{MarkerSymbol, Mode};

use serde::Serialize;

pub struct Plot {
    plotly: Plotly,
    plot_id: String,
}

impl Plot {
    /// Adds one [Trace] to self
    pub fn add_trace(&mut self, t: Box<dyn Trace>) {
        self.plotly.add_trace(t);
    }
    /// Builds new standardized 1D Time domain plot
    pub fn new_time_domain(
        plot_id: &str,
        title: &str,
        y_axis_label: &str,
        show_legend: bool,
    ) -> Self {
        let layout = Layout::new()
            .title(Title::new(title))
            .x_axis(
                Axis::new()
                    .title(Title::new("MJD"))
                    .zero_line(true)
                    .show_tick_labels(true)
                    .dtick(0.1)
                    .tick_format("{:05}"),
            )
            .y_axis(Axis::new().title(Title::new(y_axis_label)).zero_line(true))
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
    pub fn new_3d_plot(
        plot_id: &str,
        title: &str,
        x_label: &str,
        y_label: &str,
        z_label: &str,
        show_legend: bool,
    ) -> Self {
        let layout = Layout::new()
            .title(Title::new(title).side(Side::Top))
            .x_axis(
                Axis::new()
                    .title(Title::new(x_label))
                    .zero_line(true)
                    .show_tick_labels(false),
            )
            .y_axis(
                Axis::new()
                    .title(Title::new(y_label))
                    .zero_line(true)
                    .show_tick_labels(false),
            )
            .z_axis(
                Axis::new()
                    .title(Title::new(z_label))
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
    pub fn sky_plot(plot_id: &str, title: &str) -> Self {
        Self::new_polar(plot_id, title, "Elevation [°]", "Azimuth [°]")
    }
    /// Builds new Polar plot
    pub fn new_polar(plot_id: &str, title: &str, x_label: &str, y_label: &str) -> Self {
        let mut plotly = Plotly::new();
        let layout = Layout::new()
            .title(Title::new(title))
            .x_axis(
                Axis::new()
                    .title(Title::new(x_label).side(Side::Top))
                    .zero_line(true),
            )
            .y_axis(Axis::new().title(Title::new(y_label)).zero_line(true))
            .show_legend(true)
            .auto_size(true);
        Self {
            plotly,
            plot_id: plot_id.to_string(),
        }
    }
    /// Builds new World Map
    pub fn new_world_map(
        plot_id: &str,
        title: &str,
        map_style: MapboxStyle,
        center_ddeg: (f64, f64),
        zoom: u8,
        show_legend: bool,
    ) -> Self {
        let layout = Layout::new()
            .title(Title::new(title).font(Font::default()))
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
    /// Builds new standardized Time domain chart, to stacked
    /// on a Time domain plot.
    pub fn new_timedomain_chart<Y: Clone + Default + Serialize>(
        chart_id: &str,
        mode: Mode,
        symbol: MarkerSymbol,
        t: &Vec<Epoch>,
        y: Vec<Y>,
    ) -> Box<Scatter<f64, Y>> {
        let txt = t.iter().map(|t| t.to_string()).collect::<Vec<_>>();
        Scatter::new(t.iter().map(|t| t.to_mjd_utc_days()).collect(), y)
            .mode(mode)
            .hover_text_array(txt)
            .hover_info(HoverInfo::All)
            .marker(Marker::new().symbol(symbol))
    }
}

impl RenderHtml for Plot {
    fn to_inline_html(&self) -> Box<dyn RenderBox + '_> {
        box_html! {
            div(class="plot", id=&self.plot_id) {
                : self.plotly.to_inline_html(None)
            }
        }
    }
}
