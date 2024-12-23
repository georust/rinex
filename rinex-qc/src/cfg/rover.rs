use maud::{html, Markup, Render};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QcCustomRoverOpts {
    /// Manual RX position that will apply to the Rover/User data set specifically
    /// and not any other
    pub manual_rx_position: Option<(f64, f64, f64)>,
}

impl Render for QcCustomRoverOpts {
    fn render(&self) -> Markup {
        html! {
            div class="table-container" {
                table class="table is-bordered" {
                    tr {
                        th {
                            "Reference position"
                        }
                        @if let Some(manual) = self.manual_rx_position {
                            td {
                                "Manual (User Defined)"
                            }
                            td {
                                (format!("{:.3E}m {:.3E}m {:.3E}m", manual.0, manual.1, manual.2))
                            }
                        } else {
                            td {
                                "RINEX"
                            }
                        }
                    }
                }
            }
        }
    }
}
