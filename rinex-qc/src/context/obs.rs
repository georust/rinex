use crate::{
    cfg::preference::QcPreferedRoversSorting,
    context::{
        meta::{MetaData, ObsMetaData},
        QcContext,
    },
    prelude::Rinex,
    QcCtxError,
};

use anise::math::Vector6;
use rinex::prelude::Epoch;

use std::collections::hash_map::Keys;

use log::debug;

use qc_traits::Merge;

pub enum ObservationUniqueId {
    Receiver(String),
    Antenna(String),
    Operator(String),
    GeodeticMarker(String),
}

use anise::prelude::Orbit;

impl std::str::FromStr for ObservationUniqueId {
    type Err = QcCtxError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("rcvr:") {
            Ok(Self::Receiver(s[5..].to_string()))
        } else if s.starts_with("ant:") {
            Ok(Self::Antenna(s[4..].to_string()))
        } else if s.starts_with("geo:") {
            Ok(Self::GeodeticMarker(s[4..].to_string()))
        } else if s.starts_with("op:") {
            Ok(Self::Operator(s[3..].to_string()))
        } else {
            Err(QcCtxError::DataIndexing)
        }
    }
}

impl std::fmt::UpperExp for ObservationUniqueId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Receiver(rx) => write!(f, "rcvr={}", rx),
            Self::Operator(op) => write!(f, "operator={}", op),
            Self::Antenna(ant) => write!(f, "antenna={}", ant),
            Self::GeodeticMarker(geo) => write!(f, "geodetic={}", geo),
        }
    }
}

impl std::fmt::LowerHex for ObservationUniqueId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Receiver(rx) => write!(f, "rcvr:{}", rx),
            Self::Operator(op) => write!(f, "op:{}", op),
            Self::Antenna(ant) => write!(f, "ant:{}", ant),
            Self::GeodeticMarker(geo) => write!(f, "geo:{}", geo),
        }
    }
}

impl std::fmt::Display for ObservationUniqueId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Receiver(rx) => write!(f, "{}", rx),
            Self::Operator(op) => write!(f, "{}", op),
            Self::Antenna(ant) => write!(f, "{}", ant),
            Self::GeodeticMarker(geo) => write!(f, "{}", geo),
        }
    }
}

impl ObservationUniqueId {
    fn new(cfg: &QcPreferedRoversSorting, rinex: &Rinex) -> Option<Self> {
        match cfg {
            QcPreferedRoversSorting::Antenna => {
                if let Some(ant) = &rinex.header.rcvr_antenna {
                    Some(Self::Antenna(format!("{}-{}", ant.model, ant.sn)))
                } else {
                    None
                }
            },
            QcPreferedRoversSorting::Geodetic => {
                if let Some(marker) = &rinex.header.geodetic_marker {
                    if let Some(number) = marker.number() {
                        Some(Self::GeodeticMarker(format!("{}-{}", marker.name, number)))
                    } else {
                        Some(Self::GeodeticMarker(marker.name.to_string()))
                    }
                } else {
                    None
                }
            },
            QcPreferedRoversSorting::Receiver => {
                if let Some(rcvr) = &rinex.header.rcvr {
                    Some(Self::Receiver(format!("{}-{}", rcvr.model, rcvr.sn)))
                } else {
                    None
                }
            },
            QcPreferedRoversSorting::Operator => {
                if let Some(operator) = &rinex.header.observer {
                    Some(Self::Operator(operator.to_string()))
                } else {
                    None
                }
            },
        }
    }
}

impl QcContext {
    pub fn has_observations(&self) -> bool {
        !self.obs_dataset.is_empty()
    }

    pub fn has_rover_observations(&self) -> bool {
        self.observations_meta().filter(|k| k.is_rover).count() > 0
    }

    pub fn has_base_observations(&self) -> bool {
        self.observations_meta().filter(|k| !k.is_rover).count() > 0
    }

    /// Define the following abstract name as a "rover".
    /// This has no effect if no "base station" with following [MetaData] has previously defined.
    pub fn define_rover(&mut self, meta: MetaData) {}

    /// Define the following abstract name as a "base" station reference.
    pub fn define_base(&mut self, meta: MetaData) {}

    /// "Swap" rover / bases definitions:
    /// - All previously defined base stations become "rovers"
    /// - All previousy defined rovers become "base stations"
    /// This has no effect if at least one base station was previously defined.
    pub fn swap_rover_base_definitions(&mut self) {}

    /// [ObsMetaData] iterator, whatever their type
    pub fn observations_meta(&self) -> Keys<'_, ObsMetaData, Rinex> {
        self.obs_dataset.keys()
    }

    /// Rover [ObsMetaData] iterator specifically
    pub fn rover_observations_meta(&self) -> Box<dyn Iterator<Item = &ObsMetaData> + '_> {
        Box::new(self.obs_dataset.keys().filter(|meta| meta.is_rover))
    }

    /// Base stations [ObsMetaData] iterator specifically
    pub fn base_observations_meta(&self) -> Box<dyn Iterator<Item = &ObsMetaData> + '_> {
        Box::new(self.obs_dataset.keys().filter(|meta| !meta.is_rover))
    }

    pub fn rover_rx_position_ecef(&self, meta: &MetaData) -> Option<(f64, f64, f64)> {
        for (k, v) in self.obs_dataset.iter() {
            if k.is_rover {
                if &k.meta == meta {
                    return v.header.rx_position;
                }
            }
        }
        None
    }

    pub fn rover_rx_orbit(&self, meta: &MetaData) -> Option<Orbit> {
        let t0 = self.rover_first_observation(&meta)?;
        let (x_ecef_m, y_ecef_m, z_ecef_m) = self.rover_rx_position_ecef(meta)?;
        let pos_vel = Vector6::new(
            x_ecef_m / 1000.0,
            y_ecef_m / 1000.0,
            z_ecef_m / 1000.0,
            0.0,
            0.0,
            0.0,
        );
        Some(Orbit::from_cartesian_pos_vel(pos_vel, t0, self.earth_cef))
    }

    pub fn rover_first_observation(&self, meta: &MetaData) -> Option<Epoch> {
        for (k, v) in self.obs_dataset.iter() {
            if k.is_rover {
                if &k.meta == meta {
                    return v.first_epoch();
                }
            }
        }

        None
    }

    pub fn base_rx_position_ecef(&self, meta: &MetaData) -> Option<(f64, f64, f64)> {
        for (k, v) in self.obs_dataset.iter() {
            if !k.is_rover {
                if &k.meta == meta {
                    return v.header.rx_position;
                }
            }
        }
        None
    }

    pub fn base_first_observation(&self, meta: &MetaData) -> Option<Epoch> {
        for (k, v) in self.obs_dataset.iter() {
            if !k.is_rover {
                if &k.meta == meta {
                    return v.first_epoch();
                }
            }
        }
        None
    }

    pub fn base_rx_orbit(&self, meta: &MetaData) -> Option<Orbit> {
        let t0 = self.base_first_observation(&meta)?;
        let (x_ecef_m, y_ecef_m, z_ecef_m) = self.base_rx_position_ecef(meta)?;
        let pos_vel = Vector6::new(
            x_ecef_m / 1000.0,
            y_ecef_m / 1000.0,
            z_ecef_m / 1000.0,
            0.0,
            0.0,
            0.0,
        );
        Some(Orbit::from_cartesian_pos_vel(pos_vel, t0, self.earth_cef))
    }

    /// Loads a new Observation [Rinex] into this [QcContext].
    /// NB: this is by default considered "rover" data, which is compliant
    /// with direct n-D direct positioning. If you're interested in RTK (differential positioning),
    /// you will have to provmide more Observation [Rinex] and manually
    /// specify which one correspond to a base station.
    pub(crate) fn load_observation_rinex(
        &mut self,
        meta: &mut MetaData,
        data: Rinex,
    ) -> Result<(), QcCtxError> {
        // Designate an Indexing ID following prefered method
        if let Some(unique_id) =
            ObservationUniqueId::new(&self.cfg.preference.rovers_sorting, &data)
        {
            debug!(
                "{} designated by {} (prefered method)",
                meta.name, unique_id
            );

            meta.set_unique_id(&format!("{:x}", unique_id));
        }

        let obs_meta = ObsMetaData::from_meta(meta.clone());

        // Now proceed to stacking
        if let Some(entry) = self.obs_dataset.get_mut(&obs_meta) {
            entry.merge_mut(&data)?;
        } else {
            self.obs_dataset.insert(obs_meta, data);
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {

    use crate::{
        cfg::{preference::QcPreferedRoversSorting, QcConfig},
        context::{
            meta::{MetaData, ObsMetaData},
            QcContext,
        },
    };

    #[test]
    fn observation_indexing() {
        let path = format!(
            "{}/../test_resources/OBS/V3/ACOR00ESP_R_20213550000_01D_30S_MO.rnx",
            env!("CARGO_MANIFEST_DIR")
        );

        // Default indexing
        let cfg = QcConfig::default();

        let mut ctx = QcContext::new(cfg).unwrap();

        ctx.load_file(&path).unwrap();

        assert!(ctx.has_observations());
        assert_eq!(ctx.obs_dataset.len(), 1);

        let key = ObsMetaData::from_meta(MetaData {
            name: "ACOR00ESP_R_20213550000_01D_30S_MO".to_string(),
            extension: "rnx".to_string(),
            unique_id: Some("rcvr:LEICA GR50-1833574".to_string()),
        });

        assert!(
            ctx.obs_dataset.get(&key).is_some(),
            "invalid gnss receiver indexing: {:?}",
            ctx.obs_dataset.keys().collect::<Vec<_>>(),
        );

        // Prefered: Geodetic
        let mut cfg = QcConfig::default();
        cfg.preference.rovers_sorting = QcPreferedRoversSorting::Geodetic;

        let mut ctx = QcContext::new(cfg).unwrap();

        ctx.load_file(&path).unwrap();

        assert!(ctx.has_observations());
        assert_eq!(ctx.obs_dataset.len(), 1);

        let key = ObsMetaData::from_meta(MetaData {
            name: "ACOR00ESP_R_20213550000_01D_30S_MO".to_string(),
            extension: "rnx".to_string(),
            unique_id: Some("geo:ACOR-13434M001".to_string()),
        });

        assert!(
            ctx.obs_dataset.get(&key).is_some(),
            "invalid gnss receiver indexing: {:?}",
            ctx.obs_dataset.keys().collect::<Vec<_>>(),
        );

        // Prefered: Antenna
        let mut cfg = QcConfig::default();
        cfg.preference.rovers_sorting = QcPreferedRoversSorting::Antenna;

        let mut ctx = QcContext::new(cfg).unwrap();

        ctx.load_file(&path).unwrap();

        assert!(ctx.has_observations());
        assert_eq!(ctx.obs_dataset.len(), 1);

        let key = ObsMetaData::from_meta(MetaData {
            name: "ACOR00ESP_R_20213550000_01D_30S_MO".to_string(),
            extension: "rnx".to_string(),
            unique_id: Some("ant:LEIAT504        LEIS-103033".to_string()),
        });

        assert!(
            ctx.obs_dataset.get(&key).is_some(),
            "invalid gnss receiver indexing: {:?}",
            ctx.obs_dataset.keys().collect::<Vec<_>>(),
        );
    }

    #[test]
    fn observation_stacking() {
        let path_1 = format!(
            "{}/../test_resources/OBS/V3/ACOR00ESP_R_20213550000_01D_30S_MO.rnx",
            env!("CARGO_MANIFEST_DIR")
        );

        let path_2 = format!(
            "{}/../test_resources/OBS/V3/VLNS0630.22O",
            env!("CARGO_MANIFEST_DIR")
        );

        let cfg = QcConfig::default();
        let mut ctx = QcContext::new(cfg).unwrap();

        ctx.load_file(&path_1).unwrap();
        assert!(ctx.has_observations());

        ctx.load_file(&path_1).unwrap();
        assert!(ctx.has_observations());

        assert_eq!(ctx.obs_dataset.len(), 1);

        let cfg = QcConfig::default();
        let mut ctx = QcContext::new(cfg).unwrap();

        ctx.load_file(&path_1).unwrap();
        assert!(ctx.has_observations());

        ctx.load_file(&path_2).unwrap();
        assert!(ctx.has_observations());
        assert_eq!(ctx.obs_dataset.len(), 2);
    }
}
