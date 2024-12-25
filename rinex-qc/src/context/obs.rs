use crate::{
    cfg::preference::QcPreferedRoversSorting,
    context::{meta::MetaData, QcContext},
    prelude::Rinex,
    QcCtxError,
};

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
    /// True if QcObservationsDataSet is not empty
    pub fn has_observations(&self) -> bool {
        !self.obs_dataset.is_empty()
    }

    pub fn observations_meta(&self) -> Keys<'_, MetaData, Rinex> {
        self.obs_dataset.keys()
    }

    pub fn meta_rx_position_ecef(&self, meta: &MetaData) -> Option<(f64, f64, f64)> {
        for (k, v) in self.obs_dataset.iter() {
            if k == meta {
                return v.header.rx_position;
            }
        }
        None
    }

    pub fn meta_rx_orbit(&self, meta: &MetaData) -> Option<Orbit> {
        for (k, v) in self.obs_dataset.iter() {
            if k == meta {
                let t = v.first_epoch()?;
                return v.header.rx_orbit(t, self.earth_cef);
            }
        }
        None
    }

    /// Loads a new Observation [Rinex] into this [QcContext]
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

        // Now proceed to stacking
        if let Some(entry) = self.obs_dataset.get_mut(&meta) {
            entry.merge_mut(&data)?;
        } else {
            self.obs_dataset.insert(meta.clone(), data);
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {

    use crate::{
        cfg::{preference::QcPreferedRoversSorting, QcConfig},
        context::{meta::MetaData, QcContext},
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

        let key = MetaData {
            name: "ACOR00ESP_R_20213550000_01D_30S_MO".to_string(),
            extension: "rnx".to_string(),
            unique_id: Some("rcvr:LEICA GR50".to_string()),
        };

        assert!(
            ctx.obs_dataset.get(&key).is_some(),
            "invalid gnss receiver indexing"
        );

        // Prefered: Geodetic
        let mut cfg = QcConfig::default();
        cfg.preference.rovers_sorting = QcPreferedRoversSorting::Geodetic;

        let mut ctx = QcContext::new(cfg).unwrap();

        ctx.load_file(&path).unwrap();

        assert!(ctx.has_observations());
        assert_eq!(ctx.obs_dataset.len(), 1);

        let key = MetaData {
            name: "ACOR00ESP_R_20213550000_01D_30S_MO".to_string(),
            extension: "rnx".to_string(),
            unique_id: Some("geo:ACOR-13434M001".to_string()),
        };

        assert!(
            ctx.obs_dataset.get(&key).is_some(),
            "invalid gnss receiver indexing"
        );

        // Prefered: Antenna
        let mut cfg = QcConfig::default();
        cfg.preference.rovers_sorting = QcPreferedRoversSorting::Antenna;

        let mut ctx = QcContext::new(cfg).unwrap();

        ctx.load_file(&path).unwrap();

        assert!(ctx.has_observations());
        assert_eq!(ctx.obs_dataset.len(), 1);

        let key = MetaData {
            name: "ACOR00ESP_R_20213550000_01D_30S_MO".to_string(),
            extension: "rnx".to_string(),
            unique_id: Some("ant:LEIAT504        LEIS-103033".to_string()),
        };

        assert!(
            ctx.obs_dataset.get(&key).is_some(),
            "invalid gnss receiver indexing"
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
