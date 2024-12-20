use maud::{html, Markup, Render};
use serde::{Deserialize, Serialize};

use rinex::prelude::GroundPosition;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QcCustomRoverOpts {
    /// Manual [GroundPosition] that will apply to the Rover/User data set specifically
    /// and not any other
    pub manual_reference: Option<GroundPosition>,
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
                        @if let Some(manual) = self.manual_reference {
                            td {
                                "Manual (User Defined)"
                            }
                            td {
                                (manual.render())
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
