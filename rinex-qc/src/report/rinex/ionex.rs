use crate::report::Error;
use maud::{html, Markup, Render};
use rinex::ionex::{MappingFunction, RefSystem as Reference};
use rinex::prelude::{Duration, Epoch, Rinex};

use crate::plot::{MapboxStyle, Plot, Visible};

use plotly::{
    layout::update_menu::{Button, ButtonBuilder},
    DensityMapbox,
};

pub struct IonexReport {
    nb_of_maps: usize,
    map_dimension: u8,
    epoch_first_map: Epoch,
    epoch_last_map: Epoch,
    sampling_interval: Option<Duration>,
    reference: Reference,
    description: Option<String>,
    mapping: Option<MappingFunction>,
    world_map: Plot,
}

impl IonexReport {
    pub fn new(rnx: &Rinex) -> Result<Self, Error> {
        let nb_of_maps = rnx.epoch_iter().count();
        let header = rnx.header.ionex.as_ref().ok_or(Error::MissingIonexHeader)?;
        Ok(Self {
            nb_of_maps,
            map_dimension: header.map_dimension,
            epoch_last_map: header.epoch_of_last_map,
            epoch_first_map: header.epoch_of_first_map,
            mapping: header.mapping.clone(),
            reference: header.reference.clone(),
            description: header.description.clone(),
            sampling_interval: rnx.sampling_interval(),
            world_map: {
                let mut plot = Plot::world_map(
                    "ionex_tec",
                    "Ionosphere TEC maps",
                    MapboxStyle::OpenStreetMap,
                    (32.5, -40.0),
                    0,
                    true,
                );

                // Build one trace (1 map) per Epoch
                let mut buttons = Vec::<Button>::new();

                for (epoch_index, epoch) in rnx.epoch_iter().enumerate() {
                    let label = epoch.to_string();

                    let lat = rnx
                        .ionex_tecu_latlong_ddeg_alt_km_iter()
                        .filter_map(
                            |(t, _, lat, _, _)| {
                                if t == epoch {
                                    Some(lat)
                                } else {
                                    None
                                }
                            },
                        )
                        .collect::<Vec<_>>();

                    let long = rnx
                        .ionex_tecu_latlong_ddeg_alt_km_iter()
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

                    let tecu = rnx
                        .ionex_tecu_latlong_ddeg_alt_km_iter()
                        .filter_map(
                            |(t, tecu, _, _, _)| {
                                if t == epoch {
                                    Some(tecu)
                                } else {
                                    None
                                }
                            },
                        )
                        .collect::<Vec<_>>();

                    let trace = Plot::density_mapbox(
                        lat.clone(),
                        long.clone(),
                        tecu,
                        &label,
                        0.6,
                        3,
                        epoch_index == 0,
                    );

                    plot.add_trace(trace);

                    buttons.push(
                        ButtonBuilder::new()
                            .name("Epoch")
                            .label(&label)
                            .push_restyle(DensityMapbox::<f64, f64, f64>::modify_visible(
                                (0..nb_of_maps)
                                    .map(|i| {
                                        if epoch_index == i {
                                            Visible::True
                                        } else {
                                            Visible::False
                                        }
                                    })
                                    .collect(),
                            ))
                            .build(),
                    );
                }

                plot.add_custom_controls(buttons);
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
                            button aria-label="Isosurface TEC maps" data-balloon-pos="right" {
                                "2D IONEX"
                            }
                        }
                    } @else {
                        th class="is-info" {
                            button aria-label="Isofurface TEC maps by altitude layers" data-balloon-pos="right" {
                                "3D IONEX"
                            }
                        }
                    }
                }
                tr {
                    th class="is-info" {
                        "Number of Maps"
                    }
                    td {
                        (self.nb_of_maps)
                    }
                }
                tr {
                    th class="is-info" {
                        "Sampling Period"
                    }
                    @ if let Some(sampling_interval) = &self.sampling_interval {
                        td {
                            (sampling_interval.to_string())
                        }
                    } @ else {
                        td class="is-warning" {
                            "Unknown"
                        }
                    }
                }
                tr {
                    th class="is-info" {
                        "Epoch of first map"
                    }
                    td {
                        (self.epoch_first_map.to_string())
                    }
                }
                tr {
                    th class="is-info" {
                        "Epoch of Last map"
                    }
                    td {
                        (self.epoch_last_map.to_string())
                    }
                }
                tr {
                    th class="is-info" {
                        "Reference"
                    }
                    td {
                        (self.reference.to_string())
                    }
                }
                @if let Some(desc) = &self.description {
                    tr {
                        th class="is-info" {
                            "Description"
                        }
                        td {
                            (desc)
                        }
                    }
                }
                @if let Some(mapf) = &self.mapping {
                    tr {
                        th class="is-info" {
                            button aria-label="Mapping function used in TEC map evaluation" data-balloon-pos="right" {
                                "Mapping function"
                            }
                        }
                        td {
                            (mapf.to_string())
                        }
                    }
                }
                tr {
                    th class="is-info" {
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
