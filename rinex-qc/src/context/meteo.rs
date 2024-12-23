use rinex::prelude::Rinex;

use crate::{
    context::{meta::MetaData, QcContext},
    QcCtxError,
};

use qc_traits::Merge;

impl QcContext {
    pub fn has_meteo_data(&self) -> bool {
        !self.meteo_dataset.is_empty()
    }

    /// Loads a new Meteo [Rinex] into this [QcContext]
    pub(crate) fn load_meteo_rinex(
        &mut self,
        meta: &MetaData,
        data: Rinex,
    ) -> Result<(), QcCtxError> {
        if let Some(rinex) = self.meteo_dataset.get_mut(&meta) {
            rinex.merge_mut(&data)?;
        } else {
            self.meteo_dataset.insert(meta.clone(), data);
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {

    use crate::{cfg::QcConfig, context::QcContext};

    #[test]
    #[cfg(feature = "flate2")]
    fn meteo_indexing() {
        let path = format!(
            "{}/../test_resources/MET/V3/POTS00DEU_R_20232540000_01D_05M_MM.rnx.gz",
            env!("CARGO_MANIFEST_DIR")
        );

        // Default indexing
        let cfg = QcConfig::default();

        let mut ctx = QcContext::new(cfg).unwrap();

        ctx.load_gzip_file(&path).unwrap();
        assert!(ctx.has_meteo_data());
    }

    #[test]
    #[cfg(feature = "flate2")]
    fn meteo_stacking() {
        let path_1 = format!(
            "{}/../test_resources/MET/V3/POTS00DEU_R_20232540000_01D_05M_MM.rnx.gz",
            env!("CARGO_MANIFEST_DIR")
        );

        let path_2 = format!(
            "{}/../test_resources/MET/V2/abvi0010.15m",
            env!("CARGO_MANIFEST_DIR")
        );

        let cfg = QcConfig::default();
        let mut ctx = QcContext::new(cfg).unwrap();

        ctx.load_gzip_file(&path_1).unwrap();
        assert!(ctx.has_meteo_data());

        ctx.load_file(&path_2).unwrap();
        assert!(ctx.has_meteo_data());
    }
}
