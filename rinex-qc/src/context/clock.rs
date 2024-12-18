use crate::{
    context::{meta::MetaData, QcContext},
    prelude::Rinex,
    QcError,
};

use std::collections::HashMap;

use rinex::prelude::DOMES;

use qc_traits::{Filter, Merge, Preprocessing, Repair, RepairTrait};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ClockUniqueId {
    Agency(String),
    DOMES(DOMES),
}

impl std::fmt::Display for ClockUniqueId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Agency(ag) => write!(f, "ag:{}", ag),
            Self::DOMES(domes) => write!(f, "dom:{}", domes),
        }
    }
}

impl std::str::FromStr for ClockUniqueId {
    type Err = QcError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("ag:") {
            Ok(Self::Agency(s[3..].to_string()))
        } else if s.starts_with("dom:") {
            if let Ok(domes) = DOMES::from_str(&s[4..]) {
                Ok(Self::DOMES(domes))
            } else {
                Err(QcError::DataIndexingIssue)
            }
        } else {
            Err(QcError::DataIndexingIssue)
        }
    }
}

#[derive(Default)]
pub struct ClockDataSet {
    pub inner: HashMap<MetaData, Rinex>,
}

impl std::fmt::Debug for ClockDataSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for meta in self.inner.keys() {
            write!(f, "Clock RINEX: {}", meta.name)?;
        }
        Ok(())
    }
}

impl ClockDataSet {
    pub fn filter_mut(&mut self, filter: &Filter) {
        for (_, inner) in self.inner.iter_mut() {
            inner.filter_mut(filter);
        }
    }

    pub fn repair_mut(&mut self, repair: Repair) {
        for (_, inner) in self.inner.iter_mut() {
            inner.repair_mut(repair);
        }
    }
}

impl QcContext {
    /// Returns true if [QcContext] contains at least one
    /// precise clock RINEX product.
    pub fn has_precise_clock(&self) -> bool {
        self.clk_dataset.is_some()
    }

    pub fn clock_iter(&self) -> Box<dyn Iterator<Item = (Epoch, SV, f64)> + 'a> {}

    pub(crate) fn load_clock_rinex(
        &mut self,
        meta: &mut MetaData,
        data: Rinex,
    ) -> Result<(), QcError> {
        // designate an indexing ID
        if let Some(agency) = &data.header.agency {
            let unique_id = ClockUniqueId::Agency(agency.to_string());
            meta.set_unique_id(&unique_id.to_string());
        }

        // stack data
        if let Some(clock_dataset) = &mut self.clk_dataset {
            if let Some(entry) = clock_dataset.inner.get_mut(&meta) {
                entry.merge_mut(&data);
            } else {
                clock_dataset.inner.insert(meta.clone(), data);
            }
        } else {
            let mut data_set = ClockDataSet::default();
            data_set.inner.insert(meta.clone(), data);
            self.clk_dataset = Some(data_set);
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
    fn clock_indexing() {
        let path = format!(
            "{}/../test_resources/CLK/V2/COD20352.CLK",
            env!("CARGO_MANIFEST_DIR")
        );

        // Default indexing
        let cfg = QcConfig::default();

        let mut ctx = QcContext::new(cfg).unwrap();

        ctx.load_file(&path).unwrap();
        assert!(ctx.has_precise_clock());

        let clk_dataset = ctx.clk_dataset.expect("load_rinex failure");
        assert_eq!(clk_dataset.inner.len(), 1);

        let key = MetaData {
            name: "COD20352".to_string(),
            extension: "CLK".to_string(),
            unique_id: Some("ag:COD".to_string()),
        };

        assert!(
            clk_dataset.inner.get(&key).is_some(),
            "invalid clock agency indexing"
        );
    }

    #[test]
    fn clock_stacking() {
        let path_1 = format!(
            "{}/../test_resources/CLK/V2/COD20352.CLK",
            env!("CARGO_MANIFEST_DIR")
        );

        let path_2 = format!(
            "{}/../test_resources/CLK/V2/COD21925.CLK_05S",
            env!("CARGO_MANIFEST_DIR")
        );

        let cfg = QcConfig::default();
        let mut ctx = QcContext::new(cfg).unwrap();

        ctx.load_file(&path_1).unwrap();
        assert!(ctx.has_precise_clock());

        ctx.load_file(&path_2).unwrap();
        assert!(ctx.has_precise_clock());
    }
}
