use plotly::Plot as Plotly;
use plotly::common::Layout;

pub struct Plot {
    plotly: Plotly,
    html_id: String,
}

impl Plot {
    pub fn new_timedomain(html_id: &str, title: &str, y_axis_label: &str, show_legend: bool) -> Self {
        let layout = Layout::new()
            .title(Title::new(title)
            .x_axis(
                Axis::new()
                    .title(Title::new("MJD"))
                    .zero_line(true)
                    .show_tick_labels(true)
                    .dtick(0.1)
                    .tick_format("{:05}")
            )
            .y_axis(
                Axis::new()
                    .title(Title::new(y_axis_label))
                    .zero_line(true)
            )
            .show_legend(show_legend)
            .auto_size(true);
        let mut plotly = Plotly::new();
        plotly.set_layout(layout);
        Self {
            plotly,
            html_id: html_id.to_string(),
        }
    }
}

impl RenderHtml for Plot {
    fn to_inline_html(&self) -> Box<RenderBox + '_> {
        box_html! {
            div(class="plot", id=&self.html_id) {
                : self.plotly.to_inline_html()
            }
        }
    }
}
