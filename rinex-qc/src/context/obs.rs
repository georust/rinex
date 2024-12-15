use crate::{
    cfg::QcPreferedObsSorting,
    context::{meta::MetaData, QcContext},
    prelude::Rinex,
    QcError,
};

use log::debug;

use std::collections::HashMap;

use qc_traits::{Filter, Merge, Preprocessing, Repair, RepairTrait};

pub enum ObservationUniqueId {
    Receiver(String),
    Antenna(String),
    GeodeticMarker(String),
}

impl std::fmt::Display for ObservationUniqueId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Receiver(rx) => write!(f, "rcvr:{}", rx),
            Self::Antenna(ant) => write!(f, "ant:{}", ant),
            Self::GeodeticMarker(geo) => write!(f, "geo:{}", geo),
        }
    }
}

impl std::str::FromStr for ObservationUniqueId {
    type Err = QcError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("rcvr:") {
            Ok(Self::Receiver(s[4..].to_string()))
        } else if s.starts_with("ant:") {
            Ok(Self::Antenna(s[3..].to_string()))
        } else if s.starts_with("geo") {
            Ok(Self::GeodeticMarker(s[3..].to_string()))
        } else {
            Err(QcError::DataIndexingIssue)
        }
    }
}

impl ObservationUniqueId {
    fn new(cfg: &QcPreferedObsSorting, rinex: &Rinex) -> Option<Self> {
        match cfg {
            QcPreferedObsSorting::Antenna => {
                if let Some(ant) = &rinex.header.rcvr_antenna {
                    Some(Self::Antenna(format!("{}-{}", ant.model, ant.sn)))
                } else {
                    None
                }
            },
            QcPreferedObsSorting::Geodetic => {
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
            QcPreferedObsSorting::Receiver => {
                if let Some(rcvr) = &rinex.header.rcvr {
                    Some(Self::Receiver(rcvr.model.to_string()))
                } else {
                    None
                }
            },
        }
    }
}

#[derive(Default)]
pub struct ObservationDataSet {
    /// Designated ROVER in case of RTK
    pub designated_rover: Option<String>,
    /// Observation [Rinex] sorted by [MetaData]
    pub inner: HashMap<MetaData, Rinex>,
}

impl std::fmt::Debug for ObservationDataSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(rover) = &self.designated_rover {
            write!(f, "Designated ROVER: {}", rover)?;
        }
        for meta in self.inner.keys() {
            write!(f, "Observation RINEX: {}", meta.name)?;
        }
        Ok(())
    }
}

impl ObservationDataSet {
    /// Applies [Filter] to whole data set without limitation
    pub fn filter_mut(&mut self, filter: &Filter) {
        for (_, inner) in self.inner.iter_mut() {
            inner.filter_mut(&filter);
        }
    }

    /// Applies [Repair]ment to whole data set without limitation
    pub fn repair_mut(&mut self, repair: Repair) {
        for (_, inner) in self.inner.iter_mut() {
            inner.repair_mut(repair);
        }
    }
}

impl QcContext {
    /// True if QcObservationsDataSet is not empty
    pub fn has_observations(&self) -> bool {
        self.observations.is_some()
    }

    /// Loads a new Observation [Rinex] into this [QcContext]
    pub(crate) fn load_observation_rinex(
        &mut self,
        meta: &mut MetaData,
        data: Rinex,
    ) -> Result<(), QcError> {
        // Designate an Indexing ID following prefered method
        if let Some(unique_id) = ObservationUniqueId::new(&self.cfg.obs_sorting, &data) {
            debug!(
                "{} designated by {} (prefered method)",
                meta.name, unique_id
            );
            meta.set_unique_id(&unique_id.to_string());
        }

        // Now proceed to stacking
        if let Some(observations) = &mut self.observations {
            if let Some(entry) = observations.inner.get_mut(&meta) {
                entry.merge_mut(&data)?;
            } else {
                observations.inner.insert(meta.clone(), data);
            }
        } else {
            // First Observation entry
            let mut data_set = ObservationDataSet::default();
            data_set.inner.insert(meta.clone(), data);
            self.observations = Some(data_set);
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {

    use crate::{
        cfg::{QcConfig, QcPreferedObsSorting},
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

        let observations = ctx.observations.expect("load_rinex failure");
        assert!(observations.designated_rover.is_none());
        assert_eq!(observations.inner.len(), 1);

        let key = MetaData {
            name: "ACOR00ESP_R_20213550000_01D_30S_MO".to_string(),
            extension: "rnx".to_string(),
            unique_id: Some("rcvr:LEICA GR50".to_string()),
        };

        assert!(
            observations.inner.get(&key).is_some(),
            "invalid gnss receiver indexing"
        );

        // Prefered: Geodetic
        let mut cfg = QcConfig::default();
        cfg.obs_sorting = QcPreferedObsSorting::Geodetic;

        let mut ctx = QcContext::new(cfg).unwrap();

        ctx.load_file(&path).unwrap();

        let observations = ctx.observations.as_ref().expect("load_rinex failure");
        assert!(observations.designated_rover.is_none());
        assert_eq!(observations.inner.len(), 1);

        let key = MetaData {
            name: "ACOR00ESP_R_20213550000_01D_30S_MO".to_string(),
            extension: "rnx".to_string(),
            unique_id: Some("geo:ACOR-13434M001".to_string()),
        };

        assert!(
            observations.inner.get(&key).is_some(),
            "invalid gnss receiver indexing"
        );

        // Prefered: Antenna
        let mut cfg = QcConfig::default();
        cfg.obs_sorting = QcPreferedObsSorting::Antenna;

        let mut ctx = QcContext::new(cfg).unwrap();

        ctx.load_file(&path).unwrap();

        let observations = ctx.observations.expect("load_rinex failure");
        assert!(observations.designated_rover.is_none());
        assert_eq!(observations.inner.len(), 1);

        let key = MetaData {
            name: "ACOR00ESP_R_20213550000_01D_30S_MO".to_string(),
            extension: "rnx".to_string(),
            unique_id: Some("ant:LEIAT504        LEIS-103033".to_string()),
        };

        assert!(
            observations.inner.get(&key).is_some(),
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
        ctx.load_file(&path_1).unwrap();

        let observations = ctx.observations.expect("load_rinex failure");
        assert!(observations.designated_rover.is_none());
        assert_eq!(observations.inner.len(), 1);

        let cfg = QcConfig::default();
        let mut ctx = QcContext::new(cfg).unwrap();

        ctx.load_file(&path_1).unwrap();
        ctx.load_file(&path_2).unwrap();

        let observations = ctx.observations.expect("load_rinex failure");
        assert!(observations.designated_rover.is_none());
        assert_eq!(observations.inner.len(), 2);
    }
}
