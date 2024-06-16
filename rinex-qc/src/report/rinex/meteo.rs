use crate::report::Error;
use qc_traits::html::*;
use rinex::{
    meteo::sensor::Sensor,
    prelude::{Observable, Rinex},
};

// TODO
pub struct MeteoReport {
    sensors: Vec<Sensor>,
    agency: Option<String>,
}

impl MeteoReport {
    pub fn new(rnx: &Rinex) -> Result<Self, Error> {
        let header = rnx.header.meteo.as_ref().ok_or(Error::MissingMeteoHeader)?;
        Ok(Self {
            agency: None,
            sensors: header.sensors.clone(),
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
                    }
                }
            }
        }
    }
}
