use maud::{html, Markup, Render};
use serde::{Deserialize, Serialize};

use crate::cfg::QcConfigError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QcPreferedRover {
    Any,
    Prefered(String),
}

impl Default for QcPreferedRover {
    fn default() -> Self {
        Self::Any
    }
}

impl std::str::FromStr for QcPreferedRover {
    type Err = QcConfigError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();
        if trimmed.eq("*") {
            Ok(Self::Any)
        } else {
            Ok(Self::Prefered(trimmed.to_string()))
        }
    }
}

impl std::fmt::Display for QcPreferedRover {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Any => write!(f, "*"),
            Self::Prefered(rover) => write!(f, "{}", rover),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QcCustomRoverOpts {
    /// Manual RX position that will apply to the Rover/User data set specifically
    /// and not any other
    pub manual_rx_position: Option<(f64, f64, f64)>,
    /// Prefered rover, for which we will solve solutions
    pub prefered_rover: QcPreferedRover,
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
                                "RINEx"
                            }
                        }
                    }
                    tr {
                        th class="is-info" {
                            "Prefered Rover"
                        }
                        td {
                            (self.prefered_rover.to_string())
                        }
                    }
                }
            }
        }
    }
}
