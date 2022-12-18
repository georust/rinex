use super::prelude::*;
use horrorshow::{helper::doctype, RenderBox};
use strum_macros::EnumString;

mod averager;
mod sampling;
//mod advanced;
//mod navigation;
mod observation;

mod opts;
pub use opts::QcOpts;

#[derive(Debug, Clone, PartialEq, PartialOrd, Hash, Eq, EnumString)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Grade {
    #[strum(serialize = "A++")]
    GradeApp,
    #[strum(serialize = "A+")]
    GradeAp,
    #[strum(serialize = "A")]
    GradeA,
    #[strum(serialize = "B")]
    GradeB,
    #[strum(serialize = "C")]
    GradeC,
    #[strum(serialize = "D")]
    GradeD,
    #[strum(serialize = "E")]
    GradeE,
    #[strum(serialize = "F")]
    GradeF,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct QcReport {
    // stored header, for further informations
    header: Header,
    /// Sampling QC
    pub sampling: sampling::QcReport,
    /// Observation RINEX specific QC
    pub observation: Option<observation::QcReport>,
}

impl QcReport {
    pub fn basic(rnx: &Rinex, nav: &Option<Rinex>) -> Self {
        Self::new(rnx, nav, None)
    }
    /// Builds a new QC Report
    pub fn new(rnx: &Rinex, nav: &Option<Rinex>, opts: Option<QcOpts>) -> Self {
        let opts = opts.unwrap_or(QcOpts::default());
        Self {
            header: rnx.header.clone(),
            sampling: sampling::QcReport::new(rnx),
            observation: {
                if rnx.is_observation_rinex() {
                    Some(observation::QcReport::new(rnx, nav, opts))
                } else {
                    None
                }
            },
        }
    }
    /// Dumps self into (self sufficient) HTML
    pub fn to_html(&self) -> String {
        format!(
            "{}",
            html! {
                : doctype::HTML;
                html {
                    head {
                        meta(charset="utf-8");
                        title: "RINEX QC summary";
                        //to include JS:
                        //script: Raw(include_str!("test.js"));
                        //to include CSS (one option..)
                        //style: Raw(include_str!("test.css"));
                        style {
                            table {
                                font-family: "arial, sans-serif";
                                border-collapse: "collapse";
                                width: "100%";
                            }
                            td {
                                border: "1px solid #dddddd";
                                text-align: "left";
                                padding: "8px";
                            }
                            th {
                                border: "1px solid #dddddd";
                                text-align: "left";
                                padding: "8px";
                            }
                            /*tr:nth-child(event) {
                                background-color: "#dddddd";
                            }*/
                        }
                    }
                    body {
                        : self.to_inline_html()
                    }
                }
            }
        )
    }
    /// Dumps self into HTML <div> section, named as suggested
    pub fn to_inline_html(&self) -> Box<dyn RenderBox + '_> {
        box_html! {
            h2(id="heading") {
                : "RINEX Quality Check summary"
            }
            h4(id="version") {
                program-version: format!("rust-rnx: v{}", env!("CARGO_PKG_VERSION"))
            }
            div(id="general") {
                div(id="antenna") {
                    table {
                        tr {
                            th {
                                : "Antenna model"
                            }
                            th {
                                : "SN#"
                            }
                        }
                        tr {
                            @if let Some(ant) = &self.header.rcvr_antenna {
                                td {
                                    : ant.model.clone()
                                }
                                td {
                                    : ant.sn.clone()
                                }
                            } else {
                                td {
                                    : "Unknown"
                                }
                            }
                        }
                    }
                }//div=antenna
                div(id="rcvr") {
                    table {
                        tr {
                            th {
                                : "Receiver model"
                            }
                            th {
                                : "SN#"
                            }
                            th {
                                : "Firmware"
                            }
                        }
                        tr {
                            @ if let Some(rcvr) = &self.header.rcvr {
                                td {
                                    : rcvr.model.clone()
                                }
                                td {
                                    : rcvr.sn.clone()
                                }
                                td {
                                    : rcvr.firmware.clone()
                                }
                            } else {
                                td {
                                    : "Unknown"
                                }
                            }
                        }
                    }
                }//div="rcvr"
                div(id="ground-pos") {
                    table {
                        tr {
                            th {
                                : "Ground Position (ECEF)"
                            }
                            td {
                                @ if let Some((pos_x,pos_y,pos_z)) = &self.header.coords {
                                    : format!("{}, {}, {}", pos_x, pos_y, pos_z)
                                } else {
                                    : "Unknown"
                                }
                            }
                        }

                    }
                }//div="ground-pos"
            }//div=general
            div(id="sampling") {
                table {
                    tr {
                        th {
                            : "Sampling"
                        }
                    }
                    : self.sampling.to_inline_html()
                }
            }
            @ if let Some(observation) = &self.observation {
                div(id="observations", style="max-width:350px;") {
                    table {
                        tr {
                            th {
                                : "Observations"
                            }
                        }
                        : observation.to_inline_html()
                    }
                }
            }
        }
    }
}
