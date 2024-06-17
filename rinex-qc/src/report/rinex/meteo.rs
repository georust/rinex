use itertools::Itertools;
use qc_traits::html::*;
use qc_traits::processing::{Filter, FilterItem, MaskOperand, Preprocessing};
use rinex::{
    meteo::sensor::Sensor,
    prelude::{Observable, Rinex},
};
use std::collections::HashMap;

use crate::report::{shared::SamplingReport, Error};

#[cfg(feature = "plot")]
use crate::plot::{MarkerSymbol, Mode, Plot};

fn obs2physics(ob: &Observable) -> String {
    match ob {
        Observable::Pressure => "Pressure".to_string(),
        Observable::Temperature => "Temperature".to_string(),
        Observable::HumidityRate => "Moisture".to_string(),
        Observable::ZenithWetDelay => "Wet Delay".to_string(),
        Observable::ZenithDryDelay => "Dry Delay".to_string(),
        Observable::ZenithTotalDelay => "Wet+Dry Delay".to_string(),
        Observable::WindDirection => "Wind Direction".to_string(),
        Observable::WindSpeed => "Wind Speed".to_string(),
        Observable::RainIncrement => "Rain Increment".to_string(),
        Observable::HailIndicator => "Hail".to_string(),
        _ => "Not applicable".to_string(),
    }
}

fn obs2unit(ob: &Observable) -> String {
    match ob {
        Observable::Pressure => "hPa".to_string(),
        Observable::Temperature => "°C".to_string(),
        Observable::HumidityRate | Observable::RainIncrement => "%".to_string(),
        Observable::ZenithWetDelay | Observable::ZenithDryDelay | Observable::ZenithTotalDelay => {
            "s".to_string()
        },
        Observable::WindDirection => "°".to_string(),
        Observable::WindSpeed => "m/s".to_string(),
        Observable::HailIndicator => "boolean".to_string(),
        _ => "not applicable".to_string(),
    }
}

pub struct MeteoPage {
    #[cfg(feature = "plot")]
    plot: Plot,
    sampling: SamplingReport,
}

impl RenderHtml for MeteoPage {
    fn to_inline_html(&self) -> Box<dyn RenderBox + '_> {
        box_html! {
            div(class="table-container") {
                table(class="table is-bordered") {
                    tbody {
                        tr {
                            th(class="is-info") {
                                : "Sampling"
                            }
                            td {
                                : self.sampling.to_inline_html()
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Meteo RINEX analysis
pub struct MeteoReport {
    sensors: Vec<Sensor>,
    agency: Option<String>,
    sampling: SamplingReport,
    pages: HashMap<String, MeteoPage>,
}

impl MeteoReport {
    pub fn html_inline_menu_bar(&self) -> Box<dyn RenderBox + '_> {
        box_html! {
            a(id="menu:meteo") {
                span(class="icon") {
                    i(class="fa-solid fa-cloud-sun-rain");
                }
                : "Meteo Observations"
            }
            //ul(class="menu-list", id="menu:tabs:meteo", style="display:none") {
            //    @ for page in self.pages.keys().sorted() {
            //        li {
            //            a(id=&format!("menu:meteo:{}", page), style="margin-left:29px") {
            //                span(class="icon") {
            //                    i(class="fa-solid fa-cloud-sun-rain");
            //                }
            //                : page.to_string()
            //            }
            //        }
            //    }
            //}
        }
    }
    pub fn new(rnx: &Rinex) -> Result<Self, Error> {
        let header = rnx.header.meteo.as_ref().ok_or(Error::MissingMeteoHeader)?;
        Ok(Self {
            agency: None,
            sensors: header.sensors.clone(),
            sampling: SamplingReport::from_rinex(&rnx),
            pages: {
                let mut pages = HashMap::<String, MeteoPage>::new();
                for observable in rnx.observable() {
                    let filter = Filter::mask(
                        MaskOperand::Equals,
                        FilterItem::ComplexItem(vec![observable.to_string()]),
                    );
                    let focused = rnx.filter(&filter);
                    let title = format!("{} Observations", observable);
                    let y_label = format!("{} [{}]", observable, obs2unit(observable));
                    let html_id = observable.to_string();
                    let mut plot = if *observable == Observable::WindDirection {
                        unimplemented!("meteo:: wind direction plot");
                    } else {
                        Plot::new_time_domain(&html_id, &title, &y_label, true)
                    };
                    let data_x = rnx
                        .meteo()
                        .flat_map(|(t, observations)| {
                            observations.iter().filter_map(|(obs, _)| {
                                if obs == observable {
                                    Some(*t)
                                } else {
                                    None
                                }
                            })
                        })
                        .collect::<Vec<_>>();
                    let data_y = rnx
                        .meteo()
                        .flat_map(|(_, observations)| {
                            observations.iter().filter_map(|(obs, value)| {
                                if obs == observable {
                                    Some(*value)
                                } else {
                                    None
                                }
                            })
                        })
                        .collect::<Vec<_>>();
                    let trace = Plot::new_timedomain_chart(
                        &html_id,
                        Mode::LinesMarkers,
                        MarkerSymbol::TriangleUp,
                        &data_x,
                        data_y,
                    );
                    plot.add_trace(trace);

                    pages.insert(
                        obs2physics(observable),
                        MeteoPage {
                            plot,
                            sampling: SamplingReport::from_rinex(&focused),
                        },
                    );
                }
                pages
            },
        })
    }
}

impl RenderHtml for MeteoReport {
    fn to_inline_html(&self) -> Box<dyn RenderBox + '_> {
        box_html! {
            div(class="table-container") {
                table(class="table is-bordered") {
                    tbody {
                        tr {
                            th(class="is-info") {
                                : "Agency"
                            }
                            @ if let Some(agency) = &self.agency {
                                td {
                                    : agency
                                }
                            } else {
                                td {
                                    : "Unknown"
                                }
                            }
                        }
                        @ for sensor in self.sensors.iter() {
                            tr {
                                th {
                                    : &format!("{} sensor", obs2physics(&sensor.observable))
                                }
                                td {
                                    : sensor.to_inline_html()
                                }
                            }
                        }
                        tr {
                            th(class="is-info") {
                                : "Sampling"
                            }
                            td {
                                : self.sampling.to_inline_html()
                            }
                        }
                    }
                }
            }//table
            @ for constell in self.pages.keys().sorted() {
                @ if let Some(page) = self.pages.get(constell) {
                    div(class="table-container is-page", id=&format!("meteo:{}", constell), style="display:block") {
                        table(class="table is-bordered") {
                            tr {
                                th(class="is-info") {
                                    : constell.to_string()
                                }
                                td {
                                    : page.to_inline_html()
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
