use crate::report::Error;
use maud::{html, Markup, Render};
use rinex::ionex::{MappingFunction, RefSystem as Reference};
use rinex::prelude::{Duration, Epoch, Rinex};

pub struct IonexReport {
    nb_of_maps: usize,
    map_dimension: u8,
    epoch_first_map: Epoch,
    epoch_last_map: Epoch,
    sampling_interval: Duration,
    reference: Reference,
    description: Option<String>,
    mapping: Option<MappingFunction>,
}

impl IonexReport {
    pub fn new(rnx: &Rinex) -> Result<Self, Error> {
        let header = rnx.header.ionex.as_ref().ok_or(Error::MissingIonexHeader)?;
        Ok(Self {
            nb_of_maps: rnx.epoch().count(),
            epoch_first_map: header.epoch_of_first_map,
            epoch_last_map: header.epoch_of_last_map,
            sampling_interval: rnx.dominant_sample_rate().ok_or(Error::SamplingAnalysis)?,
            mapping: header.mapping.clone(),
            description: header.description.clone(),
            reference: header.reference.clone(),
            map_dimension: header.map_dimension,
        })
    }
    pub fn html_inline_menu_bar(&self) -> Markup {
        html! {
            a id="menu:ionex" {
                span class="icon" {
                    i class="fa-solid fa-earth-americas" {}
                }
                "Ionosphere Maps (IONEX)"
            }
        }
    }
}

impl Render for IonexReport {
    fn render(&self) -> Markup {
        html! {
            table class="table is-bordered" {
                tr {
                    @if self.map_dimension == 2 {
                        th class="is-info"{
                            "2D IONEX"
                        }
                    } @else {
                        th class="is-info" {
                            "3D IONEX"
                        }
                    }
                }
                tr {
                    th {
                        "Number of Maps"
                    }
                    td {
                        (self.nb_of_maps)
                    }
                }
                tr {
                    th {
                        "Epoch of first map"
                    }
                    td {
                        (self.epoch_first_map.to_string())
                    }
                }
                tr {
                    th {
                        "Epoch of Last map"
                    }
                    td {
                        (self.epoch_last_map.to_string())
                    }
                }
                tr {
                    th {
                        "Reference"
                    }
                    td {
                        (self.reference.to_string())
                    }
                }
                @if let Some(desc) = &self.description {
                    tr {
                        th {
                            "Description"
                        }
                        td {
                            (desc)
                        }
                    }
                }
                @if let Some(mapf) = &self.mapping {
                    tr {
                        th {
                            "Mapping function"
                        }
                        td {
                            (mapf.to_string())
                        }
                    }
                }
            }
        }
    }
}
