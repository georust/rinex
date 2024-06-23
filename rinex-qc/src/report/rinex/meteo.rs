use itertools::Itertools;
use maud::{html, Markup, Render};
use qc_traits::processing::{Filter, FilterItem, MaskOperand, Preprocessing};
use rinex::{
    meteo::sensor::Sensor,
    prelude::{Observable, Rinex},
};
use std::collections::HashMap;
use std::f64::consts::PI;

use crate::report::{shared::SamplingReport, Error};

#[cfg(feature = "plot")]
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

struct WindDirectionReport {
    #[cfg(feature = "plot")]
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
    observable: String,
    #[cfg(feature = "plot")]
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
    #[cfg(feature = "plot")]
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
            let mut compass_plot =
                Plot::new_time_domain(&html_id, "Wind Direction", "Angle [°]", true);
            for (index, (t, observations)) in rnx.meteo().enumerate() {
                let visible = index > 0;
                for (ob, value) in observations.iter() {
                    if *ob == Observable::WindDirection {
                        let hover_text = t.to_string();
                        let mut rho = 1.0;
                        for (rhs_ob, rhs_value) in observations.iter() {
                            if *rhs_ob == Observable::WindSpeed {
                                rho = *rhs_value;
                            }
                        }
                        let theta = value; //value * 2.0 * PI / value;
                        let trace = CompassArrow::new(
                            Mode::LinesMarkers,
                            rho,
                            *value,
                            hover_text,
                            visible,
                            0.25,
                            25.0,
                        );
                        compass_plot.add_trace(trace.scatter);
                    }
                }
            }
            let report = WindDirectionReport { compass_plot };
            Self {
                sampling: SamplingReport::from_rinex(rnx),
                inner: ObservableDependent::WindDirection(report),
            }
        } else {
            let mut plot = Plot::new_time_domain(&html_id, &title, &y_label, true);
            let data_x =
                rnx.meteo()
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
                Mode::LinesMarkers,
                MarkerSymbol::TriangleUp,
                &data_x,
                data_y,
            );
            plot.add_trace(trace);
            let report = SinglePlotReport {
                observable: observable.to_string(),
                plot,
            };
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
        Ok(Self {
            agency: None,
            sensors: header.sensors.clone(),
            sampling: SamplingReport::from_rinex(&rnx),
            pages: {
                let mut pages = HashMap::<String, MeteoPage>::new();
                for observable in rnx.observable() {
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
                            th class="is-info" {
                                "Sampling"
                            }
                            td {
                                (self.sampling.render())
                            }
                        }
                    }
                }
            }//table
            @for key in self.pages.keys().sorted() {
                @if let Some(page) = self.pages.get(key) {
                    div class="table-container is-page" id=(format!("meteo:{}", key)) style="display:block" {
                        table class="table is-bordered" {
                            tr {
                                th class="is-info" {
                                    (key.to_string())
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
