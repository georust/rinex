use crate::{
    context::{meta::ObsMetaData, obs::ObservationUniqueId, QcContext},
    prelude::Rinex,
};

use itertools::Itertools;
use std::str::FromStr;

pub enum Format {
    RINEx,
    CRINEx,
    GZipRINEx,
    GZipCRINEx,
}

impl std::fmt::Display for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RINEx => f.write_str("RINEx"),
            Self::CRINEx => f.write_str("CRINEx"),
            Self::GZipRINEx => f.write_str("RINEx + gzip"),
            Self::GZipCRINEx => f.write_str("CRINEx + gzip"),
        }
    }
}

#[derive(Default)]
pub enum RxPositionSource {
    #[default]
    RINEx,
    UserDefined,
}

impl std::fmt::Display for RxPositionSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RINEx => write!(f, "RINEx"),
            Self::UserDefined => write!(f, "User Defined"),
        }
    }
}

pub struct RxPosition {
    pub source: RxPositionSource,
    pub ecef_km: (f64, f64, f64),
    pub geodetic: (f64, f64, f64),
}

pub struct QcObservationSummary {
    pub name: String,
    pub format: Format,
    pub rx_position: Option<RxPosition>,
    pub designator: Option<ObservationUniqueId>,
}

impl QcObservationSummary {
    pub fn new(ctx: &QcContext, obs_meta: &ObsMetaData, rinex: &Rinex) -> Self {
        Self {
            name: obs_meta.meta.name.to_string(),
            format: {
                let gzip = obs_meta.meta.extension.contains("gz");
                if rinex.header.is_crinex() {
                    if gzip {
                        Format::GZipCRINEx
                    } else {
                        Format::CRINEx
                    }
                } else {
                    if gzip {
                        Format::GZipRINEx
                    } else {
                        Format::RINEx
                    }
                }
            },
            designator: if let Some(unique_id) = &obs_meta.meta.unique_id {
                let designator = ObservationUniqueId::from_str(&unique_id).unwrap();
                Some(designator)
            } else {
                None
            },
            rx_position: {
                if obs_meta.is_rover {
                    if let Some(orbit) = ctx.rover_rx_orbit(&obs_meta.meta) {
                        let pos_vel = orbit.to_cartesian_pos_vel();
                        match orbit.latlongalt() {
                            Ok(geodetic) => Some(RxPosition {
                                source: RxPositionSource::RINEx,
                                ecef_km: (pos_vel[0], pos_vel[1], pos_vel[2]),
                                geodetic,
                            }),
                            Err(e) => {
                                error!("(anise): orbit.latlongalt: {}", e);
                                None
                            },
                        }
                    } else {
                        None
                    }
                } else {
                    if let Some(orbit) = ctx.base_rx_orbit(&obs_meta.meta) {
                        let pos_vel = orbit.to_cartesian_pos_vel();
                        match orbit.latlongalt() {
                            Ok(geodetic) => Some(RxPosition {
                                source: RxPositionSource::RINEx,
                                ecef_km: (pos_vel[0], pos_vel[1], pos_vel[2]),
                                geodetic,
                            }),
                            Err(e) => {
                                error!("(anise): orbit.latlongalt: {}", e);
                                None
                            },
                        }
                    } else {
                        None
                    }
                }
            },
        }
    }
}

pub struct QcObservationsSummary {
    pub nb_fileset: usize,
    pub summaries: Vec<QcObservationSummary>,
}

impl QcObservationsSummary {
    pub fn new(ctx: &QcContext) -> Self {
        let mut summaries = Vec::new();

        let metas = ctx.obs_dataset.keys().collect::<Vec<_>>();
        let nb_fileset = metas.len();

        for meta in metas.into_iter().unique() {
            if let Some(rinex) = ctx.obs_dataset.get(&meta) {
                summaries.push(QcObservationSummary::new(ctx, meta, &rinex))
            }
        }

        Self {
            nb_fileset,
            summaries,
        }
    }
}
