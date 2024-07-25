use crate::report::Error;
use maud::{html, Markup, Render};
use rinex::ionex::{MappingFunction, RefSystem as Reference};
use rinex::prelude::{Duration, Epoch, Rinex};

#[cfg(feature = "plot")]
use crate::plot::{MapboxStyle, MarkerSymbol, NamedColor, Plot};

pub struct IonexReport {
    nb_of_maps: usize,
    map_dimension: u8,
    epoch_first_map: Epoch,
    epoch_last_map: Epoch,
    sampling_interval: Duration,
    reference: Reference,
    description: Option<String>,
    mapping: Option<MappingFunction>,
    #[cfg(feature = "plot")]
    world_map: Plot,
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
            #[cfg(feature = "plot")]
            world_map: {
                let mut plot = Plot::world_map(
                    "ionex_tec",
                    "Ionosphere TEC maps",
                    MapboxStyle::StamenTerrain,
                    (32.5, -40.0),
                    1,
                    true,
                );
                let grid_lat = Vec::<f64>::new();
                let grid_lon = Vec::<f64>::new();
                // plot grid
                let grid = Plot::mapbox(
                    grid_lat,
                    grid_lon,
                    "grid",
                    MarkerSymbol::Circle,
                    NamedColor::Black,
                    0.5,
                );
                //plot.add_trace(grid);

                // one trace(=map) per Epoch
                for (epoch_index, epoch) in rnx.epoch().enumerate() {
                    // build tec map
                    let lat = rnx
                        .tec()
                        .filter_map(
                            |(t, lat, _, _, _)| {
                                if t == epoch {
                                    Some(lat)
                                } else {
                                    None
                                }
                            },
                        )
                        .collect::<Vec<_>>();
                    let long = rnx
                        .tec()
                        .filter_map(
                            |(t, _, long, _, _)| {
                                if t == epoch {
                                    Some(long)
                                } else {
                                    None
                                }
                            },
                        )
                        .collect::<Vec<_>>();
                    let tec = rnx
                        .tec()
                        .filter_map(
                            |(t, _, _, _, tec)| {
                                if t == epoch {
                                    Some(tec)
                                } else {
                                    None
                                }
                            },
                        )
                        .collect::<Vec<_>>();
                    let label = epoch.to_string();
                    let trace =
                        Plot::density_mapbox(lat, long, tec, &label, 0.66, 3, epoch_index == 0);
                    //plot.add_trace(trace);
                }
                plot
            },
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
                tr {
                    th {
                        "TEC Map"
                    }
                }
                tr {
                    td {
                        (self.world_map.render())
                    }
                }
            }
        }
    }
}
