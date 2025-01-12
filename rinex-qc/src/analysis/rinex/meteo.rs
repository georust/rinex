use itertools::Itertools;
use maud::{html, Markup, Render};
use qc_traits::{Filter, FilterItem, MaskOperand, Preprocessing};
use rinex::{
    meteo::Sensor,
    prelude::{Observable, Rinex},
};
use std::collections::HashMap;

use crate::report::{shared::SamplingReport, Error};

use crate::plot::{CompassArrow, MarkerSymbol, Mode, Plot};

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
        Observable::HumidityRate => "%".to_string(),
        Observable::ZenithWetDelay | Observable::ZenithDryDelay | Observable::ZenithTotalDelay => {
            "s".to_string()
        },
        Observable::WindDirection => "°".to_string(),
        Observable::WindSpeed => "m/s".to_string(),
        Observable::HailIndicator => "boolean".to_string(),
        Observable::RainIncrement => "1/10 mm".to_string(),
        _ => "not applicable".to_string(),
    }
}

struct WindDirectionReport {
    compass_plot: Plot,
}

impl Render for WindDirectionReport {
    fn render(&self) -> Markup {
        html! {
            div class="table-container" {
                table class="table is-bordered" {
                    tbody {
                        tr {
                            td {
                                (self.compass_plot.render())
                            }
                        }
                    }
                }
            }
        }
    }
}

struct SinglePlotReport {
    plot: Plot,
}

impl Render for SinglePlotReport {
    fn render(&self) -> Markup {
        html! {
            (self.plot.render())
        }
    }
}

enum ObservableDependent {
    SinglePlot(SinglePlotReport),
    WindDirection(WindDirectionReport),
}

impl Render for ObservableDependent {
    fn render(&self) -> Markup {
        match self {
            Self::SinglePlot(plot) => plot.render(),
            Self::WindDirection(report) => report.render(),
        }
    }
}

struct MeteoPage {
    inner: ObservableDependent,
    sampling: SamplingReport,
}

impl MeteoPage {
    fn new(observable: &Observable, rnx: &Rinex) -> Self {
        let title = format!("{} Observations", observable);
        let y_label = format!("{} [{}]", observable, obs2unit(observable));
        let html_id = observable.to_string();

        if *observable == Observable::WindDirection {
            // special compass plot
            let mut compass_plot =
                Plot::timedomain_plot(&html_id, "Wind Direction", "Angle [°]", true);

            for (nth, (t, angle_degrees)) in rnx.wind_direction_iter().enumerate() {
                let visible = nth == 0;
                let hover_text = t.to_string();
                let trace = CompassArrow::new(
                    Mode::LinesMarkers,
                    angle_degrees,
                    angle_degrees,
                    hover_text,
                    visible,
                    0.25,
                    25.0,
                );
                compass_plot.add_trace(trace.scatter);
            }
            let report = WindDirectionReport { compass_plot };
            Self {
                sampling: SamplingReport::from_rinex(rnx),
                inner: ObservableDependent::WindDirection(report),
            }
        } else {
            let mut plot = Plot::timedomain_plot(&html_id, &title, &y_label, true);

            let data_x = rnx
                .meteo_observations_iter()
                .filter_map(|(k, _)| {
                    if &k.observable == observable {
                        Some(k.epoch)
                    } else {
                        None
                    }
                })
                .collect();

            let data_y = rnx
                .meteo_observations_iter()
                .filter_map(|(k, v)| {
                    if &k.observable == observable {
                        Some(*v)
                    } else {
                        None
                    }
                })
                .collect();

            let trace = Plot::timedomain_chart(
                &observable.to_string(),
                Mode::Markers,
                MarkerSymbol::TriangleUp,
                &data_x,
                data_y,
                true,
            );
            plot.add_trace(trace);
            let report = SinglePlotReport { plot };
            Self {
                sampling: SamplingReport::from_rinex(rnx),
                inner: ObservableDependent::SinglePlot(report),
            }
        }
    }
}

impl Render for MeteoPage {
    fn render(&self) -> Markup {
        html! {
            div class="table-container" {
                table class="table is-bordered" {
                    tbody {
                        tr {
                            th class="is-info" {
                                "Sampling"
                            }
                            td {
                                (self.sampling.render())
                            }
                        }
                        tr {
                            th class="is-info" {
                                "Observations"
                            }
                            td {
                                (self.inner.render())
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
    hail: bool,
    rain: bool,
    total_rain_drop_mm: f64,
    pressure_min_max: (f64, f64),
    temperature_min_max: (f64, f64),
    sampling: SamplingReport,
    pages: HashMap<String, MeteoPage>,
}

impl MeteoReport {
    pub fn html_inline_menu_bar(&self) -> Markup {
        html! {
            a id="menu:meteo" {
                span class="icon" {
                    i class="fa-solid fa-cloud-sun-rain" {}
                }
                "Meteo Observations"
            }
        }
    }
    pub fn new(rnx: &Rinex) -> Result<Self, Error> {
        let header = rnx.header.meteo.as_ref().ok_or(Error::MissingMeteoHeader)?;

        let mut pressure_min_max = Option::<(f64, f64)>::None;
        let mut temperature_min_max = Option::<(f64, f64)>::None;

        for (k, v) in rnx.meteo_observations_iter() {
            if k.observable == Observable::Temperature {
                if let Some((min, max)) = &mut temperature_min_max {
                    if v < min {
                        *min = *v;
                    }
                    if v > max {
                        *max = *v;
                    }
                } else {
                    temperature_min_max = Some((*v, *v));
                }
            } else if k.observable == Observable::Temperature {
                if let Some((min, max)) = &mut pressure_min_max {
                    if v < min {
                        *min = *v;
                    }
                    if v > max {
                        *max = *v;
                    }
                } else {
                    pressure_min_max = Some((*v, *v));
                }
            }
        }

        Ok(Self {
            agency: None,
            sensors: header.sensors.clone(),
            sampling: SamplingReport::from_rinex(&rnx),
            hail: rnx.hail_detected(),
            rain: rnx.rain_detected(),
            total_rain_drop_mm: rnx.total_accumulated_rain() * 10.0,
            pressure_min_max: pressure_min_max.unwrap_or_default(),
            temperature_min_max: temperature_min_max.unwrap_or_default(),
            pages: {
                let mut pages = HashMap::<String, MeteoPage>::new();

                for observable in rnx.observables_iter() {
                    let filter = if *observable == Observable::WindDirection {
                        Filter::mask(
                            MaskOperand::Equals,
                            FilterItem::ComplexItem(vec![observable.to_string(), "WS".to_string()]),
                        )
                    } else {
                        Filter::mask(
                            MaskOperand::Equals,
                            FilterItem::ComplexItem(vec![observable.to_string()]),
                        )
                    };

                    let focused = rnx.filter(&filter);
                    pages.insert(
                        obs2physics(observable),
                        MeteoPage::new(observable, &focused),
                    );
                }
                pages
            },
        })
    }
}

impl Render for MeteoReport {
    fn render(&self) -> Markup {
        html! {
            div class="table-container" {
                table class="table is-bordered" {
                    tbody {
                        tr {
                            th class="is-info" {
                                "Agency"
                            }
                            @if let Some(agency) = &self.agency {
                                td {
                                    (agency)
                                }
                            } @else {
                                td {
                                    "Unknown"
                                }
                            }
                        }
                        @for sensor in self.sensors.iter() {
                            tr {
                                th {
                                  (&format!("{} sensor", obs2physics(&sensor.observable)))
                                }
                                td {
                                    (sensor.render())
                                }
                            }
                        }
                        tr {
                            th {
                                "Temperature"
                            }
                            td {
                                (format!("min={:.1}°C  max={:.1}°C", self.temperature_min_max.0, self.temperature_min_max.1))
                            }
                        }
                        tr {
                            th {
                                "Pressure"
                            }
                            td {
                                (format!("min={:.1}hPa  max={:.1}hPa", self.pressure_min_max.0, self.pressure_min_max.1))
                            }
                        }
                        tr {
                            th {
                                "Rain"
                            }
                            td {
                                (self.rain)
                            }
                        }
                        @if self.rain {
                            tr {
                                th {
                                    "Rain (total drop)"
                                }
                                td {
                                    (self.total_rain_drop_mm)
                                }
                            }
                        }
                        tr {
                            th {
                                "Hail"
                            }
                            td {
                                (self.rain)
                            }
                        }
                        tr {
                            th {
                                "Hail"
                            }
                            td {
                                (self.hail)
                            }
                        }
                        tr {
                            th class="is-info" {
                                "Sampling"
                            }
                            td {
                                (self.sampling.render())
                            }
                        }
                        tr {
                            th class="is-info" {
                                "Observations"
                            }
                            td {
                                div class="table-container" {
                                    table class="table is-bordered" {
                                        @for key in self.pages.keys().sorted() {
                                            @if let Some(page) = self.pages.get(key) {
                                                tr {
                                                    th class="is-info" {
                                                        (format!("{} observations", key))
                                                    }
                                                    td {
                                                        (page.render())
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }//table
        }
    }
}
