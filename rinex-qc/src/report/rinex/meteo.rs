use qc_traits::html::*;
use qc_traits::processing::{Filter, FilterItem, MaskOperand, Preprocessing};
use rinex::{
    meteo::sensor::Sensor,
    prelude::{Observable, Rinex},
};
use std::collections::HashMap;

use crate::report::{shared::SamplingReport, Error};

pub struct MeteoPage {
    sampling: SamplingReport,
}

/// Meteo RINEX analysis
pub struct MeteoReport {
    sensors: Vec<Sensor>,
    agency: Option<String>,
    sampling: SamplingReport,
    pub pages: HashMap<String, MeteoPage>,
}

impl MeteoReport {
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
                    pages.insert(
                        obs2physics(observable),
                        MeteoPage {
                            sampling: SamplingReport::from_rinex(&focused),
                        },
                    );
                }
                pages
            },
        })
    }
}

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
            }
        }
    }
}
