use crate::header::Header;
use maud::{html, Markup, Render};

#[cfg(feature = "qc")]
impl Render for Header {
    fn render(&self) -> Markup {
        html! {
            tr {
                th { "Antenna" }
                @if let Some(antenna) = &self.rcvr_antenna {
                    td { (antenna.render()) }
                } @else {
                    td { "No information" }
                }
            }
            tr {
                th { "Receiver" }
                @ if let Some(rcvr) = &self.rcvr {
                    td { (rcvr.render()) }
                } else {
                    td { "No information" }
                }
            }
        }
    }
}
